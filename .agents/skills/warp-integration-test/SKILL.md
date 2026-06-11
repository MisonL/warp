---
name: warp-integration-test
description: 使用 `crates/integration` 中的自定义 Builder/TestStep framework 编写、运行并调试 Warp integration tests。添加新的 integration test、修复失败的 integration test、将测试接入 manual runner 或 nextest suite，或验证 Warp 中端到端 UI 与 terminal 行为时使用。
---

# Warp 集成测试

将本技能用于 `crates/integration/` 下 Warp 自定义 framework 中的 Rust integration tests。

这些不是普通 unit tests。它们会启动真实 Warp app instance，分配隔离的 test home directory，用 synthetic UI 和 terminal events 驱动它，并轮询 assertions，直到成功或超时。

## Framework 映射

核心组件如下：

- `crates/integration/src/bin/integration.rs`
  - 手动 integration test runner binary。
  - 将 test names 注册到 `Builder` factories。
  - 每次 invocation 只运行一个具名 test。
- `crates/integration/tests/common/mod.rs`
  - `cargo test` 和 `cargo nextest` 使用的外层 Rust test harness。
  - shell out 到 integration binary。
  - 转发有限的一组 env vars（`PATH`、`RUST_*`、`WARP_*`、`WARPUI_*`、`WGPU_*`、display-related vars）。
  - 当 integration binary 以特殊 rerun code 退出时，最多重新运行 tests 10 次。
- `crates/integration/src/test.rs`
  - integration tests 的 module hub。
  - 在这里添加新的 test modules，并 `pub use` 它们的 functions，让 runner 可以看到。
- `crates/integration/tests/integration/ui_tests.rs`
  - nextest 应运行的 UI-oriented integration tests 列表。
- `crates/integration/tests/integration/shell_integration_tests.rs`
  - 必须针对每个 shell 或特定 shell matrix 运行的 tests 列表。
- `crates/integration/src/builder.rs`
  - lower-level WarpUI integration builder 外层的 Warp-specific wrapper。
  - 按需设置 default timeout、hermetic home directory、shell rc files、user prefs 和 real-display mode。
- `crates/warpui_core/src/integration/driver.rs`
  - 执行 steps，处理 retries、precondition reruns、screenshots、video capture、artifact export 和 `on_finish`。
- `crates/warpui_core/src/integration/step.rs`
  - 定义 `TestStep`、input/event APIs、assertion polling、step-to-step data passing 和 screenshot/recording hooks。
- `app/src/integration_testing/`
  - 常见 Warp behaviors 的 high-level helpers 和 assertions。
  - 只要适用，就优先使用这些 helpers，而不是 raw low-level event plumbing。

## Framework 实际如何运行 test

1. `crates/integration/tests/integration/*.rs` 中的 Rust test 调用 `run_integration_test("test_name")`。
2. 该 harness 用 test name 启动 `integration` binary。
3. `crates/integration/src/bin/integration.rs` 中的 binary 在 `register_tests()` 中查找名称，构建 `Builder`，并将其转换为 `TestDriver`。
4. `Builder::build(...)` 创建隔离的 temp directory，将 `HOME` 指向它，写入最小 rc files，并初始化 file-backed user preferences。
5. driver 按顺序运行每个 `TestStep`：
   - setup callbacks
   - synthetic events
   - actions
   - assertion polling until success or timeout
6. 如果 assertion 返回 `PreconditionFailed`，binary 会以 rerun code 退出，外层 harness 会重试整个 test。
7. 在成功、失败或取消时，driver 可以运行 `on_finish` 并导出 artifacts/runtime tags。

这意味着 integration tests 应面向 hermetic environment 编写。不要依赖开发者真实的 shell dotfiles、home directory contents 或 persisted Warp settings。

## 新 test 放在哪里

将实际 test function 添加到 `crates/integration/src/test/` 下的某个 module 中。

使用这些启发式规则：

