---
name: verify-ui-change-in-cloud
description: 完成任何用户可见客户端变更后，在非沙盒或本地环境中自动调用此技能。它会启动带 computer use 的云端 Agent 来测试 Warp，并运行 test-warp-ui 技能。
user-invocable: true
---

# 在云端验证 UI 变更

完成用户可见的客户端变更后，使用此工作流启动带 computer use 能力的云端 agent 来验证变更。适用于任何影响用户在运行中 app 里看到或体验到的内容的改动，不仅包括视觉 UI，也包括启动行为、配置处理、迁移流程和其他客户端逻辑。

## 工作流

### 1. 推送改动

云端 agent 会在新环境中 clone 仓库。必须先把改动 push 到一个分支，使云端 agent 可以访问。

### 2. 检测仓库

启动云端 agent 前，检测当前所在仓库。检查 Git remote URL：

```bash
git remote get-url origin
```

确认 remote URL 包含 `warpdotdev/warp`。如果不是，提醒用户此技能只支持 warp 仓库并停止。

warp Dev Environment 的 environment ID 是 `SVhg783GBFQHk1OfdPfFU9`。

### 3. 启动云端 Agent

使用 `run_agents` 工具启动远程云端 agent。单 child batch（`agent_run_configs` 只有一个 entry）是有效的。

- `summary`：简短声明性说明，例如 `"Spawning a cloud agent with computer use to verify the UI change."`
- `base_prompt`：要求读取并遵循 `test-warp-ui` skill，然后写入验证任务
- `remote.environment_id`：`SVhg783GBFQHk1OfdPfFU9`
- `remote.computer_use_enabled`：`true`
- `agent_run_configs`：单个 entry，`name` 使用短显示名，例如 `"verify-ui-change"`。单 agent 的 `prompt` 可以为空，因为 `base_prompt` 已覆盖任务

`test-warp-ui` skill 是内置的，云端 agent 会自动拥有它。在 `base_prompt` 中明确要求 agent 调用该 skill，例如 “Read and follow the test-warp-ui skill.”

### 4. 编写有效 prompt

Prompt 应告诉云端 agent：

- 要测试哪个元素、流程或行为
- 需要哪些硬编码或 mock，参考下面说明和 `test-warp-ui` skill 中的沙盒约束
- 启动前要预置哪些文件系统或 app 状态，例如创建目录、写配置文件
- 要回报哪些具体观察结果

**示例 prompt：**

```text
I changed the settings dialog header to use a larger font and blue color.
Hardcode the settings dialog to open on launch, then describe the header text,
font size relative to other text, and color.
```

```text
I added a migration that symlinks config from ~/.warp into ~/.warp-preview on first launch.
The migration is gated on Channel::Preview. Before building, hardcode the migration to run
regardless of channel by removing the channel check. Also create a fake ~/.warp directory
with test files. After launching Warp, verify the symlinks were created in ~/.warp-preview.
```

### 硬编码以到达待测路径

云端 agent 用 `cargo run` 构建 Warp，运行条件可能不完全匹配你的改动，例如 channel 不同、feature flag 未启用或缺少前置状态。此时告诉 agent 临时硬编码代码，让构建能覆盖你需要测试的路径。常见例子：

- **被 gate 的代码路径**：如果改动在 channel check、feature flag 或 experiment 后面，告诉 agent 在构建前移除或绕过 gate。
- **预先存在的状态**：如果改动依赖干净环境中不存在的文件系统状态，例如旧安装的配置目录，告诉 agent 在启动前创建。
- **启动行为**：如果改动只在首次启动或迁移时发生，确保 agent 准备能触发它的前置条件。

Prompt 中要明确写出要硬编码什么以及原因，云端 agent 不会自行推断。

### 5. 展示云端 Agent 链接

不需要额外展示步骤。Warp client 会自动显示云端 agent run。
