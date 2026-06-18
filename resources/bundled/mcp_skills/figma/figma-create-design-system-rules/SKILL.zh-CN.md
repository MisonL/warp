---
name: figma-create-design-system-rules
description: Generates custom design system rules for the user's codebase. Use when user says "create design system rules", "generate rules for my project", "set up design rules", "customize design system guidelines", or wants to establish project-specific conventions for Figma-to-code workflows. Requires Figma MCP server connection.
description_zh_CN: 为用户代码库生成自定义设计系统规则。适用于用户想创建设计系统规则、为项目生成规则、设置设计规则，或为 Figma 到代码工作流建立项目专属约定时。需要 Figma MCP 服务器连接。
disable-model-invocation: false
---

# Create Design System Rules

## 概览

本技能为用户项目生成定制 design system rules。这些规则帮助 AI coding agent 在实现 Figma 设计时遵守团队约定、组件模式和架构决策，产出一致且高质量的代码。

支持的规则文件：

| Agent | Rule File |
| --- | --- |
| Claude Code | `CLAUDE.md` |
| Codex CLI | `AGENTS.md` |
| Cursor | `.cursor/rules/figma-design-system.mdc` |

## 什么是 Design System Rules

Design system rules 是项目级说明，用来编码代码库里的隐性知识：

- 使用哪些 layout primitive 和组件。
- 组件文件放在哪里。
- 组件命名和结构方式。
- 哪些值不能硬编码。
- 如何处理 design token 和 styling。
- 项目特定架构模式。

定义后，这些规则能减少重复提示，并让 Figma implementation task 保持一致。

## 前置条件

- Figma MCP server 已连接且可访问。
- 可以访问项目代码库做分析。
- 理解或愿意建立团队组件约定。

## 适用场景

- 新项目即将使用 Figma 设计。
- 让 AI coding agent 熟悉已有项目模式。
- 团队要标准化 Figma-to-code 工作流。
- 更新或细化已有 design system 约定。
- 用户明确说要创建 design system rules、Figma guidelines 或项目规则。

## 必需流程

按顺序执行，不要跳步。

### 第 1 步：运行 create_design_system_rules 工具

调用 Figma MCP server 的 `create_design_system_rules`，获取基础 prompt 和 template。

参数：

- `clientLanguages`: 项目语言列表，逗号分隔，例如 `"typescript,javascript"`、`"python"`。
- `clientFrameworks`: 框架，例如 `"react"`、`"vue"`、`"svelte"`、`"angular"`、`"unknown"`。

工具会返回创建规则的指导和模板。按工具返回的模板组织规则。

### 第 2 步：分析代码库

规则定稿前，分析已有模式。

组件组织：

- UI 组件位于哪里，例如 `src/components/`、`app/ui/`、`lib/components/`。
- 是否有专门 design system 目录。
- 组件按 feature、type 还是 flat structure 组织。

Styling：

- 使用 Tailwind、CSS Modules、styled-components 还是其他方案。
- design tokens 定义在哪里，例如 CSS variables、theme files、config files。
- 是否已有颜色、字体、spacing token。

组件模式：

- 命名约定，例如 PascalCase、kebab-case、prefix。
- props 通常如何组织。
- 是否有常见 composition pattern。

架构决策：

- 状态管理方式。
- routing system。
- import pattern 或 path alias。

### 第 3 步：生成项目专属规则

根据代码库分析，创建完整规则。包括：

#### 通用组件规则

```markdown
- IMPORTANT: Always use components from `[YOUR_PATH]` when possible
- Place new UI components in `[COMPONENT_DIRECTORY]`
- Follow `[NAMING_CONVENTION]` for component names
- Components must export as `[EXPORT_PATTERN]`
```

#### Styling 规则

```markdown
- Use `[CSS_FRAMEWORK/APPROACH]` for styling
- Design tokens are defined in `[TOKEN_LOCATION]`
- IMPORTANT: Never hardcode colors - always use tokens from `[TOKEN_FILE]`
- Spacing values must use the `[SPACING_SYSTEM]` scale
- Typography follows the scale defined in `[TYPOGRAPHY_LOCATION]`
```

#### Figma MCP 集成规则

```markdown
## Figma MCP Integration Rules

These rules define how to translate Figma inputs into code for this project and must be followed for every Figma-driven change.

### Required Flow (do not skip)

1. Run get_design_context first to fetch the structured representation for the exact node(s)
2. If the response is too large or truncated, run get_metadata to get the high-level node map, then re-fetch only the required node(s) with get_design_context
3. Run get_screenshot for a visual reference of the node variant being implemented
4. Only after you have both get_design_context and get_screenshot, download any assets needed and start implementation
5. Translate the output (usually React + Tailwind) into this project's conventions, styles, and framework
6. Validate against Figma for 1:1 look and behavior before marking complete
```

Implementation rules 应要求把 Figma MCP 输出当成设计和行为表示，而不是最终代码风格；把 Tailwind 转换为项目 styling 方案；复用已有组件；使用项目 token；遵守 routing、state 和 data-fetch 模式；对照 Figma screenshot 验证视觉和行为。

#### Asset handling

```markdown
## Asset Handling

- The Figma MCP server provides an assets endpoint which can serve image and SVG assets
- IMPORTANT: If the Figma MCP server returns a localhost source for an image or SVG, use that source directly
- IMPORTANT: DO NOT import/add new icon packages - all assets should be in the Figma payload
- IMPORTANT: DO NOT use or create placeholders if a localhost source is provided
- Store downloaded assets in `[ASSET_DIRECTORY]`
```

#### 项目专属约定

补充特殊架构模式、import 要求、测试要求、可访问性标准和性能考虑。

### 第 4 步：保存到正确规则文件

检测用户使用的 AI coding agent，并保存到对应文件：

| Agent | Rule File | Notes |
| --- | --- | --- |
| Claude Code | project root 的 `CLAUDE.md` | Markdown，可用 `.claude/rules/figma-design-system.md` 做模块化 |
| Codex CLI | project root 的 `AGENTS.md` | Markdown。若已存在，追加新 section。总大小限制 32 KiB |
| Cursor | `.cursor/rules/figma-design-system.mdc` | Markdown + YAML frontmatter |

不确定时，检查已有 rule file 或询问用户。

Cursor 规则要包 YAML frontmatter：

```markdown
---
description: Rules for implementing Figma designs using the Figma MCP server. Covers component organization, styling conventions, design tokens, asset handling, and the required Figma-to-code workflow.
globs: "src/components/**"
alwaysApply: false
---

[Generated rules here]
```

按项目实际目录调整 `globs`。

### 第 5 步：验证和迭代

创建规则后：

1. 用一个简单 Figma component implementation 测试。
2. 验证 agent 是否遵守规则。
3. 细化未生效规则。
4. 分享给团队反馈。
5. 项目演进时更新规则。

## 规则类别

至少覆盖：

- Component discovery：组件、feature components、layout primitives 的目录。
- Design token usage：颜色、spacing、typography token 来源，禁止硬编码。
- Component patterns：命名、props、export、composition。
- Asset handling：使用 Figma MCP localhost asset，不新增 icon package，不创建 placeholder。
- Testing and validation：实现后如何对照 Figma 和项目测试。
