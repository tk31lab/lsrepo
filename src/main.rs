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

    for line in render(&rows) {
        println!("{line}");
    }
    ExitCode::SUCCESS
}

const HEADER: [&str; 3] = ["NAME", "BRANCH", "CHANGES"];

/// 見出し行 + 各行を、列幅をそろえた文字列にする
fn render(rows: &[Row]) -> Vec<String> {
    let width = |header: &str, cell: fn(&Row) -> &str| {
        rows.iter()
            .map(|r| cell(r).chars().count())
            .chain([header.chars().count()])
            .max()
            .unwrap_or(0)
    };
    let name_width = width(HEADER[0], |r| &r.name);
    let branch_width = width(HEADER[1], |r| &r.branch);
    let count_width = width(HEADER[2], |r| &r.uncommitted);

    let format = |name: &str, branch: &str, count: &str| {
        format!("{name:<name_width$}  {branch:<branch_width$}  {count:>count_width$}")
    };
    std::iter::once(format(HEADER[0], HEADER[1], HEADER[2]))
        .chain(
            rows.iter()
                .map(|r| format(&r.name, &r.branch, &r.uncommitted)),
        )
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, branch: &str, uncommitted: &str) -> Row {
        Row {
            name: name.into(),
            branch: branch.into(),
            uncommitted: uncommitted.into(),
        }
    }

    #[test]
    fn render_aligns_with_header() {
        let rows = [
            row("proj/app", "feature/long", "12"),
            row("w", "(detached 9f177c9)", "?"),
        ];
        assert_eq!(
            render(&rows),
            [
                "NAME      BRANCH              CHANGES",
                "proj/app  feature/long             12",
                "w         (detached 9f177c9)        ?",
            ]
        );
    }

    #[test]
    fn header_wider_than_cells() {
        assert_eq!(
            render(&[row("a", "b", "0")]),
            ["NAME  BRANCH  CHANGES", "a     b             0"]
        );
    }
}
