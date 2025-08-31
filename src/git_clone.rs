use git2::*;

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
    println!("basename: {}", basename);

    let clone_path = path.as_ref().join(basename);
    println!("clone path: {:?}", clone_path);

    Repository::clone(&url, clone_path)
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
