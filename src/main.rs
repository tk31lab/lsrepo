mod git;
mod scan;
#[cfg(test)]
mod testutil;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

struct Row {
    name: String,
    branch: String,
    uncommitted: String,
}

fn main() -> ExitCode {
    let root = match std::env::args_os().nth(1) {
        Some(arg) => PathBuf::from(arg),
        None => PathBuf::from("."),
    };
    if !root.is_dir() {
        eprintln!("lsrepo: {}: ディレクトリではありません", root.display());
        return ExitCode::FAILURE;
    }

    let rows: Vec<Row> = scan::find_repos(&root)
        .iter()
        .map(|repo| Row {
            name: display_name(&root, repo),
            branch: git::head(repo).map_or("?".to_string(), |head| head.to_string()),
            uncommitted: git::uncommitted_count(repo).map_or("?".to_string(), |n| n.to_string()),
        })
        .collect();

    if rows.is_empty() {
        eprintln!("lsrepo: Git リポジトリが見つかりませんでした");
        return ExitCode::SUCCESS;
    }

    let name_width = rows
        .iter()
        .map(|r| r.name.chars().count())
        .max()
        .unwrap_or(0);
    let branch_width = rows
        .iter()
        .map(|r| r.branch.chars().count())
        .max()
        .unwrap_or(0);
    for row in &rows {
        println!(
            "{:<name_width$}  {:<branch_width$}  {:>3}",
            row.name, row.branch, row.uncommitted
        );
    }
    ExitCode::SUCCESS
}

/// 表示名: root からの相対パス。root 自体がリポジトリならそのディレクトリ名。
fn display_name(root: &Path, repo: &Path) -> String {
    let relative = repo.strip_prefix(root).unwrap_or(repo);
    if !relative.as_os_str().is_empty() {
        return relative.display().to_string();
    }
    repo.canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| ".".to_string())
}
