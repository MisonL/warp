---
name: figma-create-new-file
description: 创建新的空白 Figma file。当用户想创建新的 Figma design 或 FigJam file，或在调用 use_figma 前需要新文件时使用。必要时通过 whoami 处理 plan 解析。用法：/figma-create-new-file [editorType] [fileName]（例如 /figma-create-new-file figjam My Whiteboard）
disable-model-invocation: true
---

# create_new_file - 创建新的 Figma File

使用 `create_new_file` MCP tool，在用户的 drafts folder 中创建新的空白 Figma file。通常在需要全新文件继续操作时，在 `use_figma` 之前使用。

## Skill 参数

此 skill 接受可选参数：`/figma-create-new-file [editorType] [fileName]`

- **editorType**：`design`（默认）或 `figjam`
- **fileName**：新文件名称（默认是 "Untitled"）

示例：
- `/figma-create-new-file` - 创建名为 "Untitled" 的 design file
- `/figma-create-new-file figjam My Whiteboard` - 创建名为 "My Whiteboard" 的 FigJam file
- `/figma-create-new-file design My New Design` - 创建名为 "My New Design" 的 design file

从 skill invocation 中解析参数。如果未提供 editorType，默认为 `"design"`。如果未提供 fileName，默认为 `"Untitled"`。

## 工作流

### 步骤 1：解析 planKey

`create_new_file` tool 需要 `planKey` 参数。遵循以下决策树：

1. **用户已提供 planKey**（例如来自之前的 `whoami` 调用，或包含在 prompt 中）：直接使用它，跳到步骤 2。

2. **没有可用 planKey**：调用 `whoami` tool。响应包含 `plans` array。每个 plan 都有 `key`、`name`、`seat` 和 `tier`。

   - **单个 plan**：自动使用它的 `key` 字段。
   - **多个 plan**：询问用户想在哪个 team 或 organization 中创建文件，然后使用对应 plan 的 `key`。

### 步骤 2：调用 create_new_file

使用以下参数调用 `create_new_file` tool：

| 参数 | 是否必需 | 描述 |
|-------------|----------|-------------|
| `planKey` | 是 | 步骤 1 中的 plan key |
| `fileName` | 是 | 新文件名称 |
| `editorType` | 是 | `"design"` 或 `"figjam"` |

示例：
```json
{
  "planKey": "team:123456",
  "fileName": "My New Design",
  "editorType": "design"
}
```

### 步骤 3：使用结果

tool 返回：
- `file_key` - 新创建文件的 key
- `file_url` - 在 Figma 中打开文件的直接 URL

将 `file_key` 用于后续 tool call，例如 `use_figma`。

## 重要说明

- 文件会创建在所选 plan 下用户的 **drafts folder** 中。
- 只支持 `"design"` 和 `"figjam"` editor type。
- 如果下一步是 `use_figma`，调用前先加载 `figma-use` skill。
