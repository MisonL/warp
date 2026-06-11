---
name: integration-test-video
description: 运行带截图和视频捕获的 Warp 集成测试，包括鼠标和键盘输入的事件 overlay 标注。当用户想录制集成测试、从测试收集截图、审查生成的录制 artifact，或编写用于调试/演示的视频捕获测试时使用此技能。
---

# 集成测试视频录制

在本分支上处理 Warp 集成测试录制流水线时使用此 skill。

相关实现位于：
- `integration/src/bin/integration.rs`
- `integration/src/test/video_recording.rs`
- `integration/tests/integration/ui_tests.rs`
- `ui/src/integration/driver.rs`
- `ui/src/integration/step.rs`
- `ui/src/integration/video_recorder.rs`
- `ui/src/integration/artifacts.rs`
- `ui/src/integration/overlay.rs`

## 调用测试的命令

对单个手动调用的录制测试，优先使用 integration binary：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
cargo run -p integration --bin integration -- test_video_recording
```

这是 `integration/src/test/video_recording.rs` 中 sample test 展示的命令。

如果希望 driver 自动录制某个测试或一组测试，请添加 `WARP_INTEGRATION_TEST_VIDEO`：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_VIDEO=test_video_recording \
cargo run -p integration --bin integration -- test_video_recording
```

对更广泛的集成测试运行，同样的环境变量也可用于普通 test runner：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_VIDEO=test_foo,test_bar \
cargo nextest run --no-fail-fast --workspace test_foo
```

## 环境变量

### `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS`

- 需要真实 frame capture 时，将它设置为 `1`。
- 用于 screenshot/video 工作流和手动视觉验证。
- 没有真实 display 时，录制工作流预计会不完整或不可用。

### `WARP_INTEGRATION_TEST_VIDEO`

这是控制 `ui/src/integration/driver.rs` 中 driver-managed video recording 的主要环境变量。

行为：
- 未设置或为空：禁用自动录制。
- `1` 或 `all`：自动录制本次运行中的每个测试。
- 逗号分隔的测试名：只自动录制这些测试。

示例：

```bash
# 录制本次运行中的每个测试
WARP_INTEGRATION_TEST_VIDEO=all
```

```bash
# 只录制特定测试
WARP_INTEGRATION_TEST_VIDEO=test_foo,test_bar
```

重要细节：
- 如果测试本身显式调用 `with_start_recording()` 和 `with_stop_recording()`，则不需要 `WARP_INTEGRATION_TEST_VIDEO`。
- 当你希望在不修改测试代码的情况下录制整个测试时，使用该环境变量。

### `WARP_INTEGRATION_TEST_ARTIFACTS_DIR`

它控制 `ui/src/integration/artifacts.rs` 中 `TestArtifacts` 使用的根 artifact 目录。

如果未设置，artifact 会写入：

```text
$TMPDIR/warp_integration_test_artifacts
```

每次运行都会获得一个带时间戳的目录：

```text
<artifacts_root>/<test_name>/<timestamp>/
```

这是检查截图、日志和最终 `recording.mp4` 的主要目录。

### `WARP_INTEGRATION_TEST_VIDEO_DIR`

该环境变量存在于 `ui/src/integration/video_recorder.rs` 中，是较低层 recorder output root helper，默认值为：

```text
$TMPDIR/warp_integration_video_captures
```

在本分支上，普通 integration driver 流程会改为将最终视频写入 test artifacts 目录，因此审查结果时通常更关心 `WARP_INTEGRATION_TEST_ARTIFACTS_DIR`。

## 如何指定要录制哪些测试

有两种模式：

### 1. 在测试代码中录制

在测试内部使用 `TestStep::with_start_recording()` 和 `TestStep::with_stop_recording()`。当你只想捕获测试中的特定片段时，这是最佳方式。

### 2. 从环境变量录制

将 `WARP_INTEGRATION_TEST_VIDEO` 设置为：
- `all`
- `1`
- 或类似 `test_a,test_b` 的逗号分隔列表

这会在匹配测试开始时开始录制，并在测试完成时写出视频。

## Overlay 如何工作

本分支没有单独的 overlay 环境变量。

Overlay annotation 由录制处于 active 状态时测试派发的 input event 产生。Overlay pipeline 实现在 `ui/src/integration/overlay.rs` 中，event capture hook 位于 `ui/src/integration/step.rs`。

要在最终视频中获得有用的 overlay，请使用会发出鼠标和键盘事件的 API 驱动测试，例如：
- `with_event(...)`
- `with_event_fn(...)`
- `with_click_on_saved_position(...)`
- `with_keystrokes(...)`

Sample test 当前覆盖的 overlay 类型：
- 鼠标点击指示器
- 拖拽轨迹
- 键盘快捷键 pill

实践中：
- Mouse down / drag / mouse up event 会创建点击和拖拽 overlay。
- KeyDown event 会创建键盘 overlay pill。
- 如果测试只录制 frame，从未派发相关 input event，生成的视频不会显示这些 annotation。

## 如何编写截图测试

使用 `TestStep::with_take_screenshot("filename.png")`。

示例模式：

```rust
TestStep::new("Take screenshot after bootstrap")
    .with_take_screenshot("after_bootstrap.png")
