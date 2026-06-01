---
name: figma-code-connect-components
description: 使用 Code Connect mapping tools 将 Figma design component 连接到代码 component。当用户说 "code connect"、"connect this component to code"、"map this component"、"link component to code"、"create code connect mapping"，或希望在 Figma design 与代码实现之间建立映射时使用。对于通过 `use_figma` 写入 canvas 的任务，请使用 `figma-use`。
disable-model-invocation: false
---

# Code Connect Components

## 概览

此 skill 帮助你使用 Figma 的 Code Connect 功能，将 Figma design component 连接到对应的代码实现。它会分析 Figma design 结构，搜索代码库中的匹配 component，并建立保持设计与代码一致性的映射。

## Skill 边界

- 将此 skill 用于 `get_code_connect_suggestions` + `send_code_connect_mappings` 工作流。
- 如果任务需要用 Plugin API scripts 写入 Figma canvas，切换到 [figma-use](../figma-use/SKILL.md)。
- 如果任务是根据代码或描述在 Figma 中构建或更新整页 screen，切换到 [figma-generate-design](../figma-generate-design/SKILL.md)。
- 如果任务是根据 Figma 实现产品代码，切换到 [figma-implement-design](../figma-implement-design/SKILL.md)。

## 前置条件

- Figma MCP server 必须已连接且可访问
- 用户必须提供带 node ID 的 Figma URL：`https://figma.com/design/:fileKey/:fileName?node-id=1-2`
  - **重要：** Figma URL 必须包含 `node-id` 参数。缺少它会导致 Code Connect mapping 失败。
- **或者** 使用 `figma-desktop` MCP 时：用户可以直接在 Figma desktop app 中选择 node（无需 URL）
- **重要：** Figma component 必须已发布到 team library。Code Connect 只适用于已发布的 component 或 component set。
- **重要：** Code Connect 仅在 Organization 和 Enterprise plan 上可用。
- 需要访问项目代码库来扫描 component

## 必需工作流

**按顺序执行这些步骤。不要跳过步骤。**

### 步骤 1：获取 Code Connect 建议

调用 `get_code_connect_suggestions`，一次性识别所有未映射的 component。此工具会自动：

- 从 Figma scenegraph 获取 component 信息
- 识别选区中的已发布 component
- 检查现有 Code Connect mapping，并过滤已经连接的 component
- 为每个未映射 component 返回 component 名称、属性和缩略图

#### 选项 A：使用 `figma-desktop` MCP（未提供 URL）

如果 `figma-desktop` MCP server 已连接，并且用户没有提供 Figma URL，立即调用 `get_code_connect_suggestions`。不需要解析 URL，desktop MCP server 会自动使用当前打开 Figma file 中选中的 node。

**注意：** 用户必须打开 Figma desktop app，并且已选择一个 node。`fileKey` 不会作为参数传入，server 会使用当前打开的文件。

#### 选项 B：提供了 Figma URL

解析 URL 以提取 `fileKey` 和 `nodeId`，然后调用 `get_code_connect_suggestions`。

**重要：** 从 Figma URL 提取 node ID 时，要转换格式：

- URL format 使用连字符：`node-id=1-2`
- 工具期望冒号：`nodeId=1:2`

**解析 Figma URL：**

- URL format：`https://figma.com/design/:fileKey/:fileName?node-id=1-2`
- 提取 file key：`:fileKey`（`/design/` 后面的 segment）
- 从 URL 提取 node ID：`1-2`，然后转换为工具使用的 `1:2`

```
get_code_connect_suggestions(fileKey=":fileKey", nodeId="1:2")
```

**处理响应：**

- 如果工具返回 **"No published components found in this selection"**，告知用户并停止。component 可能需要先发布到 team library。
- 如果工具返回 **"All component instances in this selection are already connected to code via Code Connect"**，告知用户所有内容都已映射。
- 否则，响应会包含未映射 component 列表，每一项包含：
  - Component name
  - Node ID
  - Component properties（包含 prop name 和 value 的 JSON）
  - Component 的缩略图（用于视觉检查）

### 步骤 2：扫描代码库以查找匹配 component

针对 `get_code_connect_suggestions` 返回的每个未映射 component，在代码库中搜索匹配的代码 component。

**查找内容：**