- 当 test 匹配某个已有 feature area 时，将它放入现有 module。
- 当 feature 无法自然放入现有 module 时，创建新 module。
- 如果它主要是 UI/app behavior test，将 test 添加到 `crates/integration/tests/integration/ui_tests.rs`。
- 如果它需要针对每个 shell 运行，或依赖特定 shell/set of shells，将 test 添加到 `crates/integration/tests/integration/shell_integration_tests.rs`。

只出现在 `crates/integration/src/test/*.rs` 中还不够。要让 test 在 `cargo nextest` 下运行，还需要将它列入 `crates/integration/tests/integration/` 中的某个 macro file。

## 新 test 编写清单

添加新的 integration test 时，完成以下所有事项：

1. 在 `crates/integration/src/test/` 下的 module 中实现 `pub fn test_name() -> Builder`。
2. 将 module 添加到 `crates/integration/src/test.rs`。
3. 从 `crates/integration/src/test.rs` 中 `pub use` 新 module 的 exports。
4. 在 `crates/integration/src/bin/integration.rs` 中添加 `register_test!(test_name);`。
5. 将 `test_name` 添加到以下位置之一：
   - `crates/integration/tests/integration/ui_tests.rs`，或
   - `crates/integration/tests/integration/shell_integration_tests.rs`
6. 一旦 test 添加到这些 macro lists 之一，默认让它在 CI 中运行。只有当任务明确要求 manual-only coverage，或存在具体且已记录的原因说明它无法在 CI 中可靠运行时，才标记为 `#[ignore]`。
7. 先手动运行 test；当它对所选 suite 足够稳定后，再通过 nextest 运行。

## 编写 test 正文

常规形态如下：

```rust
use crate::Builder;
use warp::integration_testing::step::new_step_with_default_assertions;
use warp::integration_testing::terminal::{
    clear_blocklist_to_remove_bootstrapped_blocks,
    execute_command_for_single_terminal_in_tab,
    wait_until_bootstrapped_single_pane_for_tab,
    util::ExpectedExitStatus,
};

pub fn test_example() -> Builder {
    Builder::new()
        .with_step(wait_until_bootstrapped_single_pane_for_tab(0))
        .with_step(clear_blocklist_to_remove_bootstrapped_blocks())
        .with_step(execute_command_for_single_terminal_in_tab(
            0,
            "echo hello".to_string(),
            ExpectedExitStatus::Success,
            "hello".to_string(),
        ))
        .with_step(
            new_step_with_default_assertions("Assert some UI state")
                .add_named_assertion("specific assertion name", |app, window_id| {
                    // inspect app state and return AssertionOutcome
                    warpui::integration::AssertionOutcome::Success
                }),
        )
}
```

优先使用少量具备描述性名称的 focused steps，而不是一个巨大的 monolithic test。

## Builder 指南

### `Builder::new()`

几乎每次都从这里开始。

Warp 的 wrapper 会自动提供：

- 每个 test 独立的 root directory
- 隔离的 `HOME`
- 为 Bash、Zsh 和 Fish 生成的 rc files
- file-backed user preferences
- 默认 2 分钟 hard timeout
- 如果存在 `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS`，则启用 real-display support

### `with_setup(...)`

在 app 运行前需要 filesystem 或 environment setup 时使用它。

常见模式：

- `utils.set_env("NAME", Some(value))`
- 在 `utils.test_dir()` 下创建 files
- 写入 fixture config files

优先使用这种方式，而不是触碰真实 filesystem。

### `with_user_defaults(...)`

用它在 test 启动前设置 persisted Warp preferences。

对于由 user preferences 而不是 environment variables 支撑的 settings，这是正确工具。

### `set_should_run_test(...)`

当 test 确实无法在所有环境运行时，用它基于 shell/platform/runtime capabilities 对 tests 做 gate。

### `with_on_finish(...)`

