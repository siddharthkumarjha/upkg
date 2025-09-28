use crate::*;
use std::sync::LazyLock;

use git2::*;
use indicatif::*;

fn git_url_basename(repo: &str) -> String {
    let mut base_name = match repo.split_once("://") {
        Some((_, rhs)) => rhs,
        None => repo,
    };

    let mut last_at = None;
    base_name
        .char_indices()
        .take_while(|(_, c)| !std::path::is_separator(*c))
        .skip_while(|(_, c)| *c != '@')
        .for_each(|(idx, _)| last_at = Some(idx));

    if let Some(idx) = last_at {
        base_name = &base_name[idx + 1..];
    }

    base_name = base_name.trim_end_matches(|c| std::path::is_separator(c) || c.is_whitespace());

    let suffix = std::format!("{}{}", std::path::MAIN_SEPARATOR, ".git");
    base_name = base_name.trim_end_matches(suffix.as_str());
    base_name = base_name.trim_end_matches(std::path::is_separator);

    if !base_name.contains('/') && base_name.contains(':') {
        let mut ptr = base_name.len();
        while ptr > 0 {
            let ch = base_name.as_bytes()[ptr - 1] as char;
            if ch.is_ascii_digit() {
                ptr -= 1;
                continue;
            }
            if ch == ':' {
                base_name = &base_name[..ptr];
            }
            break;
        }
    }

    if let Some(pos) = base_name.rfind(|c| std::path::is_separator(c) || c == ':') {
        base_name = &base_name[pos + 1..];
    }

    base_name = base_name.trim_end_matches(".bundle");
    base_name = base_name.trim_end_matches(".git");

    if base_name.len() <= 0 || (base_name.len() == 1 && base_name.chars().next().unwrap() == '/') {
        panic!("no dir name could be guessed, pls specify a dir name on command line");
    }

    base_name = base_name.trim_start();
    base_name = base_name.trim_end_matches(std::path::is_separator);

    return base_name.to_string();
}

fn checkout_branch(repo: &Repository, branch_name: &str, force: bool) -> Result<(), Error> {
    // Try to find a local branch first
    match repo.find_branch(branch_name, BranchType::Local) {
        Ok(branch) => {
            let commit = git_ok!(branch.get().peel_to_commit());

            let mut opts = build::CheckoutBuilder::new();
            if force {
                opts.force();
            } else {
                opts.safe();
            }
            git_ok!(repo.checkout_tree(commit.as_object(), Some(&mut opts)));

            let branch_ref = git_ok!(
                branch
                    .get()
                    .name()
                    .ok_or_else(|| Error::from_str("Invalid branch name"))
            );
            git_ok!(repo.set_head(branch_ref));
        }
        Err(_) => {
            // Try remote branch: "origin/<branch_name>"
            let remote_branch_name = format!("origin/{}", branch_name);
            let remote_branch = git_ok!(repo.find_branch(&remote_branch_name, BranchType::Remote));
            let commit = git_ok!(remote_branch.get().peel_to_commit());

            // Create local branch that tracks the remote
            let mut local = git_ok!(repo.branch(branch_name, &commit, false));
            git_ok!(local.set_upstream(Some(&remote_branch_name)));

            let mut opts = build::CheckoutBuilder::new();
            if force {
                opts.force();
            } else {
                opts.safe();
            }

            git_ok!(repo.checkout_tree(commit.as_object(), Some(&mut opts)));
            git_ok!(repo.set_head(&format!("refs/heads/{}", branch_name)));
        }
    }

    Ok(())
}

fn checkout_tag(repo: &Repository, tag_name: &str) -> Result<(), git2::Error> {
    // Resolve tag to object
    let obj = git_ok!(repo.revparse_single(&format!("refs/tags/{}", tag_name)));
    let commit = git_ok!(obj.peel_to_commit()); // peel in case it’s an annotated tag
    let tree = git_ok!(commit.tree());

    // Checkout files
    git_ok!(repo.checkout_tree(tree.as_object(), None));

    // Detach HEAD to this commit
    git_ok!(repo.set_head_detached(commit.id()));

    Ok(())
}