- 与 Figma component 名称匹配或相似的 component 名称
- 与 Figma hierarchy 对齐的 component 结构
- 对应 Figma property 的 props（variant、text、style）
- 典型 component 目录中的文件（`src/components/`、`components/`、`ui/` 等）

**搜索策略：**

1. 搜索名称匹配的 component 文件
2. 读取候选文件，检查结构和 props
3. 将代码 component 的 props 与步骤 1 返回的 Figma component properties 对比
4. 检测编程语言（TypeScript、JavaScript）和 framework（React、Vue 等）
5. 根据结构相似度识别最佳匹配，并权衡：
   - Prop 名称及其与 Figma property 的对应关系
   - 与 Figma defaults 匹配的默认值
   - CSS class 或 style object
   - 能阐明意图的描述性注释
6. 如果多个候选同样合适，选择 prop-interface 最接近的一个，并在工具调用前用 1-2 句注释记录理由

**示例搜索模式：**

- 如果 Figma component 是 "PrimaryButton"，搜索 `Button.tsx`、`PrimaryButton.tsx`、`Button.jsx`
- 检查常见 component 路径：`src/components/`、`app/components/`、`lib/ui/`
- 查找 `variant`、`size`、`color` 等与 Figma variant 匹配的 variant props

### 步骤 3：向用户展示匹配项

展示发现结果，并让用户选择要创建哪些 mapping。用户可以接受全部、部分或不接受建议的 mapping。

**按此格式展示匹配项：**

```
The following components match the design:
- [ComponentName](path/to/component): DesignComponentName at nodeId [nodeId](figmaUrl?node-id=X-Y)
- [AnotherComponent](path/to/another): AnotherDesign at nodeId [nodeId2](figmaUrl?node-id=X-Y)

Would you like to connect these components? You can accept all, select specific ones, or skip.
```

**如果某个 component 没有精确匹配：**

- 展示最接近的 2 个候选
- 说明差异
- 请用户确认使用哪个 component，或提供正确路径

**如果用户拒绝所有 mapping**，告知用户并停止。无需继续调用工具。

### 步骤 4：创建 Code Connect Mappings

用户确认选择后，只用已接受的 mapping 调用 `send_code_connect_mappings`。此工具会在一次调用中批量创建所有 mapping。

**示例：**

```
send_code_connect_mappings(
  fileKey=":fileKey",
  nodeId="1:2",
  mappings=[
    { nodeId: "1:2", componentName: "Button", source: "src/components/Button.tsx", label: "React" },
    { nodeId: "1:5", componentName: "Card", source: "src/components/Card.tsx", label: "React" }
  ]
)
```

**每个 mapping 的关键参数：**

- `nodeId`：Figma node ID（冒号格式：`1:2`）
- `componentName`：要连接的 component 名称（例如 "Button"、"Card"）
- `source`：代码 component 文件路径（相对于 project root）
- `label`：此 Code Connect mapping 的 framework 或 language label。有效值包括：
  - Web: 'React', 'Web Components', 'Vue', 'Svelte', 'Storybook', 'Javascript'
  - iOS: 'Swift UIKit', 'Objective-C UIKit', 'SwiftUI'
  - Android: 'Compose', 'Java', 'Kotlin', 'Android XML Layout'
  - Cross-platform: 'Flutter'
  - Docs: 'Markdown'

**调用之后：**

- 成功时：工具会确认 mapping 已创建
- 出错时：工具会报告具体哪个 mapping 失败以及原因（例如 "Component is already mapped to code"、"Published component not found"、"Insufficient permissions"）

**处理后提供摘要**：

```
Code Connect Summary:
- Successfully connected: 3
  - Button (1:2) → src/components/Button.tsx
  - Card (1:5) → src/components/Card.tsx
  - Input (1:8) → src/components/Input.tsx
- Could not connect: 1
  - CustomWidget (1:10) - No matching component found in codebase
```

## 示例

### 示例 1：连接 Button Component

用户说："将这个 Figma button 连接到我的代码：https://figma.com/design/kL9xQn2VwM8pYrTb4ZcHjF/DesignSystem?node-id=42-15"

**操作：**

