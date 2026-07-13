---
name: verify-ui-change-in-cloud
description: 完成任何用户可见客户端变更后，在非沙盒或本地环境中自动调用此技能。它会启动带 Computer Use 的云端 Agent 来测试 Warp，并运行 test-warp-ui 技能。
user-invocable: true
---

# 在云端验证 UI 变更

完成用户可见的客户端变更后，使用此工作流启动带 Computer Use 能力的云端 Agent 来验证变更。适用于任何影响用户在运行中应用里看到或体验到的内容的改动，不仅包括视觉 UI，也包括启动行为、配置处理、迁移流程和其他客户端逻辑。

## 工作流

### 1. 推送改动

云端 Agent 会在新环境中 clone 仓库。必须先把改动 push 到一个分支，使云端 Agent 可以访问。

### 2. 检测仓库

启动云端 Agent 前，检测当前所在仓库。检查 Git 远程 URL：

```bash
git remote get-url origin
```

确认远程 URL 包含 `warpdotdev/warp`。如果不是，提醒用户此技能只支持 warp 仓库并停止。

Warp Dev Environment 的环境 ID 是 `SVhg783GBFQHk1OfdPfFU9`。

### 3. 启动云端 Agent

使用 `run_agents` 工具启动远程云端 Agent。单个子批次（`agent_run_configs` 只有一个条目）是有效的。

- `summary`：简短声明性说明，例如 `"启动带 Computer Use 的云端 Agent 来验证 UI 变更。"`
- `base_prompt`：要求读取并遵循 `test-warp-ui` 技能，然后写入验证任务
- `remote.environment_id`：`SVhg783GBFQHk1OfdPfFU9`
- `remote.computer_use_enabled`：`true`
- `agent_run_configs`：单个条目，`name` 使用短显示名，例如 `"verify-ui-change"`。单 Agent 的 `prompt` 可以为空，因为 `base_prompt` 已覆盖任务

`test-warp-ui` 技能是内置的，云端 Agent 会自动拥有它。在 `base_prompt` 中明确要求 Agent 调用该技能，例如“读取并遵循 test-warp-ui 技能。”

### 4. 编写有效提示词

提示词应告诉云端 Agent：

- 要测试哪个元素、流程或行为
- 需要哪些硬编码或模拟数据/行为，参考下面说明和 `test-warp-ui` 技能中的沙盒约束
- 启动前要预置哪些文件系统或应用状态，例如创建目录、写配置文件
- 要回报哪些具体观察结果

**示例提示词：**

```text
我把设置对话框标题改成了更大的字号和蓝色。
请临时硬编码启动时打开设置对话框，然后描述标题文本、
相对其他文本的字号以及颜色。
```

```text
我添加了一个迁移，会在首次启动时把配置从 ~/.warp 符号链接到 ~/.warp-preview。
迁移受 Channel::Preview 限制。构建前请移除 channel 检查，让迁移无论 channel
如何都会运行。同时创建一个包含测试文件的假 ~/.warp 目录。启动 Warp 后，
验证符号链接是否已在 ~/.warp-preview 中创建。
```

### 硬编码以到达待测路径

云端 Agent 用 `cargo run` 构建 Warp，运行条件可能不完全匹配你的改动，例如渠道不同、功能标志未启用或缺少前置状态。此时告诉 Agent 临时硬编码代码，让构建能覆盖你需要测试的路径。常见例子：

- **被门控的代码路径**：如果改动在渠道检查、功能标志或实验后面，告诉 Agent 在构建前移除或绕过门控。
- **预先存在的状态**：如果改动依赖干净环境中不存在的文件系统状态，例如旧安装的配置目录，告诉 Agent 在启动前创建。
- **启动行为**：如果改动只在首次启动或迁移时发生，确保 Agent 准备能触发它的前置条件。

提示词中要明确写出要硬编码什么以及原因，云端 Agent 不会自行推断。

### 5. 展示云端 Agent 链接

不需要额外展示步骤。Warp 客户端会自动显示云端 Agent 运行。
