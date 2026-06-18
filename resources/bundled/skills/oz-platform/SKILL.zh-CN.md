---
name: oz-platform
description: Use Warp's REST API and command line to run, configure, and inspect Oz cloud agents
description_zh_CN: 使用 Warp 的 REST API 和命令行运行、配置并检查 Oz 云端 Agent。
---

# oz-platform

使用 Oz REST API 和 CLI 完成：

- 启动 cloud agent。
- 查看 cloud agent 状态。
- 创建重复运行的计划任务。
- 创建和管理 cloud agent 运行环境。
- 提供 cloud agent 可用的 secret。

## 命令行

Oz CLI 安装为 `{{warp_cli_binary_name}}`。查看帮助：

```sh
{{warp_cli_binary_name}} help
{{warp_cli_binary_name}} help <subcommand>
```

人工查看优先用 `--output-format text`，需要用 `jq` 解析时用 `--output-format json`。更多信息见 https://docs.warp.dev/reference/cli。

关键命令：

- `{{warp_cli_binary_name}} agent run-cloud`: 启动新的 cloud agent，可配置 prompt、model、environment 等。
- `{{warp_cli_binary_name}} run list` 和 `{{warp_cli_binary_name}} run get <run-id>`: 列出运行并查看某次运行详情。
- `{{warp_cli_binary_name}} environment list` 和 `{{warp_cli_binary_name}} environment get`: 列出和查看 environment。
- `{{warp_cli_binary_name}} schedule list` 和 `{{warp_cli_binary_name}} schedule get`: 列出计划任务和查看某次计划运行。

大多数子命令支持 `--output-format json`。

### 示例

启动 cloud agent 并查看状态：

```sh
$ {{warp_cli_binary_name}} agent run-cloud --prompt "Update the login error to be more specific" --environment UA17BXYZ
# ...
Spawned agent with run ID: 5972cca4-a410-42af-930a-e56bc23e07ac
```

```sh
$ {{warp_cli_binary_name}} run get 5972cca4-a410-42af-930a-e56bc23e07ac
# ...
```

每天 UTC 8 点让 agent 汇总反馈：

```sh
$ {{warp_cli_binary_name}} schedule create --cron "0 8 * * *" \
    --name "GitHub issue summary" \
    --prompt "Collect all feedback from new GitHub issues and provide a summary report" \
    --environment UA17BXYZ
```

列出和查看计划任务：

```sh
$ {{warp_cli_binary_name}} schedule list
$ {{warp_cli_binary_name}} schedule get <schedule-id>
```

创建 cloud agent 可用的 secret：

```sh
$ {{warp_cli_binary_name}} secret create JIRA_API_KEY --team --value-file jira_key.txt --description "API key to access Jira"
```

## REST API

Oz 提供 REST API 用于启动和检查 cloud agent。所有请求都需要 API key。用户可在 Warp 设置的 `Platform` 页面生成 API key，入口为 `{{warp_url_scheme}}://settings/platform`。

完整 OpenAPI 规格见 https://docs.warp.dev/reference/api-and-sdk。

### TypeScript / JavaScript SDK

- Package: https://www.npmjs.com/package/oz-agent-sdk
- Source: https://github.com/warpdotdev/oz-sdk-typescript
- API reference: https://raw.githubusercontent.com/warpdotdev/oz-sdk-typescript/HEAD/api.md

### Python SDK

- Package: https://pypi.org/project/oz-agent-sdk/
- Source: https://github.com/warpdotdev/oz-sdk-python
- API reference: https://raw.githubusercontent.com/warpdotdev/oz-sdk-python/refs/heads/main/api.md

### API 示例

```sh
curl -L -X POST {{warp_server_url}}/api/v1/agent/run \
    --header 'Authorization: Bearer YOUR_API_KEY' \
    --header 'Content-Type: application/json' \
    --data '{
        "prompt": "Update the login error to be more specific",
        "config": {
            "environment_id": "UA17BXYZ"
        }
    }'
```

```sh
curl -L -X GET {{warp_server_url}}/api/v1/agent/runs/5972cca4-a410-42af-930a-e56bc23e07ac \
    --header 'Authorization: Bearer YOUR_API_KEY' \
    --header 'Content-Type: application/json'
```

## GitHub Actions 集成

可以从 GitHub Actions workflow 触发 Oz cloud agent，用于 issue 分诊、PR 检查、CI 事件响应等自动化。

当触发源本身在 GitHub 中时使用 GitHub Actions，例如 issue opened、PR labeled、push、CI workflow completed。周期性任务优先用 `{{warp_cli_binary_name}} schedule create`，这样 Oz 平台可以追踪计划运行。