用它执行所有 steps 完成后才应进行的 final verification 或 artifact inspection，例如检查 screenshots 或 recordings 是否已写入。

### `with_real_display()`

当 test 需要 real display 执行 frame capture 或 visual workflows 时，显式使用它。除非存在稳定的 real-display path，否则 video/screenshot tests 通常应为 manual 或在 CI 中 ignored。

## `TestStep` 指南

`TestStep` 是执行单元。每个 step 可以包含：

- setup callbacks
- input events
- actions
- assertions
- a timeout
- retry count
- failure handling

### 从 helper constructors 开始

优先使用：

- `wait_until_bootstrapped_single_pane_for_tab(0)`
- `new_step_with_default_assertions("...")`
- `new_step_with_default_assertions_for_pane("...", tab, pane)`

默认 step helpers 已经会 assert：

- 没有 pending model events
- 没有 block executing

这些是大多数 UI interactions 的良好 baseline invariants。

### 优先使用 helper APIs，而不是 raw event plumbing

只要可能，就使用 `app/src/integration_testing/` 中的 high-level helpers：

- terminal command execution helpers
- block list helpers
- command palette helpers
- navigation helpers
- settings helpers
- workflow/file tree/notebook helpers

只有在没有合适 helper 时，才降级使用 raw `with_event(...)`、`with_event_fn(...)` 或 saved-position mouse events。

### 使用 named assertions

优先使用 `add_named_assertion(...)`，而不是 unnamed assertions。Named assertions 会让 failure output 和 runtime tags 更容易解读。

### 使用 polling assertions，而不是 sleeps

Assertions 会被轮询，直到成功或超时。应利用这个模型，而不是硬编码 sleeps。

良好模式：

- 触发 event 或 action
- 对最终 UI/model state 做 assert

避免脆弱的 timing assumptions。

### 当一个 step 为下一个 step 计算内容时，使用 step data

如果后续 step 需要来自前一个 step 的 data，使用：

- `add_named_assertion_with_data_from_prior_step(...)`
- `StepDataMap`

这对于保存 prior frames 中测量得到的 positions、counts、IDs 或其他 values 很有用。

### 谨慎使用 retries

`set_retries(...)` 可以帮助处理确实可 retry 的 step，但不要用它隐藏 deterministic failures。优先先让 step 更 robust。

### 对已知 environmental flakes 使用 `PreconditionFailed`

如果 environment 到达某种使 test 剩余部分无效的状态，返回 `AssertionOutcome::PreconditionFailed(...)`，而不是直接 hard fail。外层 harness 可以最多重新运行整个 test 10 次。

现有 bootstrap helper 是这方面的良好范例。

## 常见 test 编写模式

### 1. 先等待 bootstrap

对大多数 terminal-facing tests，第一个真实 step 应该是：

- `wait_until_bootstrapped_single_pane_for_tab(0)`

不要在 bootstrap 完成前开始 assert terminal UI。

### 2. 如果 block indices 很重要，清理 bootstrapped blocks

如果 test 依赖 `block_index:0` 这类 saved positions，在 bootstrap 后清理 block list：

- `clear_blocklist_to_remove_bootstrapped_blocks()`

否则，第一个 user-generated block index 会依赖 bootstrap output 和 active shell。

### 3. 使用 helper command runners

优先使用如下 helpers：

- `execute_command_for_single_terminal_in_tab(...)`
- `execute_echo(...)`
- `execute_echo_str(...)`
- `execute_long_running_command(...)`

这些 helpers 已经处理了大量 correctness 和 output validation。

### 4. 断言 visible behavior，而不只是 internal mutation

高质量 integration test 会验证 user-observable behavior：

- terminal 中可见 output
- focus 移动到预期位置
- UI element 打开/关闭
- selection changed
- settings applied

Internal state assertions 仍然有用，但应支撑 visible behavior，而不是替代它。

### 5. 保持 tests feature-focused

