---
name: agent-add-mcp
description: 帮助用户向 Warp 配置添加 MCP 服务器时使用此技能。
---

# 向 Warp 添加 MCP Server

Warp 通过原生配置文件支持 MCP server。帮助用户添加 MCP server 时按以下步骤执行。

## 步骤 1：确定作用域

如果用户没有说明，询问他们要配置为**全局**（所有项目可用）还是**项目级**（仅特定仓库可用）。

配置文件路径：

- **全局（用户级）**：`~/.warp/.mcp.json`
- **项目级**：`{repo_root}/.warp/.mcp.json`

## 步骤 2：收集 server 详情

如果用户没有提供 server 连接详情，使用 WebSearch 查找指定 server 的正确配置。

如果不确定该 server 应作为本地 CLI 进程运行（stdio transport），还是通过 URL 连接（HTTP/SSE streaming transport），询问用户偏好。

## 步骤 3：检查并准备配置文件

检查目标配置文件是否存在。

- **如果不存在**，用 `mkdir -p` 创建目录，并用空 `mcpServers` wrapper 初始化：

  ```json
  {
    "mcpServers": {}
  }
  ```

- **如果存在**，读取文件并确认已有的顶层 wrapper key。识别以下 wrapper key，按优先级排序：
  - `mcpServers`（优先）
  - `mcp_servers`
  - `servers`
  - `mcp.servers`（嵌套在 `mcp` key 下）
  - flat map（每个顶层 key 都是 server 名）

  写回时保留已有 wrapper key。如果已有 key 无法识别或不兼容，则切换到 `mcpServers`。

  **不要删除已有 server 条目**，只能添加或更新新 server。

## 步骤 4：写入 server 配置

### 基于命令的 server（stdio transport）

```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@scope/package-name"],
      "env": {
        "API_KEY": "${API_KEY}"
      }
    }
  }
}
```

默认情况下，Warp 从发现配置的目录启动 stdio server：

- 项目级配置（`{repo_root}/.warp/.mcp.json`）从 repo root 运行。
- 全局配置（`~/.warp/.mcp.json`、`~/.claude.json` 等）从 home directory 运行。

如果 server 的 `command` 或 `args` 是相对路径，例如 `./tooling/mcp/server.js`，或者 server 需要特定 cwd，则设置 `working_directory` 覆盖默认值：

```json
{
  "mcpServers": {
    "server-name": {
      "command": "node",
      "args": ["./tooling/mcp/server.js"],
      "working_directory": "/absolute/path/to/repo"
    }
  }
}
```

### 基于 URL 的 server（HTTP/SSE streaming transport）

```json
{
  "mcpServers": {
    "server-name": {
      "url": "https://example.com/mcp",
      "env": {
        "API_KEY": "${API_KEY}"
      }
    }
  }
}
```

对于包含密钥的环境变量，使用 `${VAR_NAME}` 语法。Warp 会在运行时从用户环境中替换值。

## 备注

- Warp 会在保存后自动检测 `.mcp.json` 文件变化，不需要重启。
- 配置的 server 会显示在 Warp Settings 的 MCP 页面，并标记为 **Detected from Warp**。
- 全局配置适用于所有 session；项目配置仅在该仓库内工作时适用。
