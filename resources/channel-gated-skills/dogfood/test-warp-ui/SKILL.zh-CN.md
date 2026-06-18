---
name: test-warp-ui
description: 使用 computer use 工具测试 Warp UI 功能和变更。仅当 agent 可使用 computer_use 工具时使用，覆盖启动 Warp 并验证 UI 行为。
user-invocable: false
---

# 使用 Computer Use 测试 Warp UI

使用 `computer_use` 工具可视化测试 UI 变更后 Warp 的外观和行为是否符合预期。

## 运行 Warp

从仓库根目录启动 Warp：

```bash
cargo run -- --api-key $STAGING_USER_WARP_API_KEY
```

`--api-key` flag 会使用 `STAGING_USER_WARP_API_KEY` 环境变量中的 API key 认证，因此 app 会直接启动，不需要交互式登录。

首次构建可能需要几分钟；后续增量构建会更快。

## 测试流程

### 1. 必要时硬编码或 mock 数据

如果只需要验证某个 UI 外观，可以临时硬编码或 mock 数据，让目标 UI 状态无需完整导航流程即可到达。此步骤是可选的；测试应自然工作的端到端流程时跳过。

适合硬编码的例子：

- **条件 UI**：功能只在特定条件下出现，例如某个设置、非空数据集、活跃订阅。可硬编码条件让 UI 始终出现。
- **Feature flags**：功能在尚未启用的 flag 后面。可直接启用。
- **错误状态**：需要测试错误处理 UI。可硬编码错误响应或失败条件。

mock 改动必须最小且聚焦，只改到达待测 UI 状态所必需的内容。

### 2. 调用 Computer Use

调用 `computer_use` 工具，并在任务描述中包含：

- 构建并启动 Warp 的命令，通常是在仓库根目录运行 `cargo run -- --api-key $STAGING_USER_WARP_API_KEY`
- 导航到待测 UI 的逐步说明
- **需要报告的具体观察项**：明确要求工具描述哪些元素、文本、颜色、布局或状态
- **不要**在任务中写预期值。工具应报告它看到的内容，而不是判断正确性

### 3. 验证结果

将 `computer_use` 返回的观察结果与你的预期对比。如果 UI 不符合预期，继续调查并调整代码或 mock。

## 提示

- **任务描述要具体**：不要说“检查对话框是否正确”，而是说“打开 Settings，点击 General tab，并描述第一个 section 的文本和布局”。
- **一次只测试一件事**：聚焦测试更容易在观察结果不符时调试。
- **调用前先构建**：调用 `computer_use` 前必须确认构建成功。该工具不能修复构建错误。