agent 可以使用 `gh` CLI 回写仓库。优先提示 agent 使用 `gh`，而不是要求 agent 输出结构化结果让 workflow 解析。

### Action 设置

workflow 中使用 `warpdotdev/oz-agent-action@main`。必填输入：

- `prompt`: agent 的任务说明。
- `warp_api_key`: API key，放在 GitHub secrets 中。
- `profile`: 可选 agent profile identifier。

action 输出 `agent_output`。

### 最小示例

```yaml
name: Run Oz Agent
on:
  issues:
    types: [opened, labeled]

jobs:
  agent:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      issues: write
      pull-requests: write
    steps:
      - uses: actions/checkout@v6
      - uses: warpdotdev/oz-agent-action@main
        id: agent
        with:
          prompt: |
            Analyze the GitHub issue and provide a summary.
            Issue: ${{ github.event.issue.title }}
            ${{ github.event.issue.body }}

            Respond to the issue with a comment containing your summary using the `gh` CLI.
          warp_api_key: ${{ secrets.WARP_API_KEY }}
          profile: ${{ vars.WARP_AGENT_PROFILE || '' }}
      - name: Use Agent Output
        run: echo "${{ steps.agent.outputs.agent_output }}"
```

## Environments

所有 cloud agent 都运行在 environment 中。environment 定义：

- 预装哪些程序，通常基于 Docker image。
- agent 启动前 checkout 哪些 Git 仓库。
- 要执行哪些 setup command，例如 `npm install` 或 `cargo fetch`。

几乎总是应该使用 environment，否则 agent 可能缺少必要代码或工具。cloud agent 在 sandbox 中运行，可以安装额外程序，也有 Git credentials 用于创建 PR 和 push branch。

environment 不存储 API key 等 secret。使用 `{{warp_cli_binary_name}} secret` 命令管理 secret。

## 使用第三方 Coding CLI

Oz environment 支持 Claude Code、Codex、Gemini CLI、Amp、Copilot CLI、OpenCode 等第三方 coding agent CLI。预构建 Oz Docker image 的 `-agents` tag 包含常见 CLI，例如 `warpdotdev/dev-rust:1.85-agents`。不带 `-agents` 的 base tag 不包含这些 CLI。

详细 per-CLI 文档见 `references/third-party-clis.md`。

### 交互式 agent 启动 cloud agent

当交互式 agent 要启动 cloud agent 使用第三方 CLI 时：

1. 先问用户使用哪个 environment。可展示公开 `-agents` image：
   - `warpdotdev/dev-base:latest-agents`
   - `warpdotdev/dev-go:1.23-agents`
   - `warpdotdev/dev-rust:1.83-agents`
   - `warpdotdev/dev-rust:1.85-agents`
   - `warpdotdev/dev-java:21-agents`
   - `warpdotdev/dev-dotnet:8.0-agents`
   - `warpdotdev/dev-ruby:3.3-agents`
   - `warpdotdev/dev-web:latest-agents`
   - `warpdotdev/dev-full:latest-agents`

   也询问是否使用已有 environment。列出已有 environment：

   ```sh
   {{warp_cli_binary_name}} environment list --output-format text
   ```

2. 构造简单 prompt，把 CLI 调用委托给 cloud agent：

   ```sh
   {{warp_cli_binary_name}} agent run-cloud \
       --environment <ENV_ID> \
       --prompt 'Read the oz-platform skill for instructions on using [CLI name] to solve: <task description>'
   ```

   不要在 prompt 中包含 CLI 命令语法。cloud agent 会读取本技能并按说明操作。

### Cloud agent 调用第三方 CLI

当你是 cloud agent，并被要求使用第三方 CLI：

1. 你已经在预装 CLI 的 environment 中运行。
2. API key 通过环境变量提供，例如 `ANTHROPIC_API_KEY`、`OPENAI_API_KEY`、`GEMINI_API_KEY`。
3. 用户任务必须完全由第三方 CLI 完成。不要使用 Warp 内置工具替它读文件、搜索、编辑或分析。你的职责只是设置 CLI、构造 prompt、运行并监控 CLI、调试 CLI 本身、报告产物。
4. 读取 `references/third-party-clis.md`，确认非交互模式 flag、认证步骤、常用参数和示例命令。
5. 如果第三方 CLI 创建了 PR，从输出中解析 PR URL 和 branch，调用 `report_pr` 在 Warp UI 中登记产物。

示例：

```sh
$ claude -p "Summarize the architecture of this project"
```

不要自己调用文件读取、grep 或编辑工具来帮 CLI 完成用户任务。让第三方 CLI 做完整工作。
