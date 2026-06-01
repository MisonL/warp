---
name: edit-figma-design
description: 使用 Figma MCP authoring tools，直接根据书面产品或 UI 描述创建或更新 Figma 设计。当用户希望从文本在 Figma 中设计 mockup、wireframe、screen、component、flow 或 concept，或希望根据文字反馈迭代现有 Figma 文件时使用。尽管名称如此，此 skill 可以从新的空白文件开始，也可以编辑现有文件。不要用于将运行中页面转换为 Figma 的 capture-based workflow；此类任务使用 `figma-generate-design`，代码实现请求使用 `implement-design`。需要连接 Figma MCP server。
metadata:
  mcp-server: figma
---

# 编辑 Figma 设计

## 概览

此 skill 直接根据自然语言描述创建或更新 Figma 设计。它结合 Figma library search 与直接 file authoring，并且只在需要让设计更理解产品或代码库时使用 Warp 更广泛的 agent 能力。

## 何时使用此 skill

当用户希望你执行以下任务时，使用此 skill：

- 根据书面描述在 Figma 中设计新的 screen、flow、component 或 mockup
- 根据文字反馈优化或扩展现有 Figma 文件
- 直接在 Figma 中创建初版 wireframe 或更高保真度的设计
- 让 Figma 设计对齐现有 design system 或产品词汇

以下情况不要使用此 skill：

- 用户希望从设计生成生产代码，应使用 `implement-design`
- 用户希望将运行中的页面或 app 捕获到 Figma，应使用 `figma-generate-design`
- 用户只是希望检查或拉取现有 Figma context，应使用 `pull-figma-content`

## 前置条件

- Figma MCP server 必须已连接且可访问。
  - 确认 `search_design_system`、`create_new_file` 和 `use_figma` 可用。
- 收集继续执行所需的最少信息：
  - 要设计什么
  - 使用现有 Figma 文件，还是创建新文件
  - 结果是否应对齐现有 design system 或代码库
- 只有在用户尚未提供足够启动细节时才提出澄清问题。问题应简短，并尽量合并到一条消息中。

## 必需工作流

**按顺序执行这些步骤。不要跳过步骤。**

### 步骤 1：确认这是 Figma authoring 请求

如果用户实际是在请求实现，停止并查阅 `implement-design`。

如果用户想要 screenshot-to-Figma 或网页捕获流程，停止并查阅 `figma-generate-design`。那个 skill 用于 capture-based workflow；此 skill 用于 text-to-design authoring。

### 步骤 2：先确定目标文件

`search_design_system` 和 `use_figma` 都需要 `fileKey`，因此在搜索或编辑之前先确定目标。

**如果用户提供了现有 Figma URL 或 file key：**

- 提取并使用该 `fileKey`。
- 回复时复用用户提供的 URL。

**如果用户想要新文件：**

1. 根据请求确定清晰的文件名。
2. 如果用户已经提供 `planKey`，使用它。
3. 否则调用 Figma MCP 的 `whoami` tool，检查已认证的 Figma 用户和可用 plan。这不是 shell 的 `whoami` 命令。
4. 如果只有一个 plan，使用它的 `key`。
5. 如果有多个 plan，询问用户要使用哪个团队或组织。
6. 调用 `create_new_file(editorType="design", fileName=..., planKey=...)`。
7. 保存返回的 `fileKey` 和 URL。第一个可用草稿就绪后再分享 URL。

### 步骤 3：只在需要时收集合适上下文

判断实际需要多少非 Figma 上下文。

当用户想要探索性 concept、wireframe 或 mockup，且没有要求对齐代码库时，**只留在 Figma MCP 内部**。

当用户希望设计匹配现有产品或 design system 时，**有选择地使用 Warp agent context**：

- 如果存在 `AGENTS.md` 和/或 `WARP.md`，读取其中的项目规则
- 使用语义化代码库搜索、grep 和文件读取，查找相关 component、产品词汇、layout pattern 和 design-token 来源
- 只有当 prompt 直接依赖其他 MCP source 或 web search 时才使用它们，例如另一系统中的产品需求或明确的灵感请求
- 不要把编辑代码、运行 REPL 命令或使用 computer use 作为此 skill 正常工作流的一部分

### 步骤 4：authoring 前先搜索 design system

在创建新 component 或 style 之前，使用已确定的 `fileKey` 调用 `search_design_system`。

优先搜索最可复用的 asset：

- component 和 component set
- variable 和 token-like value
- color、typography、spacing 或 effect 的 style

从用户的领域术语以及从项目规则或代码库搜索中发现的名称开始。

如有需要，使用返回的 library key 缩窄后续搜索，而不是立即扩大搜索范围。