```

截图请求会在 step 期间存储，并由 driver 在该 step 渲染后写出。PNG 会落在该测试带时间戳的 artifacts 目录中。

## 如何编写视频录制测试

### 最小模式

1. 使用 `Builder::new().with_real_display()`。
2. 添加一个带 `with_start_recording()` 的 step。
3. 运行你想捕获的 action/event。
4. 添加一个带 `with_stop_recording()` 的 step。

示例形态：

```rust
Builder::new()
    .with_real_display()
    .with_step(TestStep::new("Start recording").with_start_recording())
    .with_step(/* actions and events */)
    .with_step(TestStep::new("Stop recording").with_stop_recording())
```

### 对 overlay 友好的录制

优先使用会发出鼠标和 key event 的显式 UI-driving step：
- 使用 `with_click_on_saved_position(...)` 点击
- 使用 `with_event(...)` / `with_event_fn(...)` 派发原始鼠标事件
- 使用 `with_keystrokes(...)` 发送键盘快捷键

对拖拽 overlay，发送如下 sequence：
- `LeftMouseDown`
- 一个或多个 `LeftMouseDragged`
- `LeftMouseUp`

### 可选验证

添加 `with_on_finish(...)` hook 检查预期 artifact 是合理的，例如：
- `recording.mp4`
- `recording.log`
- screenshot PNG

Sample test 正是这样做的。

## 视频 asset 写入位置

普通输出位置是：

```text
${WARP_INTEGRATION_TEST_ARTIFACTS_DIR:-$TMPDIR/warp_integration_test_artifacts}/<test_name>/<timestamp>/
```

该目录中的常见 artifact：
- `recording.mp4`
- `recording.log`
- 任何通过 `with_take_screenshot(...)` 请求的截图

对 `test_video_recording`，sample test 预期：
- `after_bootstrap.png`
- `after_commands.png`
- `recording.mp4`
- `recording.log`

如果 MP4 encoding 在 finalization 期间失败，recorder 会 fallback 到 sibling directory 中的逐帧 PNG，例如：

```text
recording_frames/
```

文件名类似：

```text
recording_0000.png
```

## 如何审查 asset

1. 打开该测试最新的带时间戳 artifact 目录。
2. 先审查 `recording.mp4`，确认：
   - UI state 正确
   - 录制确实在预期 window 中开始和停止
   - overlay annotation 在正确时刻出现
3. 审查测试捕获的任何 PNG 截图。
4. 如果输出看起来不完整或可疑，检查 `recording.log`。
5. 如果缺少 `recording.mp4`，查找 fallback frame PNG。

向用户汇总结果时，请包含精确的 artifact 目录路径。

## 视频录制 sample test

Sample manual test 是 `test_video_recording`。

它：
- 在 `integration/src/bin/integration.rs` 中注册
- 列在 `integration/tests/integration/ui_tests.rs` 中
- 实现在 `integration/src/test/video_recording.rs` 中

运行方式：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
cargo run -p integration --bin integration -- test_video_recording
```

如果还想从环境变量启用全测试自动录制，请使用：

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_VIDEO=test_video_recording \
cargo run -p integration --bin integration -- test_video_recording
```

## Agent 工作模式

当被要求录制或调试带视频的集成测试时：

1. 识别精确测试名。
2. 决定录制应在测试中显式开启，还是通过 `WARP_INTEGRATION_TEST_VIDEO` 启用。
3. 确保运行时使用 `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1`。
4. 如果用户需要可见的交互 overlay，确保测试在录制 active 期间派发鼠标和键盘 event。
5. 运行后检查带时间戳的 artifact 目录，并将输出路径报告给用户。
