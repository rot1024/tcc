# tcc

[TaskChute Cloud](https://taskchute.cloud) utility command line tool

## Install

```
curl -L https://github.com/rot1024/tcc/releases/download/v0.1.0/tcc-`uname -s`-`uname -m` > /usr/local/bin/tcc && chmod +x /usr/local/bin/tcc
```

## tcc project

TaskChute Cloud で出力したCSVデータ（実際にはTSV）を読み込んで、プロジェクトのIDと名前の一覧を出力します。
プロジェクトIDは、後述の `analyze` コマンドで用います。

```sh
tcc project taskchute.tsv
```

## tcc analyze

TaskChute Cloud で出力したCSVデータ（実際にはTSV）を使って、レポートを出力します。

レポートには、以下の内容が出力されます。

- プロジェクトの全タスク工程表
- 合計所要時間
- タスクの見積もり時間と所要時間のギャップ

```sh
# プロジェクトID指定してマークダウンでレポートを出力
tcc analyze --project 100 --format md taskchute.csv > taskchute.md
```
