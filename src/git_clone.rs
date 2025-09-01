use crate::io_err_context;
use std::sync::LazyLock;

use git2::*;
use indicatif::*;

pub fn git_url_basename(repo: &str) -> String {
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

pub fn git_clone<RepoPath: AsRef<std::path::Path>>(
    url: &str,
    path: RepoPath,
    repo_name: Option<&str>,
) -> Result<Repository, Error> {
    let basename = match repo_name {
        Some(val) => val.to_string(),
        None => git_url_basename(&url),
    };

    println!("attempting to clone: {url}");

    let clone_path = path.as_ref().join(&basename);
    println!("clone path: {:?}", clone_path);

    if clone_path.exists() {
        println!("path exists, removing it...");
        std::fs::remove_dir_all(&clone_path)
            .map_err(io_err_context!())
            .map_err(|err| -> git2::Error {
                let err_msg = std::format!("{}", err);
                git2::Error::new(git2::ErrorCode::User, git2::ErrorClass::Os, err_msg)
            })?;
    }

    let mut fetch_opts = FetchOptions::new();
    let mut repo_handle = build::RepoBuilder::new();

    fetch_opts.remote_callbacks(setup_rmt_callbacks(basename));
    repo_handle.fetch_options(fetch_opts);

    repo_handle
        .clone(&url, &clone_path)
        .map_err(|err| -> git2::Error {
            let err_msg = format!("[{}:{}] {}", file!(), line!(), err);
            git2::Error::from_str(&err_msg)
        })
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

    pb.abandon();
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
