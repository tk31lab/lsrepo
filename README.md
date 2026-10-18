# lsrepo

カレントディレクトリ(または引数で指定したディレクトリ)以下の Git リポジトリを探し、
「名前 / ブランチ / 未コミット数」を一覧表示する CLI。

```
$ lsrepo ~/src
proj/app  feature/long-branch-name    2
work      (detached 9f177c9)          0
```

- 名前: 探索ルートからの相対パス(ルート自体がリポジトリならそのディレクトリ名)
- ブランチ: detached HEAD のときは `(detached <短縮ハッシュ>)`
- 未コミット数: `git status --porcelain` の行数(untracked を含む)
- 取得に失敗した値は `?`
- `.git` の中、`node_modules`、`target` には潜らない。シンボリックリンクはたどらない

現在は `git` コマンドに依存している(`src/git.rs`)。将来は gix に差し替えて
スタンドアローンにする予定。