为一个 behavior 或一个紧密相关的 flow 编写一个 test。如果需要覆盖多个 scenarios，考虑使用多个 tests，而不是一个巨大的 script。

## 运行 tests

### 直接通过 integration binary 运行单个 test

编写时先使用这个命令：

```bash
cargo run -p integration --bin integration -- test_name
```

这是迭代特定 test 的最快方式，因为它绕过外层 Rust test wrapper，直接运行具名 test。

### 通过 nextest 运行单个 test

一旦它被接入 `tests/integration/*.rs` macro lists 之一，就用 nextest 运行它：

```bash
cargo nextest run --no-fail-fast --workspace test_name
```

### 需要时使用 real display 运行

对于 screenshot/video 或其他 real-display flows：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 cargo run -p integration --bin integration -- test_name
```

或使用 nextest：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 cargo nextest run --no-fail-fast --workspace test_name
```

## 调试与调查

### 失败时获取 backtrace

```bash
RUST_BACKTRACE=1 cargo run -p integration --bin integration -- test_name
```

### 失败时暂停

本地运行且想检查失败的 UI state 时，这很有用：

```bash
WARPUI_PAUSE_INTEGRATION_TEST_ON_FAILURE=1 cargo run -p integration --bin integration -- test_name
```

### 每个 step 后暂停

这有助于准确理解 test 正在做什么：

```bash
WARPUI_PAUSE_INTEGRATION_TEST_AT_EVERY_STEP=1 cargo run -p integration --bin integration -- test_name
```

### Video 与 screenshots

如果任务专门涉及 recording a test、collecting screenshots 或 validating overlay/video artifacts，也使用 `integration-test-video` 技能（位于 `.warp/skills/integration-test-video/SKILL.md`）。

### Environment variable 注意事项

`utils.set_env(...)` 会影响 `std::env::var(...)` 这类 runtime environment lookups。

它不会影响 `option_env!(...)` 这类 compile-time lookups。如果 product code 使用 `option_env!`，在 test 内修改 env var 不会在未重新构建的情况下改变该行为。

## Verification checklist

在认为新的 integration test 完成之前，验证以下所有事项：

- test function 位于 `crates/integration/src/test/` 下。
- module 已添加到 `crates/integration/src/test.rs` 并重新导出。
- test 已在 `crates/integration/src/bin/integration.rs` 中注册。
- test 已列入正确的 nextest macro file，并默认会在 CI 中运行，除非它因已记录原因被明确设为 manual-only。
- 直接通过 integration binary 运行时，test 通过。
- 如果它应属于 automated suite，通过 nextest 运行时，test 通过。
- assertions 检查预期 user-visible behavior。
- test 不依赖开发者真实 home directory、shell config 或 machine state。
- 如果 test 使用 screenshots/video，实际检查了生成的 artifacts，而不是只假设它们存在。

## 要避免的 anti-patterns

- 只在 `src/test/*.rs` 中编写 test，却忘记 nextest macro list。
- 未先清理 bootstrapped blocks，就 assert bootstrap-sensitive block indices。
- 已有 helper 时仍到处使用 raw events。
- 添加 sleeps，而不是使用 assertion polling。
- 让 test 依赖个人 dotfiles、真实 settings 或 non-hermetic filesystem state。
- 使用 retries 掩盖 deterministic bug。
- 在没有稳定路径的情况下，让 real-display/manual test 在 CI 中启用。

## Agents 的良好 workflow

当被要求添加或修复 integration test 时：

1. 找到该 feature 最接近的现有 integration test module。
2. 在发明新的 low-level plumbing 前，先复用 helper assertions 和 step constructors。
3. 在所有必要位置注册 test，而不只是 implementation file。
4. 先手动运行 test。
5. 如果它属于 automation，也用 nextest 运行。
6. 如果 test 覆盖 visual behavior，直接验证生成的 UI behavior 或 artifacts。
