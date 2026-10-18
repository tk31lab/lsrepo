//! 一覧表の組み立てと色付け

use crate::git::Head;

pub struct Row {
    pub name: String,
    pub head: Option<Head>,
    pub uncommitted: Option<usize>,
}

const HEADER: [&str; 3] = ["NAME", "BRANCH", "CHANGES"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    Plain,
    Bold,
    Red,
    Green,
    Yellow,
    Magenta,
    Cyan,
}

impl Style {
    fn code(self) -> Option<&'static str> {
        match self {
            Style::Plain => None,
            Style::Bold => Some("1"),
            Style::Red => Some("31"),
            Style::Green => Some("32"),
            Style::Yellow => Some("33"),
            Style::Magenta => Some("35"),
            Style::Cyan => Some("36"),
        }
    }
}

struct Cell {
    text: String,
    style: Style,
}

fn cells(row: &Row) -> [Cell; 3] {
    let branch = match &row.head {
        Some(head @ Head::Branch(_)) => Cell {
            text: head.to_string(),
            style: Style::Cyan,
        },
        Some(head @ Head::Detached(_)) => Cell {
            text: head.to_string(),
            style: Style::Magenta,
        },
        None => Cell {
            text: "?".into(),
            style: Style::Red,
        },
    };
    let uncommitted = match row.uncommitted {
        Some(0) => Cell {
            text: "0".into(),
            style: Style::Green,
        },
        Some(n) => Cell {
            text: n.to_string(),
            style: Style::Yellow,
        },
        None => Cell {
            text: "?".into(),
            style: Style::Red,
        },
    };
    [
        Cell {
            text: row.name.clone(),
            style: Style::Plain,
        },
        branch,
        uncommitted,
    ]
}

/// 見出し行 + 各行を、列幅をそろえた文字列にする。`color` が true なら ANSI カラーを付ける。
pub fn render(rows: &[Row], color: bool) -> Vec<String> {
    let header = HEADER.map(|text| Cell {
        text: text.into(),
        style: Style::Bold,
    });
    let lines: Vec<[Cell; 3]> = std::iter::once(header)
        .chain(rows.iter().map(cells))
        .collect();

    let mut widths = [0; 3];
    for line in &lines {
        for (width, cell) in widths.iter_mut().zip(line) {
            *width = (*width).max(cell.text.chars().count());
        }
    }

    lines
        .iter()
        .map(|line| {
            let [name, branch, count] = line;
            // 色のエスケープシーケンスが幅計算に混ざらないよう、パディングしてから色を付ける
            let name = paint(
                format!("{:<w$}", name.text, w = widths[0]),
                name.style,
                color,
            );
            let branch = paint(
                format!("{:<w$}", branch.text, w = widths[1]),
                branch.style,
                color,
            );
            let count = paint(
                format!("{:>w$}", count.text, w = widths[2]),
                count.style,
                color,
            );
            format!("{name}  {branch}  {count}")
        })
        .collect()
}

fn paint(text: String, style: Style, color: bool) -> String {
    match style.code() {
        Some(code) if color => format!("\x1b[{code}m{text}\x1b[0m"),
        _ => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, head: Option<Head>, uncommitted: Option<usize>) -> Row {
        Row {
            name: name.into(),
            head,
            uncommitted,
        }
    }

    fn sample() -> Vec<Row> {
        vec![
            row(
                "proj/app",
                Some(Head::Branch("feature/long".into())),
                Some(12),
            ),
            row("w", Some(Head::Detached("9f177c9".into())), None),
            row("x", None, Some(0)),
        ]
    }

    #[test]
    fn render_aligns_with_header() {
        assert_eq!(
            render(&sample(), false),
            [
                "NAME      BRANCH              CHANGES",
                "proj/app  feature/long             12",
                "w         (detached 9f177c9)        ?",
                "x         ?                         0",
            ]
        );
    }

    #[test]
    fn header_wider_than_cells() {
        let rows = [row("a", Some(Head::Branch("b".into())), Some(0))];
        assert_eq!(
            render(&rows, false),
            ["NAME  BRANCH  CHANGES", "a     b             0"]
        );
    }

    #[test]
    fn colors_wrap_padded_cells() {
        let lines = render(&sample(), true);
        assert_eq!(
            lines[0],
            "\x1b[1mNAME    \x1b[0m  \x1b[1mBRANCH            \x1b[0m  \x1b[1mCHANGES\x1b[0m"
        );
        assert_eq!(
            lines[1],
            "proj/app  \x1b[36mfeature/long      \x1b[0m  \x1b[33m     12\x1b[0m"
        );
        assert_eq!(
            lines[2],
            "w         \x1b[35m(detached 9f177c9)\x1b[0m  \x1b[31m      ?\x1b[0m"
        );
        assert_eq!(
            lines[3],
            "x         \x1b[31m?                 \x1b[0m  \x1b[32m      0\x1b[0m"
        );
    }
}
