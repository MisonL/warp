---
name: figma-create-new-file
description: 创建新的空白 Figma 文件。适用于用户想创建新的 Figma 设计或 FigJam 文件，或调用 use_figma 前需要新文件时。必要时会通过 whoami 解析套餐信息。用法：/figma-create-new-file [editorType] [fileName]。
description_zh_CN: 创建新的空白 Figma 文件。适用于用户想创建新的 Figma 设计或 FigJam 文件，或调用 use_figma 前需要新文件时。必要时会通过 whoami 解析套餐信息。
disable-model-invocation: true
---

# create_new_file - 创建新的 Figma 文件

使用 `create_new_file` MCP tool 在用户 drafts folder 中创建新的空白 Figma 文件。通常在需要 fresh file 并随后调用 `use_figma` 时使用。

## 技能参数

本技能接受可选参数：

```text
/figma-create-new-file [editorType] [fileName]
```

- `editorType`: `design` 默认，或 `figjam`。
- `fileName`: 新文件名称，默认 `"Untitled"`。

示例：

- `/figma-create-new-file`: 创建名为 `"Untitled"` 的 design 文件。
- `/figma-create-new-file figjam My Whiteboard`: 创建名为 `"My Whiteboard"` 的 FigJam 文件。
- `/figma-create-new-file design My New Design`: 创建名为 `"My New Design"` 的 design 文件。

从技能调用中解析参数。未提供 `editorType` 时默认 `"design"`。未提供 `fileName` 时默认 `"Untitled"`。

## 工作流

### 第 1 步：解析 planKey

`create_new_file` 工具需要 `planKey` 参数。按以下决策：

1. 用户已提供 `planKey`，例如来自之前的 `whoami` 调用或 prompt，直接使用并跳到第 2 步。
2. 没有 `planKey`，调用 `whoami` 工具。响应包含 `plans` array，每个 plan 有 `key`、`name`、`seat`、`tier`。
   - 只有一个 plan：自动使用其 `key`。
   - 多个 plan：询问用户要在哪个 team 或 organization 创建文件，然后使用对应 `key`。

### 第 2 步：调用 create_new_file

调用 `create_new_file`：

| Parameter | Required | Description |
| --- | --- | --- |
| `planKey` | Yes | 第 1 步得到的 plan key |
| `fileName` | Yes | 新文件名称 |
| `editorType` | Yes | `"design"` 或 `"figjam"` |

示例：

```json
{
  "planKey": "team:123456",
  "fileName": "My New Design",
  "editorType": "design"
}
```

### 第 3 步：使用结果

工具返回：

- `file_key`: 新文件 key。
- `file_url`: 可直接打开的 Figma URL。

后续调用 `use_figma` 等工具时使用 `file_key`。

## 注意事项

- 文件会创建在所选 plan 的用户 drafts folder 中。
- 仅支持 `"design"` 和 `"figjam"` editor type。
- 如果下一步要调用 `use_figma`，先加载 `figma-use` 技能。
