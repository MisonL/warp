# Warp 中的集成测试

这是一份关于在 Warp 中编写集成测试的简短指南。

## 何时添加新的集成测试？

我们对 unit test 与 integration test 的一般理念可总结如下：

### 以下情况编写 unit test：

* 测试单个函数；
* 函数依赖极少，且没有 pty 依赖；
* 可以纯粹在 Rust 中运行，例如 parser。

### 以下情况 integration test 会有帮助：

* 从用户视角测试某个 use case；
* 场景较慢或需要 shell。

## 什么是好的集成测试？

测试通常采用以下格式：
* 在应用中设置某些状态；
* 模拟用户操作（例如输入或点击）；
* 验证应用处于预期状态。

## 如何添加新的集成测试？

当前集成测试需要你处理 **3** 个文件：[integration/tests/integration.rs](integration.rs)、[integration/src/bin/integration.rs](../src/bin/integration.rs) 和 [integration/src/test.rs](../src/test.rs)。（目前大多数测试位于 `test.rs` 中，但你可能希望将自己的测试写到单独文件中。）

先为你的功能编写一个*新的集成测试*。为此，只需在 [integration/src/bin/integration.rs](../src/bin/integration.rs) 中添加一个新方法。它应接收 **0 个参数**，并**返回 `TestDriver`** object。你还应在同一文件的 `register_tests()` 方法中注册它，并随后（为了确保它会被执行）将它添加到 [integration/tests/integration.rs](integration.rs) 的 `integration_tests!` macro 中。按照约定，每个测试方法名都以 `test_` 前缀开头（不过请注意，它不需要像 Rust 中普通 unit test 那样带 `#[test]` annotation）。

完成第一步后，就可以使用集成测试框架了。下面用一个示例说明集成测试中可以做什么（更多解释见注释）：

```rs
fn test_simple_example() -> TestDriver {
    new_builder() // 初始化集成测试 builder
        // 每个测试可以有多个 step，复杂度可高可低，
        // 例如，你可以等待某个特定 action 发生，就像下面这一行。
        .with_step(wait_for_bootstrapping(0))
        .with_step(
            // 你也可以创建自己的 `TestStep`。
            TestStep::new("Run ls and verify block exists") // 每个 `TestStep` 都有名称
                // 后续可以指定发生什么。你可以验证输入了哪些字符：
                .with_keystrokes(&[
                    Keystroke::parse("l").unwrap(),
                    Keystroke::parse("s").unwrap(),
                    Keystroke::parse("enter").unwrap(),
                ])
                // 设置 timeout，超过后测试会失败：
                .set_timeout(Duration::from_secs(5))
                // 指定某些 assertion：
                .set_assertion(Box::new(|app, window_id, presenter| {
                    let presenter = presenter.expect("presenter should be set");
                    assert!(presenter.scene().is_some());
                    let views = app.views_of_type(window_id).unwrap();
                    let terminal_view: &ViewHandle<TerminalView> = views.get(0).unwrap();
                    terminal_view.read(app, |view, _ctx| {
                        let model = view.model.lock();
                        async_assert!(
                            !model.is_block_list_empty(),
                            "Block list should not be empty"
                        )
                    })
                })),
        )
        .build()
}
```

到目前为止，我发现 `with_keystrokes` 和 `with_input_string` 是最有帮助的方法。你可以在 [ui/src/integration/test_driver.rs](../../ui/src/integration/test_driver.ts) 中查看实现（并扩展它）。

## 何时使用 `assert!` 与 `async_assert!`

前者会在第一次为 false 时让测试失败。后者会在 timeout 内从未看到成功时让测试失败。如果没有指定 timeout，会使用默认 timeout。

在我们的 UI 框架中，dispatch event 和 action 通常是同步的。并发主要来自 event loop。

同步 assertion 示例：
如果第一次失败就会 panic。否则成功。

```rs
assert_eq!(
    view.buffer_text(ctx),
    "".to_string(),
    "Input should be empty"
);
AssertionOutcome::Success
```

异步 assertion 示例：

```rs
async_assert_eq!(
    expect_bootstrapped,
    bootstrapped,
    "terminal should be bootstrapped ({})",
    expect_bootstrapped
)
```

由于很多测试是 async，我建议在合并前本地循环运行以避免 flake，例如：

```sh
for i in {0..100}; do
    WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 RUST_BACKTRACE=full WARP_SHELL_PATH=/bin/bash cargo run -p integration -- test_simple_example
    if [ $? -ne 0 ]; then return; fi
done
```

这曾帮助我们捕获系统中很多已有 bug。

注意，为了让 `async_assert` 真正工作，`set_assertion` 需要通过 `async_assert` **return**。

## 如何添加 sqlite snapshot？

* 你可以直接从 ~/Library/Application\ Support/{warp, dev.warp.Warp-(Dev|Preview|Stable)} 复制 warp.sqlite 文件
* 你可能需要清理一些与你个人相关的信息（例如 cwd https://staging.warp.dev/block/FNBafyVtxvjmdNIx6HxUM5）

### 如何运行集成测试？

要运行特定集成测试，可以使用：

```
  WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS="1" cargo run --bin integration -- test_simple_example
```

`WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS="1"` 会强制打开新的 terminal window，这在迭代集成测试实现时很有帮助。

### 已知问题 / 限制

* 要从 `TestStep` 中判断测试使用哪个 shell，可以尝试检查 `WARP_SHELL_PATH` 环境变量（这在 GitHub CI 中有效），或检查用户 passwd（本地运行时）。
* 类似地，你可以设置 `WARP_SHELL_PATH` 后再运行测试，以使用特定 shell。注意，如果使用 fish，在该 feature flag 移除前，还需要传入 `--features fish_shell`。例如：`WARP_SHELL_PATH=/usr/local/bin/fish`，然后运行 `cargo run --bin integration --features fish_shell -- test_simple_example`
* Binding 默认不会在集成测试中暴露，请在原始 binding 所在文件中添加它们。来自 `editor/view.rs` 的示例：

```rust
if ChannelState::channel() == Channel::Integration {
        app.register_fixed_bindings([
            // Hack：为测试显式添加 binding，因为测试注入的
            // keypress 不会触发 Mac menu item。遗憾的是我们不能使用
            // cfg[test]，因为我们是一个单独进程。
            Binding::new(
                "cmd-z",
                EditorAction::Undo,
                Some("EditorView && !IMEOpen")
            ),
        ]);
    }
```
