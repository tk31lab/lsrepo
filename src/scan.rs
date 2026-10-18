use std::fs;
use std::path::{Path, PathBuf};

/// 潜らないディレクトリ名
const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target"];

/// `root` 以下にある Git リポジトリのパスを列挙する(ソート済み)。
///
/// `.git` (ディレクトリまたはファイル) を持つディレクトリをリポジトリとみなす。
/// `.git` の中には潜らないが、リポジトリ内の他のディレクトリは探索を続けるので
/// ネストしたリポジトリも見つかる。読めないディレクトリは黙ってスキップする。
pub fn find_repos(root: &Path) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    walk(root, &mut repos);
    repos.sort();
    repos
}

fn walk(dir: &Path, repos: &mut Vec<PathBuf>) {
    if dir.join(".git").exists() {
        repos.push(dir.to_path_buf());
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        // file_type() はシンボリックリンクをたどらないので、リンクのループを踏まない
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        if SKIP_DIRS.iter().any(|skip| name == *skip) {
            continue;
        }
        walk(&entry.path(), repos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    #[test]
    fn finds_repos_and_skips_ignored_dirs() {
        let tmp = TempDir::new("scan");
        let root = tmp.path();
        for dir in [
            "a/.git",
            "a/nested/.git",
            "b/c/.git",
            "node_modules/x/.git",
            "target/y/.git",
            "a/.git/fake/.git",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        // worktree / submodule のように .git がファイルの場合
        fs::create_dir_all(root.join("wt")).unwrap();
        fs::write(root.join("wt/.git"), "gitdir: /somewhere\n").unwrap();

        let found: Vec<_> = find_repos(root)
            .into_iter()
            .map(|p| p.strip_prefix(root).unwrap().to_path_buf())
            .collect();
        let expected: Vec<PathBuf> = ["a", "a/nested", "b/c", "wt"]
            .iter()
            .map(PathBuf::from)
            .collect();
        assert_eq!(found, expected);
    }
}
