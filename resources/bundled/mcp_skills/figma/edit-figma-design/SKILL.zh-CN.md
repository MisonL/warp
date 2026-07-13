---
name: edit-figma-design
description: 使用 Figma MCP 创作工具，根据文字产品或 UI 描述直接创建或更新 Figma 设计。适用于从文本生成模型图、线框、页面、组件、流程或概念设计，或根据文字反馈迭代现有 Figma 文件。需要 Figma MCP 服务器连接。
description_zh_CN: 使用 Figma MCP 创作工具，根据文字产品或 UI 描述直接创建或更新 Figma 设计。适用于从文本生成模型图、线框、页面、组件、流程或概念设计，或根据文字反馈迭代现有 Figma 文件。需要 Figma MCP 服务器连接。
metadata:
  mcp-server: figma
---

# 编辑 Figma 设计

## 概览

本技能用于根据自然语言描述直接创建或更新 Figma 设计。它结合 Figma 库搜索和直接文件编辑；只有当设计需要贴合产品或代码库时，才使用 Warp Agent 的额外上下文能力。

## 适用场景

当用户想要：

- 根据文字描述在 Figma 中设计新页面、流程、组件或模型图。
- 根据文字反馈修改或扩展现有 Figma 文件。
- 直接在 Figma 中创建第一版线框图或更高保真设计。
- 让 Figma 设计对齐现有设计系统或产品词汇。

不要用于：

- 从设计生成生产代码，改用 `implement-design`。
- 把运行中的页面或应用捕获到 Figma，改用 `figma-generate-design`。
- 仅检查或拉取已有 Figma 上下文，改用 `pull-figma-content`。

## 前置条件

- Figma MCP 服务器已连接且可访问。
- 确认 `search_design_system`、`create_new_file`、`use_figma` 可用。
- 收集最少必要信息：要设计什么、使用现有文件还是新文件、是否要贴合设计系统或代码库。
- 只有信息不足以开始时才提问；问题要短，并尽量一次性问完。

## 必需流程

按顺序执行，不要跳步。

### 第 1 步：确认这是 Figma 创作请求

如果用户实际要实现代码，停止并改用 `implement-design`。

如果用户要从截图或网页捕获生成 Figma，停止并改用 `figma-generate-design`。本技能用于从文本生成设计。

### 第 2 步：先确定目标文件

`search_design_system` 和 `use_figma` 都需要 `fileKey`，所以先确定目标。

如果用户提供了 Figma URL 或 file key：

- 提取并使用该 `fileKey`。
- 回复时复用该 URL。

如果用户要新建文件：

1. 根据请求确定清晰文件名。
2. 如果用户已提供 `planKey`，直接使用。
3. 否则调用 Figma MCP 的 `whoami` 工具查看认证用户和可用 plan。注意这不是 shell `whoami`。
4. 只有一个 plan 时使用其 `key`。
5. 有多个 plan 时询问用户使用哪个 team 或 organization。
6. 调用 `create_new_file(editorType="design", fileName=..., planKey=...)`。
7. 保存返回的 `fileKey` 和 URL。第一版可用草稿完成后分享 URL。

### 第 3 步：按需收集上下文

判断是否需要 Figma 之外的上下文。

当用户只要探索性概念、线框图或模型图，且没有要求对齐代码库时，只使用 Figma MCP。

当用户要求对齐现有产品或设计系统时，选择性使用 Warp Agent 上下文：

- 读取 `AGENTS.md` 或 `WARP.md` 中的项目规则。
- 使用语义搜索、grep 和文件读取寻找相关组件、产品词汇、布局模式和设计 token 来源。
- 只有用户请求直接依赖外部系统或灵感来源时，才使用其他 MCP 或 web search。
- 本技能正常流程中不要编辑代码、运行 REPL 或使用 Computer Use。

### 第 4 步：创作前搜索设计系统

创建组件或样式前，先用目标 `fileKey` 调用 `search_design_system`。

优先搜索可复用资产：

- components 和 component sets
- variables 和类似 token 的值
- color、typography、spacing、effect styles

从用户领域词和项目上下文中发现的名称开始搜索。需要时用返回的 library key 缩小后续搜索。优先复用和 import 已有匹配项，而不是从零重建。

### 第 5 步：安全准备 `use_figma`

第一次调用 `use_figma` 前，规划编辑序列并遵守 Plugin API 约束。

增量 authoring 顺序：

1. 创建 page 和 frame 结构。
2. 建立 layout 和主要区块。
3. 复用或 import design-system 资产。
4. 应用 variables、styles、typography。
5. 添加内容和 polish。
6. 根据当前文件内容做针对性修订。

### 第 6 步：用小步 `use_figma` 编辑

使用多个小的 `use_figma` 调用，不要写一个巨大的 script。

好的切分边界：

- 创建 page 和 top-level frame。
- 布局 header、sidebar、hero 或 content region。
- import 或放置一组可复用组件。
- 绑定颜色、文本样式或 spacing variables。
- 更新某个区块的文案、状态或对齐方式。

每一步之后检查结果，确认成功后再继续。创建类似 component 的内容时，优先使用第 4 步发现的 library asset。

### 第 7 步：交付设计并给出后续选项

第一版可用草稿完成后：

- 返回 Figma URL。
- 高层概述创建或更新了什么。
- 询问用户是否要继续在 Figma 中修订。

如果用户要求把已确认设计实现为代码，停止使用本技能，改用 `implement-design`。

## Warp agent 指引

使用 Warp 的额外能力是为了减少手工提示，而不是增加无关工作。

适合使用：

- 查找 repo 中已有组件名或 design token。
- 读取约束 layout、命名或品牌的项目规则。
- 用户明确依赖其他系统时，从已连接系统拉取产品需求。

通常不需要：

- shell 命令或 REPL。
- 代码编辑。
- computer-use 验证。
- 没有明确请求的广泛 web research。

## 示例

新文件：用户要求为桌面应用设计账单概览界面，并使用现有设计系统。先确认是 Figma 创作，然后用 `whoami` 和 `create_new_file` 确定目标文件，必要时读取项目规则，调用 `search_design_system` 搜索账单相关组件，再用小步 `use_figma` 构建页面，最后返回文件 URL。

更新现有文件：用户提供 Figma URL 并要求添加入门检查清单。提取 `fileKey`，搜索检查清单、卡片、徽章、进度资产，用增量 `use_figma` 添加新区块，返回同一 URL 并说明变更。

纯探索概念：用户要移动端健身计划应用模型图，且不需要匹配代码库。新建文件，跳过代码库搜索，只用 Figma 库资产，直接构建概念并询问下一步修订。

## 常见问题

- 未说明使用现有文件还是新文件：问一个直接问题解决目标文件。拿到 `fileKey` 前不要调用 `search_design_system` 或 `use_figma`。
- `create_new_file` 有多个 Figma plan：询问用户用哪个 team 或 organization，不要猜。
- 用户要求匹配产品约定但请求模糊：先读项目规则，再做有目标的代码库搜索。
- 用户同时要求 Figma 设计和实现：若主要请求是 Figma 创作，先创建设计；若主要请求是实现，改用 `implement-design`。设计确认后可另起实现步骤。
- `use_figma` 失败或脚本变大：拆成更小调用，先结构，再样式，再局部修订。

## 资源

- [Figma MCP Server Documentation](https://developers.figma.com/docs/figma-mcp-server/)
- [Figma MCP Server Tools and Prompts](https://developers.figma.com/docs/figma-mcp-server/tools-and-prompts/)
