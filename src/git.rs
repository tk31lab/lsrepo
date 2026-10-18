//! Git 情報の取得。いまは git コマンドを呼んでいるが、
//! 公開している関数のシグネチャを保ったまま gix に差し替えられるようにしてある。

use std::fmt;
use std::path::Path;
use std::process::Command;

/// HEAD の状態
#[derive(Debug, PartialEq, Eq)]
pub enum Head {
    /// ブランチ上にいる(コミットがまだ無いブランチも含む)
    Branch(String),
    /// detached HEAD。中身は短縮コミットハッシュ
    Detached(String),
}

impl fmt::Display for Head {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Head::Branch(name) => f.write_str(name),
            Head::Detached(hash) => write!(f, "(detached {hash})"),
        }
    }
}

/// HEAD の状態を取得する。git が失敗したら `None`。
pub fn head(path: &Path) -> Option<Head> {
    let branch = run_git(path, &["branch", "--show-current"])?;
    let branch = branch.trim();
    if !branch.is_empty() {
        return Some(Head::Branch(branch.to_string()));
    }
    let hash = run_git(path, &["rev-parse", "--short", "HEAD"])?;
    Some(Head::Detached(hash.trim().to_string()))
}

/// 未コミットの変更数(untracked を含む)。git が失敗したら `None`。
pub fn uncommitted_count(path: &Path) -> Option<usize> {
    let status = run_git(path, &["status", "--porcelain"])?;
    Some(status.lines().filter(|line| !line.is_empty()).count())
}

fn run_git(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
            .args(args)
            .output()
            .unwrap()
            .status;
        assert!(status.success(), "git {args:?} failed");
    }

    #[test]
    fn branch_detached_and_counts() {
        let tmp = TempDir::new("git");
        let repo = tmp.path();
        git(repo, &["init", "-q", "-b", "main"]);

        // コミットが無くてもブランチ名は取れる
        assert_eq!(head(repo), Some(Head::Branch("main".into())));
        assert_eq!(uncommitted_count(repo), Some(0));

        fs::write(repo.join("a.txt"), "a").unwrap();
        assert_eq!(uncommitted_count(repo), Some(1));

        git(repo, &["add", "a.txt"]);
        git(repo, &["commit", "-q", "-m", "first"]);
        fs::write(repo.join("a.txt"), "changed").unwrap();
        fs::write(repo.join("b.txt"), "b").unwrap();
        assert_eq!(uncommitted_count(repo), Some(2));

        git(repo, &["checkout", "-q", "--detach"]);
        match head(repo) {
            Some(Head::Detached(hash)) => assert!(!hash.is_empty()),
            other => panic!("expected detached, got {other:?}"),
        }
    }

    #[test]
    fn not_a_repo() {
        let tmp = TempDir::new("notrepo");
        assert_eq!(head(tmp.path()), None);
        assert_eq!(uncommitted_count(tmp.path()), None);
    }

    #[test]
    fn display() {
        assert_eq!(Head::Branch("main".into()).to_string(), "main");
        assert_eq!(
            Head::Detached("abc1234".into()).to_string(),
            "(detached abc1234)"
        );
    }
}