1. 解析 URL：fileKey=`kL9xQn2VwM8pYrTb4ZcHjF`，nodeId=`42-15`，转换为 `42:15`
2. 运行 `get_code_connect_suggestions(fileKey="kL9xQn2VwM8pYrTb4ZcHjF", nodeId="42:15")`
3. 响应显示：Button component（未映射），包含 `variant`（primary/secondary）和 `size`（sm/md/lg）属性，以及缩略图
4. 在代码库中搜索 Button component：找到 `src/components/Button.tsx`
5. 读取 `Button.tsx`，确认它有 `variant` 和 `size` props
6. 向用户展示：
   ```
   I found a match:
   - [Button](src/components/Button.tsx): Button at nodeId [42:15](https://figma.com/design/kL9xQn2VwM8pYrTb4ZcHjF/DesignSystem?node-id=42-15)

   Would you like to connect this component?
   ```

7. 用户确认："Yes"
8. 检测到它是 TypeScript React component
9. 运行 `send_code_connect_mappings(fileKey="kL9xQn2VwM8pYrTb4ZcHjF", nodeId="42:15", mappings=[{ nodeId: "42:15", componentName: "Button", source: "src/components/Button.tsx", label: "React" }])`

**结果：** Figma button component 现在已连接到代码中的 Button component。

### 示例 2：多个 component，部分选择

用户说："连接这个 frame 中的 component：https://figma.com/design/pR8mNv5KqXzGwY2JtCfL4D/Components?node-id=10-50"

**操作：**

1. 解析 URL：fileKey=`pR8mNv5KqXzGwY2JtCfL4D`，nodeId=`10-50`，转换为 `10:50`
2. 运行 `get_code_connect_suggestions(fileKey="pR8mNv5KqXzGwY2JtCfL4D", nodeId="10:50")`
3. 响应显示 3 个未映射 component：ProductCard、Badge 和 CustomWidget
4. 搜索代码库：
   - ProductCard：找到 `src/components/ProductCard.tsx`（props 匹配）
   - Badge：找到 `src/components/Badge.tsx`（props 匹配）
   - CustomWidget：未找到匹配 component
5. 向用户展示：
   ```
   The following components match the design:
   - [ProductCard](src/components/ProductCard.tsx): ProductCard at nodeId [10:51](https://figma.com/design/pR8mNv5KqXzGwY2JtCfL4D/Components?node-id=10-51)
   - [Badge](src/components/Badge.tsx): Badge at nodeId [10:52](https://figma.com/design/pR8mNv5KqXzGwY2JtCfL4D/Components?node-id=10-52)

   I couldn't find a match for CustomWidget (10:53).

   Would you like to connect these components? You can accept all, select specific ones, or skip.
   ```

6. 用户："Just connect ProductCard, skip Badge for now"
7. 运行 `send_code_connect_mappings(fileKey="pR8mNv5KqXzGwY2JtCfL4D", nodeId="10:50", mappings=[{ nodeId: "10:51", componentName: "ProductCard", source: "src/components/ProductCard.tsx", label: "React" }])`

**结果：** 只按用户选择连接 ProductCard。

### 示例 3：需要创建 component

用户说："连接这个 icon：https://figma.com/design/8yJDMeWDyBz71EnMOSuUiw/Icons?node-id=5-20"

**操作：**

1. 解析 URL：fileKey=`8yJDMeWDyBz71EnMOSuUiw`，nodeId=`5-20`，转换为 `5:20`
2. 运行 `get_code_connect_suggestions(fileKey="8yJDMeWDyBz71EnMOSuUiw", nodeId="5:20")`
3. 响应显示：CheckIcon component（未映射），包含 color 和 size 属性
4. 在代码库中搜索 CheckIcon：未找到匹配项
5. 搜索通用 Icon component：找到 `src/icons/` 目录，里面有其他 icon
6. 向用户报告："I couldn't find a CheckIcon component, but I found an icons directory at src/icons/. Would you like to:
   - Create a new CheckIcon.tsx component first, then connect it
   - Connect to a different existing icon
   - Provide the path to the CheckIcon if it exists elsewhere"
7. 用户提供路径："src/icons/CheckIcon.tsx"
8. 从文件中检测 language 和 framework
9. 运行 `send_code_connect_mappings(fileKey="8yJDMeWDyBz71EnMOSuUiw", nodeId="5:20", mappings=[{ nodeId: "5:20", componentName: "CheckIcon", source: "src/icons/CheckIcon.tsx", label: "React" }])`