优先复用和 import 匹配项，而不是从头重新创建。

### 步骤 5：安全准备 `use_figma`

第一次调用 `use_figma` 之前，规划编辑顺序，并遵守该工具要求的 Plugin API 约束。

保持 authoring plan 增量化：

1. 创建 page 和 frame 结构
2. 建立 layout 和主要 section
3. 复用或 import design-system asset
4. 应用 variable、style 和 typography
5. 添加内容并打磨
6. 基于文件当前内容做定向修订

### 步骤 6：用小步 `use_figma` 编辑设计

使用多个小型 `use_figma` 调用，而不是一个巨大的脚本。

良好的步骤边界：

- 创建 page 和顶层 frame
- 布局 header、sidebar、hero 或 content region
- import 或放置一组可复用 component
- 绑定 color、text style 或 spacing variable
- 更新某个具体 section 的文案、state 或 alignment

每一步之后检查结果，只有在上一步成功后才继续。

创建任何类似 component 的内容时，优先使用步骤 4 中发现并 import 的 library asset。

### 步骤 7：交付设计和后续选项

第一个可用草稿就绪后：

- 如果有 Figma URL，返回它
- 从高层概述创建或更新的内容
- 询问用户是否希望在 Figma 中继续修订

如果用户要求用代码实现已批准的设计，停止使用此 skill 并查阅 `implement-design`。

## Warp-agent 指引

使用 Warp 更广泛的能力来减少手动追问，而不是增加不必要的工作。

**此 skill 中适合使用 Warp agent 能力的场景：**

- 在 repo 中查找现有 component 名称或 design token
- 读取约束 layout、命名或品牌的项目规则
- 当用户明确依赖其他已连接系统时，从中拉取产品需求

**此 skill 中通常不必要的操作：**

- shell 命令或 REPL 访问
- 代码编辑
- computer-use validation
- 没有具体用户请求的广泛 web research

## 示例

### 示例 1：根据产品描述创建新文件

用户说："为我们的桌面 app 在 Figma 中设计一个 billing overview screen。使用现有 design system，并创建一个新文件。"

**操作：**

1. 确认这是 Figma authoring，而不是代码实现。
2. 如有需要，调用 `whoami` 来确定目标，然后调用 `create_new_file`。
3. 只有在需要理解 billing 术语和现有 component 时，读取 `AGENTS.md` 或 `WARP.md`，或搜索代码库。
4. 使用 billing 相关 query 调用 `search_design_system`。
5. 用小步 `use_figma` 构建 screen。
6. 返回新的 Figma 文件 URL，并询问是否需要修订。

### 示例 2：更新现有 Figma 文件

用户说："向这个 Figma 文件添加 onboarding checklist：https://figma.com/design/FILEKEY/Product?node-id=1-2"

**操作：**

1. 从现有 URL 中提取 `fileKey`。
2. 创建任何新内容前，在 design system 中搜索 checklist、card、badge 和 progress asset。
3. 使用增量 `use_figma` 调用添加新 section。
4. 返回同一个 Figma URL，并概述变更。

### 示例 3：纯探索性 concept

用户说："在 Figma 中创建一个初版移动端 workout planner mockup。暂时不需要匹配我的代码库。"

**操作：**

1. 如有需要，创建新文件。
2. 跳过代码库搜索和项目规则检查。
3. 仅为复用相关 Figma library asset 而使用 `search_design_system`。
4. 用小步 `use_figma` 直接在 Figma 中构建 concept。
5. 分享文件链接，并询问下一步要优化什么。

## 常见问题与响应

### 问题：用户没有说明使用现有文件还是新文件

提出一个直接问题来确定目标。在获得 `fileKey` 前，不要启动 `search_design_system` 或 `use_figma`。

### 问题：`create_new_file` 有多个可用 Figma plan

询问用户要使用哪个团队或组织。不要猜测。

### 问题：用户希望设计匹配现有产品约定，但请求很模糊

先读取项目规则。然后使用定向代码库搜索，只收集与请求界面相关的 component 和约定。

### 问题：用户同时要求 Figma design 和实现

只有当用户请求主要是 Figma authoring 时，才先创建或更新 Figma 设计。如果请求主要是实现，则改为查阅 `implement-design`。设计获批后，可以在单独步骤中继续实现。

### 问题：`use_figma` 失败或脚本变大

将任务拆成更小的 `use_figma` 调用。优先处理结构，其次是样式，最后是定向修订。

## 其他资源

- [Figma MCP Server Documentation](https://developers.figma.com/docs/figma-mcp-server/)
- [Figma MCP Server Tools and Prompts](https://developers.figma.com/docs/figma-mcp-server/tools-and-prompts/)