fn latest_tag_by_creation(repo: &Repository) -> Result<Option<String>, Error> {
    let tag_names = git_ok!(repo.tag_names(None));
    let mut latest: Option<(String, i64)> = None;

    for tag_name in tag_names.iter().flatten() {
        if let Ok(obj) = repo.revparse_single(tag_name) {
            let tag_time = if obj.kind() == Some(ObjectType::Tag) {
                // Annotated tag: use tagger timestamp
                let tag = obj.as_tag().unwrap();
                tag.tagger().map(|sig| sig.when().seconds()).unwrap_or(0)
            } else if obj.kind() == Some(ObjectType::Commit) {
                // Lightweight tag: use commit timestamp as fallback
                let commit = git_ok!(obj.peel_to_commit());
                commit.time().seconds()
            } else {
                0
            };

            match latest {
                Some((_, ts)) if ts >= tag_time => {}
                _ => latest = Some((tag_name.to_string(), tag_time)),
            }
        }
    }

    Ok(latest.map(|(name, _)| name))
}

fn fetch_repo<RepoPath: AsRef<std::path::Path>>(
    url: &str,
    clone_path: RepoPath,
    basename: String,
) -> Result<Repository, Error> {
    let repo = git_ok!(Repository::open(&clone_path));

    {
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(setup_rmt_callbacks(basename));

        let mut remote = git_ok!(
            repo.find_remote("origin")
                .or_else(|_| repo.remote("origin", url))
        );

        git_ok!(remote.fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_opts),
            None,
        ));
    }

    Ok(repo)
}

fn clone_repo<RepoPath: AsRef<std::path::Path>>(
    url: &str,
    clone_path: RepoPath,
    basename: String,
) -> Result<Repository, Error> {
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(setup_rmt_callbacks(basename));

    let mut repo_handle = build::RepoBuilder::new();
    repo_handle.fetch_options(fetch_opts);

    repo_handle
        .clone(url, clone_path.as_ref())
        .map_err(git_err_ctx!())
}

static SEMVER_RE: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\d+").unwrap());

fn normalize_tag_to_semver(tag: &str) -> Vec<u32> {
    // Matches consecutive digits
    return SEMVER_RE
        .find_iter(tag)
        .filter_map(|m| m.as_str().parse::<u32>().ok())
        .collect();
}

pub fn git_sync_with_remote<RepoPath: AsRef<std::path::Path>>(
    url: &str,
    path: RepoPath,
    repo_name: Option<&str>,
    checkout: &CheckoutType,
) -> Result<Repository, Error> {
    let basename = match repo_name {
        Some(val) => val.to_string(),
        None => git_url_basename(&url),
    };
    println!("attempting to clone: {url}");

    let clone_path = path.as_ref().join(&basename);
    println!("clone path: {:?}", clone_path);

    if clone_path.exists() {
        println!("path exists, trying to sync repo with remote...");
        let repo = git_ok!(fetch_repo(url, clone_path, basename));

        {
            match checkout {
                CheckoutType::tag(tag) => {
                    let norm_tag = normalize_tag_to_semver(&tag);

                    if let Some(new_tag) = git_ok!(latest_tag_by_creation(&repo)) {
                        let new_tag_norm = normalize_tag_to_semver(&new_tag);
                        if norm_tag < new_tag_norm {
                            println!("updating HEAD to tag: {}", new_tag);
                            git_ok!(checkout_tag(&repo, &new_tag));
                        } else {
                            println!("not updating HEAD, old tag: {}, new tag: {}", tag, new_tag);
                        }
                    }
                }
                CheckoutType::branch(branch) => {
                    let remote_ref = format!("refs/remotes/origin/{}", branch);

                    let target_ref_id = git_ok!(repo.refname_to_id(&remote_ref));
                    let target_commit = git_ok!(repo.find_commit(target_ref_id));

                    println!("updating HEAD to branch: {}", branch);
                    git_ok!(repo.reset(target_commit.as_object(), ResetType::Hard, None));
                }
                CheckoutType::none => (),
            }
        }

        Ok(repo)
    } else {
        println!("trying to clone repo...");

        let repo_handle = git_ok!(clone_repo(url, clone_path, basename));
        match checkout {
            CheckoutType::tag(tag) => {
                println!("checkout to tag: {}", tag);
                if let Some(new_tag) = git_ok!(latest_tag_by_creation(&repo_handle)) {
                    git_ok!(checkout_tag(&repo_handle, &new_tag));
                }
            }
            CheckoutType::branch(branch) => {
                println!("checkout to branch: {}", &branch);
                git_ok!(git_clone::checkout_branch(&repo_handle, &branch, false));
            }
            CheckoutType::none => (),
        }

        Ok(repo_handle)
    }
}

