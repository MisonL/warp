---
name: figma-implement-design
description: 将 Figma 设计转换为可用于生产的应用代码，并尽量保持 1:1 视觉还原。适用于从 Figma 文件实现 UI、生成代码、实现组件，或根据 Figma 规格构建组件时。若要通过 `use_figma` 写入画布，请使用 `figma-use`。
description_zh_CN: 将 Figma 设计转换为可用于生产的应用代码，并尽量保持 1:1 视觉还原。适用于从 Figma 文件实现 UI、生成代码、实现组件，或根据 Figma 规格构建组件时。
disable-model-invocation: false
---

# Implement Design

## 概览

本技能用于把 Figma 设计转成生产可用代码，并尽量保持 pixel-perfect。它提供结构化流程，确保正确使用 Figma MCP、design tokens，并与设计保持 1:1 视觉一致。

## 边界

- 交付物是用户仓库中的代码时使用。
- 若用户要在 Figma 内创建、编辑或删除节点，切换到 `figma-use`。
- 若用户要从代码或描述在 Figma 中构建完整页面，切换到 `figma-generate-design`。
- 若用户只要 Code Connect mapping，切换到 `figma-code-connect-components`。
- 若用户要编写 agent rules，例如 `CLAUDE.md` 或 `AGENTS.md`，切换到 `figma-create-design-system-rules`。

## 前置条件

- Figma MCP server 已连接且可访问。
- 用户提供 Figma URL：`https://figma.com/design/:fileKey/:fileName?node-id=1-2`。
- 或使用 `figma-desktop` MCP 时，用户已在 Figma desktop app 中选择节点。
- 项目最好已有 design system 或 component library。

## 必需流程

### 第 1 步：获取 Node ID

如果用户提供 URL，解析：

- `fileKey`: `/design/` 后的路径段。
- `nodeId`: query 参数 `node-id` 的值，例如 `1-2`。

使用本地 desktop MCP 时，tool 会使用当前打开文件，通常只需要 nodeId，甚至可使用当前 selection。

### 第 2 步：获取 Design Context

调用 `get_design_context`：

```text
get_design_context(fileKey=":fileKey", nodeId="1-2")
```

它提供 layout、constraints、sizing、typography、colors、tokens、component structure、variants、spacing、padding 等结构化数据。

若响应过大或被截断：

1. 调用 `get_metadata` 获取高层 node map。
2. 从 metadata 中识别需要的 child nodes。
3. 对具体 child node 再调用 `get_design_context`。

### 第 3 步：捕获视觉参考

用同一个 file key 和 node ID 调用 `get_screenshot`。该 screenshot 是视觉验证的 source of truth，整个实现过程中都要可访问。

```text
get_screenshot(fileKey=":fileKey", nodeId="1-2")
```

### 第 4 步：下载所需 assets

下载 Figma MCP server 返回的 image、icon、SVG 等 assets。

规则：

- 如果返回 localhost source，直接使用它。
- 不要新增 icon package；所有 assets 应来自 Figma payload。
- 有 localhost source 时不要创建 placeholder。
- assets 由 Figma MCP server 的 assets endpoint 提供。

### 第 5 步：转换为项目约定

把 Figma 输出转换成项目框架、样式和约定。

- 把 Figma MCP 输出当作设计和行为表示，不是最终代码风格。
- 将 Tailwind utility class 转换为项目首选 styling 或 design tokens。
- 复用已有 button、input、typography、icon wrapper 等组件，避免重复造轮子。
- 使用项目颜色系统、字体 scale 和 spacing tokens。
- 遵守现有 routing、state management 和 data-fetch patterns。

### 第 6 步：实现 1:1 视觉一致

优先匹配 Figma。避免硬编码，尽量使用 Figma 或项目 design token。若项目 token 与 Figma 规格冲突，通常优先项目 token，但可最小调整 spacing 或 size 保持视觉一致。遵守 WCAG 可访问性要求。必要时补充组件文档。

### 第 7 步：对照 Figma 验证

完成前对照截图验证：

- Layout 的 spacing、alignment、sizing 匹配。
- Typography 的 font、size、weight、line height 匹配。
- Colors 精确匹配或有明确 token 映射。
- hover、active、disabled 等状态按设计工作。
- responsive behavior 符合 Figma constraints。
- assets 正确渲染。
- 满足 accessibility 标准。

## 实现规则

### Component organization

- 把 UI components 放到项目指定 design system 目录。
- 遵循项目组件命名约定。
- 除非动态值确实需要，否则避免 inline style。

### Design system integration

- 尽可能使用项目 design system 中已有组件。
- 把 Figma design token 映射到项目 design token。
- 有匹配组件时扩展它，而不是创建新组件。
- 新增 design system 组件时补充文档。

### Code quality

- 避免硬编码值，抽取到常量或 design token。
- 保持组件可组合、可复用。
- 为 component props 添加 TypeScript 类型。
- 对导出组件添加必要 JSDoc。

## 示例

Button component：解析 URL，调用 `get_design_context` 和 `get_screenshot`，下载 icon，检查项目是否已有 Button，优先扩展现有组件，映射颜色 token，按 padding、radius、typography 对照截图验证。

Dashboard layout：先用 `get_metadata` 理解页面结构，识别 header、sidebar、content、cards 等 child nodes，分别拉取 context，下载 assets，用项目 layout primitives 和已有组件实现，然后验证 responsive behavior。

## 最佳实践

- 不要基于猜测实现。先获取 `get_design_context` 和 `get_screenshot`。
- 实现中频繁验证，不要只在最后检查。
- 如因可访问性或技术限制偏离 Figma，在代码注释中说明原因。
- 优先复用已有组件。代码库一致性比机械复制 Figma 更重要。
- 不确定时，优先项目 design system 模式，再做必要视觉调整。

## 常见问题

- Figma 输出被截断：用 `get_metadata` 获取结构，再分别拉取关键节点。
- 实现后不匹配：与第 3 步截图并排比较，重点查 spacing、colors、typography。
- assets 加载失败：检查 Figma MCP assets endpoint，直接使用 localhost URL，不要改写。
- design token 与 Figma 不同：优先项目 token 保持一致，同时微调 spacing 和 sizing 维护视觉还原。

## 资源

- [Figma MCP Server Documentation](https://developers.figma.com/docs/figma-mcp-server/)
- [Figma MCP Server Tools and Prompts](https://developers.figma.com/docs/figma-mcp-server/tools-and-prompts/)
- [Figma Variables and Design Tokens](https://help.figma.com/hc/en-us/articles/15339657135383-Guide-to-variables-in-Figma)
