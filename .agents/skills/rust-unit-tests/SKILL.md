---
name: rust-unit-tests
description: 在 warp Rust 代码库中编写、改进并运行 Rust 单元测试。
---

# warp 中的 Rust 单元测试

## 范围
- 本技能聚焦 crate 级单元测试。
- 优先编写增量且范围清晰的测试，每个 case 只覆盖一个函数或行为。

## 单元测试放置位置
- 将单元测试放在名为 `${filename}_tests.rs` 或 `mod_test.rs` 的独立文件中。
- 在对应 source file 的末尾引入 test module：

```rust
#[cfg(test)]
#[path = "filename_tests.rs"] // or "mod_test.rs"
mod tests;
```

## 编写高质量测试
- 使用描述性名称：`fn parses_utf8_sequence_when_valid()`。
- 优先使用 `assert_eq!`/`assert_ne!`，而不是 `assert!`，以获得更清晰的 diff。
- 仅当 panic 语义是预期 API 时，才使用 `#[should_panic]`。
- 尽量减少 global state；通过 traits/constructors 注入依赖，让逻辑无需重度 mocking 也可测试。
- 添加 enum 或扩展行为时，在被测代码中优先使用 exhaustive matches，并在 tests 中镜像对应 cases。
- 注意 terminal model locking：避免 tests 在同一 call stack 中获取多次 `model.lock()` 的模式。

## Async 与 feature-gated code
- 对 async logic，当代码需要 runtime 时使用 `#[tokio::test]`。
- 优先使用 runtime feature checks（例如 `FeatureFlag::X.is_enabled()`），而不是 `#[cfg(...)]`，这样 tests 切换行为时不需要重新编译。

## Quickstart harness（UI/model tests）
- 对 views/models 相关的确定性单元测试，优先使用 `warpui::App::test`。
- 初始化 app models 一次，然后通过 `update` 修改，并通过 `read` 断言。

```rust
use warpui::App;
// In app crate tests prefer `crate::test_util::...`; from other crates use `warp::test_util::...`.
use warp::test_util::{terminal::initialize_app_for_terminal_view, add_window_with_terminal};

#[test]
fn example() {
    App::test((), |mut app| async move {
        // One-time app setup for terminal/view tests
        initialize_app_for_terminal_view(&mut app); // includes settings init
        let term = add_window_with_terminal(&mut app, None);

        // Act
        term.update(&mut app, |view, _ctx| {
            view.model.lock().simulate_block("ls", "out");
        });

        // Assert
        term.read(&app, |view, _ctx| {
            assert!(view.model.lock().block_list().len() > 0);
        });
    })
}
```

## 常用 helpers
- Terminal model shortcuts：`TerminalModel::mock(..)`、`.simulate_block(..)`、`.finish_block()`、`.simulate_cmd(..)`。
- 用于 focused tests 的 builders：`terminal::model::test_utils::{TestBlockListBuilder, TestBlockBuilder}`。
- 用于 IO-heavy code 的 virtual filesystem：
```rust
use virtual_fs::{VirtualFS, Stub};
VirtualFS::test("case", |_dirs, mut fs| {
    fs.with_files(vec![Stub::FileWithContent("path/file.txt", "contents")]);
    // run logic and assert
});
```
- Feature flags（scoped）：
```rust
use warp::features::FeatureFlag; // or `use crate::features::FeatureFlag;` inside the app crate
let _flag = FeatureFlag::CreatingSharedSessions.override_enabled(true);
```
- UI numeric assertions（lines）：
```rust
assert_lines_approx_eq!(actual_lines, INLINE_BANNER_HEIGHT);
```
- Concurrency：保持 `model.lock()` scopes 尽可能小；避免在同一 call chain 中嵌套或 re-entrant locks。
- 使用 `initialize_app_for_terminal_view` 时，不要直接调用 `initialize_settings_for_tests`（它已经会调用）。
- Async needs：需要真实 runtime 时使用 `#[tokio::test]`；否则优先使用 `App::test`。
- 触及 global/external state 的 tests：考虑使用 `serial_test` 的 `#[serial]` 或 local mocking，而不是 parallelism。

## 运行单元测试
- Workspace（parallel）：
```bash
cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2
```
- 单个 crate：
```bash
cargo nextest run -p <crate_name>
```
- 单个 test（按名称过滤）：
```bash
cargo nextest run -E 'test(<substring>)'
```
- Doc tests：
```bash
cargo test --doc
```

## Linting 与 formatting
提交变更前运行：
```bash
./script/format
cargo clippy --workspace --all-targets --all-features --tests -- -D warnings
```

PR 前如需完整本地检查，也可以运行：
```bash
./script/presubmit
```