static SIDEBAND_PROGRESS_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^(Counting|Compressing) objects:\s+(\d+)% \((\d+)/(\d+)\)").unwrap()
});

fn setup_rmt_callbacks<'a>(basename: String) -> RemoteCallbacks<'a> {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{prefix}]: {msg}: [{bar:40.bold.dim}] {pos}/{len} ({percent}%)")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_prefix(basename.to_owned());
    pb.set_message("Git Clone");

    let pb_rc = std::rc::Rc::new(pb);
    let pb_transfer = pb_rc.clone();
    let pb_sideband = pb_rc.clone();

    let mut callbacks = RemoteCallbacks::new();

    callbacks.transfer_progress(move |prog: Progress<'_>| {
        if prog.total_objects() != prog.received_objects() {
            pb_transfer.set_length(prog.total_objects() as u64);
            pb_transfer.set_position(prog.received_objects() as u64);
            pb_transfer.set_message("Receiving objects");
        } else {
            pb_transfer.set_length(prog.total_deltas() as u64);
            pb_transfer.set_position(prog.indexed_deltas() as u64);
            pb_transfer.set_message("Resolving deltas");
        }
        true
    });

    callbacks.sideband_progress(move |data: &[u8]| {
        if let Ok(msg) = str::from_utf8(data) {
            if let Some(caps) = SIDEBAND_PROGRESS_RE.captures(msg) {
                let stage = &caps[1];
                let current: u64 = caps[3].parse().unwrap();
                let total: u64 = caps[4].parse().unwrap();

                pb_sideband.set_length(total);
                pb_sideband.set_position(current);

                pb_sideband.set_message(stage.to_owned());
            }
        }
        true
    });

    callbacks
}

#[cfg(test)]
mod tests {
    use super::git_url_basename;

    #[test]
    fn test_https_url_with_git_suffix() {
        assert_eq!(git_url_basename("https://github.com/user/repo.git"), "repo");
    }

    #[test]
    fn test_https_url_without_git_suffix() {
        assert_eq!(git_url_basename("https://github.com/user/repo"), "repo");
    }

    #[test]
    fn test_ssh_url_with_git_suffix() {
        assert_eq!(git_url_basename("git@github.com:user/repo.git"), "repo");
    }

    #[test]
    fn test_ssh_url_without_git_suffix() {
        assert_eq!(git_url_basename("git@github.com:user/repo"), "repo");
    }

    #[test]
    fn test_file_path_with_git_suffix() {
        assert_eq!(git_url_basename("/home/user/projects/repo.git"), "repo");
    }

    #[test]
    fn test_file_path_without_git_suffix() {
        assert_eq!(git_url_basename("/home/user/projects/repo"), "repo");
    }

    #[test]
    fn test_only_repo_name() {
        assert_eq!(git_url_basename("repo.git"), "repo");
    }

    #[test]
    fn test_only_repo_name_without_git() {
        assert_eq!(git_url_basename("repo"), "repo");
    }

    #[test]
    fn test_port_num_uri() {
        assert_eq!(git_url_basename("/foo/bar:2222.git"), "2222");
        assert_eq!(
            git_url_basename("ssh://git@example.com:2222/myrepo.git"),
            "myrepo"
        );
        assert_eq!(
            git_url_basename("https://example.com:8443/myrepo.git"),
            "myrepo"
        );
        assert_eq!(
            git_url_basename("git://example.com:9419/myrepo.git"),
            "myrepo"
        );
    }

    #[test]
    #[should_panic(expected = "no dir name could be guessed")]
    fn test_empty_string() {
        git_url_basename("");
    }
}
