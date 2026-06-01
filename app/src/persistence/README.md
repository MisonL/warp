# 如何执行 migration

## Migration 如何工作？

sqlite 数据库是用户计算机上的单个文件。它不是一个单独进程，而是一组打包在我们应用中的 C 函数。Warp 应用启动时，我们会在一个 transaction 中将 schema 升级到最新版本。
由于我们无法控制用户机器，因此必须对发布的 migration 极其谨慎。

TODO：C.I. 和失败 migration 的修复处理

## 第 1 步：一次性设置

请确保之前至少运行过一次 `/script/bootstrap`。它会安装我们 fork 的 `diesel_cli`。我们的 fork 是一个旧版本，会将 SQLite [捆绑](https://github.com/warpdotdev/diesel/blob/b2c58897c39c519a946314bd5b63765d3af56204/diesel_cli/Cargo.toml#L54)进 `diesel_cli`。我们使用这些版本的 Diesel 和 SQLite，而不是依赖机器上的版本。因此不要遵循官方 Diesel CLI 安装说明。

此外，`sqlite3` binary 在开发中很有用。Macbook 自带版本也可以，但很可能缺少一些基本语言特性。你可以从官方网站 https://www.sqlite.org/download.html 获取最新 binary。注意，我们使用了一些旧版 sqlite 不可用的 SQL 语言特性。还要注意，你需要在 Mac 系统偏好设置中批准它运行。

## 第 2 步：编写 migration

```
diesel migration generate <descriptive name of your migration>
```

这会创建一个包含 up.sql 和 down.sql 的新文件夹。

## 第 3 步：运行 migration 并生成 schema

```
cd <repo root>
diesel migration run --database-url="/Users/$USER/Library/Application Support/dev.warp.Warp-Local/warp.sqlite"
```

这会在你本地运行应用时使用的同一个 Warp 数据库上运行 migration。该命令会自动生成或更新 `crates/persistence/src/schema.rs`。我们不手动编辑 `schema.rs`。

你也可以从已经包含该 migration 的数据库打印 schema：

```
diesel print-schema --database-url="/Users/$USER/Library/Application Support/dev.warp.Warp-Local/warp.sqlite"
```

## 回退/重做 migration

在编写功能和切换分支时，你会需要撤销 migration 来修复数据库，并让它与旧代码兼容。迭代 schema 时，redo 也可能很有帮助。

```
diesel migration revert --database-url="/Users/$USER/Library/Application Support/dev.warp.Warp-Local/warp.sqlite"
diesel migration redo --database-url="/Users/$USER/Library/Application Support/dev.warp.Warp-Local/warp.sqlite"
```

# Schema 风格

- 对整数主键使用 `id`。如果主键有更特殊的含义，请考虑使用更具描述性的名称。
- 表名使用复数；Rust model 代码中的 struct 使用单数。
- 如果有一张 `foos` 表和一张 `bars` 表，而 `bars` 中有一个引用 `foos.id` 的外键列，请将其命名为 `foo_id`。

# `schema.patch` 文件

`crates/persistence/schema.patch` 由我们手动更新。该文件允许我们在 `diesel_cli` 生成的 `schema.rs` 文件之上进行手动变更。

创建 `schema.patch` 文件时，我们会：
1. 运行 diesel migration
1. 手动编辑 `schema.rs`
1. 运行 `git diff -U6 > crates/persistence/schema.patch`。

关于该 patch 文件的更多信息，可阅读官方 [Diesel 文档](https://diesel.rs/guides/configuring-diesel-cli.html#the-patch_file-field)。
