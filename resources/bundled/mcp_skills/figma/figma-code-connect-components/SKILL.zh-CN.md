---
name: figma-code-connect-components
description: Connects Figma design components to code components using Code Connect mapping tools. Use when user says "code connect", "connect this component to code", "map this component", "link component to code", "create code connect mapping", or wants to establish mappings between Figma designs and code implementations. For canvas writes via `use_figma`, use `figma-use`.
description_zh_CN: 使用 Code Connect 映射工具将 Figma 设计组件连接到代码组件。适用于用户要求建立 Figma 设计与代码实现之间的映射时。若要通过 `use_figma` 写入画布，请使用 `figma-use`。
disable-model-invocation: false
---

# Code Connect Components

## 概览

本技能使用 Figma Code Connect，把设计组件映射到对应代码实现。它会分析 Figma 结构、搜索代码库中的匹配组件，并建立保持设计和代码一致的 mapping。

## 边界

- 用于 `get_code_connect_suggestions` + `send_code_connect_mappings` 工作流。
- 若需要通过 Plugin API 写入 Figma canvas，切换到 `figma-use`。
- 若要从代码或描述构建完整 Figma 页面，切换到 `figma-generate-design`。
- 若要从 Figma 实现产品代码，切换到 `figma-implement-design`。

## 前置条件

- Figma MCP server 已连接且可访问。
- 用户提供带 node ID 的 Figma URL：`https://figma.com/design/:fileKey/:fileName?node-id=1-2`。
- 或使用 `figma-desktop` MCP，用户已在 Figma desktop app 中选择节点。
- Figma component 必须发布到 team library。Code Connect 只适用于已发布 component 或 component set。
- Code Connect 只在 Organization 和 Enterprise plan 可用。
- 需要访问项目代码库以扫描组件。

## 必需流程

### 第 1 步：获取 Code Connect 建议

调用 `get_code_connect_suggestions`，一次性识别所有未映射组件。该工具会获取 scenegraph component 信息、识别 selection 中已发布组件、过滤已映射组件，并返回未映射组件的名称、属性和缩略图。

如果使用 `figma-desktop` MCP 且用户没有提供 Figma URL，直接调用 `get_code_connect_suggestions`。desktop MCP 会使用当前打开文件中已选中的节点，不需要传 `fileKey`。

如果用户提供了 Figma URL，解析 `fileKey` 和 `nodeId`，再调用工具。注意 URL 中 `node-id=1-2` 使用 hyphen，工具期望 `nodeId="1:2"`，需要转换。

```text
get_code_connect_suggestions(fileKey=":fileKey", nodeId="1:2")
```

处理响应：

- 返回 "No published components found in this selection" 时，告知用户需要先发布组件到 team library，然后停止。
- 返回 "All component instances in this selection are already connected to code via Code Connect" 时，告知用户都已映射。
- 否则记录每个未映射组件的 component name、node ID、component properties 和 thumbnail。

### 第 2 步：扫描代码库匹配组件

对每个未映射组件搜索代码库。

查找内容：

- 与 Figma 组件名相同或相近的组件。
- 与 Figma 层级结构一致的组件结构。
- 与 Figma properties 对应的 props，例如 variants、text、styles。
- 常见目录：`src/components/`、`components/`、`ui/` 等。

搜索策略：

1. 搜索名称匹配的组件文件。
2. 读取候选文件，检查结构和 props。
3. 对比代码 props 和第 1 步返回的 Figma properties。
4. 判断语言和框架，例如 TypeScript、JavaScript、React、Vue。
5. 基于 prop 名、默认值、CSS class 或 style object、注释等结构相似性选择最佳匹配。
6. 多个候选同样合适时，选 prop interface 最接近者，并在工具调用前用 1 到 2 句话说明理由。

### 第 3 步：把匹配结果给用户确认

让用户选择要创建的 mapping。用户可以接受全部、部分或跳过。

格式：

```text
The following components match the design:
- [ComponentName](path/to/component): DesignComponentName at nodeId [nodeId](figmaUrl?node-id=X-Y)
- [AnotherComponent](path/to/another): AnotherDesign at nodeId [nodeId2](figmaUrl?node-id=X-Y)

Would you like to connect these components? You can accept all, select specific ones, or skip.
```

若没有精确匹配，展示 2 个最接近候选、说明差异，并请用户确认或提供正确路径。若用户拒绝所有 mapping，说明后停止，不再调用工具。

### 第 4 步：创建 Code Connect mapping

用户确认后，只对接受的 mapping 调用 `send_code_connect_mappings`。该工具会批量创建。

```text
send_code_connect_mappings(
  fileKey=":fileKey",
  nodeId="1:2",
  mappings=[
    { nodeId: "1:2", componentName: "Button", source: "src/components/Button.tsx", label: "React" },
    { nodeId: "1:5", componentName: "Card", source: "src/components/Card.tsx", label: "React" }
  ]
)
```

每个 mapping 的关键参数：

- `nodeId`: Figma node ID，使用 colon 格式。
- `componentName`: 要连接的代码组件名。
- `source`: 相对项目根目录的组件文件路径。
- `label`: framework 或语言标签。常见值包括 `React`、`Web Components`、`Vue`、`Svelte`、`Storybook`、`Javascript`、`Swift UIKit`、`Objective-C UIKit`、`SwiftUI`、`Compose`、`Java`、`Kotlin`、`Android XML Layout`、`Flutter`、`Markdown`。

调用后：

- 成功时工具会确认 mapping 已创建。
- 失败时工具会说明哪些 mapping 失败以及原因，例如已映射、未找到 published component、权限不足。

最后给出汇总：

```text
Code Connect Summary:
- Successfully connected: 3
  - Button (1:2) -> src/components/Button.tsx
  - Card (1:5) -> src/components/Card.tsx
  - Input (1:8) -> src/components/Input.tsx
- Could not connect: 1
  - CustomWidget (1:10) - No matching component found in codebase
```

## 示例

单个 Button：解析 URL，转换 `node-id=42-15` 到 `42:15`，调用 `get_code_connect_suggestions`，确认返回 Button 未映射；搜索 `src/components/Button.tsx` 并确认有 `variant` 和 `size` props；让用户确认；确认后调用 `send_code_connect_mappings`。

多个组件：对 frame 调用建议工具，分别搜索 ProductCard、Badge、CustomWidget；展示可连接项和无法匹配项，让用户选择；只为用户接受的组件发送 mapping。