**结果：** CheckIcon component 已成功连接到 Figma design。

## 最佳实践

### 主动发现 Component

不要只是向用户索要文件路径，而应主动搜索代码库来查找匹配 component。这会提供更好的体验，并发现潜在 mapping 机会。

### 准确匹配结构

将 Figma component 与代码 component 对比时，不要只看名称。检查：

- Props 是否对齐（variant 类型、size 选项等）
- Component hierarchy 是否匹配（嵌套元素）
- Component 是否服务于同一目的

### 清晰沟通

提出创建 mapping 时，清楚说明：

- 发现了什么
- 为什么它是好匹配
- mapping 会做什么
- props 将如何连接

### 处理歧义

如果多个 component 都可能匹配，展示选项而不是猜测。让用户最终决定连接哪个 component。

### 优雅降级

如果找不到精确匹配，提供有帮助的后续步骤：

- 展示接近的候选
- 建议创建 component
- 询问用户指引

## 常见问题与解决方案

### 问题："No published components found in this selection"

**原因：** Figma component 尚未发布到 team library。Code Connect 只适用于已发布的 component。
**解决方案：** 用户需要在 Figma 中将 component 发布到 team library：

1. 在 Figma 中选择 component 或 component set
2. 右键选择 "Publish to library"，或使用 Team Library publish modal
3. 发布 component
4. 发布后，使用同一个 node ID 重试 Code Connect mapping

### 问题："Code Connect is only available on Organization and Enterprise plans"

**原因：** 用户的 Figma plan 不包含 Code Connect access。
**解决方案：** 用户需要升级到 Organization 或 Enterprise plan，或联系管理员。

### 问题：代码库中未找到匹配 component

**原因：** 代码库搜索没有找到名称或结构匹配的 component。
**解决方案：** 询问用户该 component 是否以其他名称存在，或位于其他位置。他们可能需要先创建 component，或该 component 可能位于非预期目录中。

### 问题："Published component not found" (CODE_CONNECT_ASSET_NOT_FOUND)

**原因：** source file path 不正确、该位置不存在 component，或 componentName 与实际 export 不匹配。
**解决方案：** 验证 source path 正确且相对于 project root。检查 component 是否按指定的精确 componentName 从文件中正确 export。

### 问题："Component is already mapped to code" (CODE_CONNECT_MAPPING_ALREADY_EXISTS)

**原因：** 此 component 已经存在 Code Connect mapping。
**解决方案：** 此 component 已连接。如果用户想更新 mapping，可能需要先在 Figma 中删除现有 mapping。

### 问题："Insufficient permissions to create mapping" (CODE_CONNECT_INSUFFICIENT_PERMISSIONS)

**原因：** 用户没有 Figma file 或 library 的编辑权限。
**解决方案：** 用户需要有包含该 component 的文件的编辑权限。联系文件所有者或 team admin。

### 问题：Code Connect mapping 因 URL 错误失败

**原因：** Figma URL format 不正确，或缺少 `node-id` 参数。
**解决方案：** 验证 URL 符合所需格式：`https://figma.com/design/:fileKey/:fileName?node-id=1-2`。`node-id` 参数是必需的。调用工具时也要确保将 `1-2` 转换为 `1:2`。

### 问题：找到多个相似 component

**原因：** 代码库包含多个可能匹配 Figma component 的 component。
**解决方案：** 向用户展示所有候选及其文件路径，并让他们选择要连接哪一个。不同 component 可能用于不同上下文（例如 `Button.tsx` 和 `LinkButton.tsx`）。

## 理解 Code Connect

Code Connect 会在设计与代码之间建立双向链接：

**对设计师：** 查看哪个代码 component 实现了某个 Figma component
**对开发者：** 从 Figma design 直接导航到实现它们的代码
**对团队：** 维护 component mapping 的单一事实来源

你创建的 mapping 会让这些连接明确且可发现，从而帮助设计与代码保持同步。

## 其他资源

关于 Code Connect 的更多信息：

- [Code Connect Documentation](https://help.figma.com/hc/en-us/articles/23920389749655-Code-Connect)
- [Figma MCP Server Tools and Prompts](https://developers.figma.com/docs/figma-mcp-server/tools-and-prompts/)
