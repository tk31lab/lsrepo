mod git;
mod scan;
mod table;
#[cfg(test)]
mod testutil;

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use table::Row;

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
            head: git::head(repo),
            uncommitted: git::uncommitted_count(repo),
        })
        .collect();

    if rows.is_empty() {
        eprintln!("lsrepo: Git リポジトリが見つかりませんでした");
        return ExitCode::SUCCESS;
    }

    for line in table::render(&rows, use_color()) {
        println!("{line}");
    }
    ExitCode::SUCCESS
}

/// 端末に出力していて、NO_COLOR (https://no-color.org) が設定されていなければ色を付ける
fn use_color() -> bool {
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    !no_color && std::io::stdout().is_terminal()
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
