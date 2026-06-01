---
name: figma-create-design-system-rules
description: 为用户的代码库生成自定义 design system rules。当用户说 "create design system rules"、"generate rules for my project"、"set up design rules"、"customize design system guidelines"，或希望为 Figma-to-code 工作流建立项目专属约定时使用。需要连接 Figma MCP server。
disable-model-invocation: false
---

# 创建 Design System Rules

## 概览

此 skill 帮助你生成适合项目具体需求的自定义 design system rules。这些规则会指导 AI coding agent 在实现 Figma design 时产出一致、高质量的代码，确保自动遵循团队约定、component pattern 和架构决策。

### 支持的规则文件

| Agent | 规则文件 |
|-------|-----------|
| Claude Code | `CLAUDE.md` |
| Codex CLI | `AGENTS.md` |
| Cursor | `.cursor/rules/figma-design-system.mdc` |

## 什么是 Design System Rules？

Design system rules 是项目级指令，用来编码代码库中的“未写明知识”，也就是有经验开发者知道并会传授给新团队成员的专业经验：

- 应使用哪些 layout primitive 和 component
- Component 文件应放在哪里
- Component 应如何命名和组织结构
- 哪些内容绝不能 hardcode
- 如何处理 design token 和 styling
- 项目专属架构 pattern

定义完成后，这些规则会显著减少重复提示，并确保所有 Figma implementation task 都输出一致结果。

## 前置条件

- Figma MCP server 必须已连接且可访问
- 需要访问项目代码库进行分析
- 理解团队的 component 约定，或愿意建立这些约定

## 何时使用此 skill

在以下情况使用此 skill：

- 启动一个会使用 Figma design 的新项目
- 将 AI coding agent onboarding 到已有成熟 pattern 的项目
- 在团队内标准化 Figma-to-code 工作流
- 更新或细化现有 design system 约定
- 用户明确请求："create design system rules"、"set up Figma guidelines"、"customize rules for my project"

## 必需工作流

**按顺序执行这些步骤。不要跳过步骤。**

### 步骤 1：运行 Create Design System Rules 工具

调用 Figma MCP server 的 `create_design_system_rules` 工具，获取基础 prompt 和 template。

**参数：**

- `clientLanguages`：项目中使用语言的逗号分隔列表（例如 "typescript,javascript"、"python"、"javascript"）
- `clientFrameworks`：使用的 framework（例如 "react"、"vue"、"svelte"、"angular"、"unknown"）

此工具会返回用于创建 design system rules 的指引和 template。

按照工具响应中提供的 template format 来组织 design system rules。

### 步骤 2：分析代码库

最终确定规则前，先分析项目以理解现有 pattern：

**Component Organization：**

- UI component 位于哪里？（例如 `src/components/`、`app/ui/`、`lib/components/`）
- 是否有专用 design system 目录？
- Component 如何组织？（按 feature、按 type、平铺结构）

**Styling Approach：**

- 使用什么 CSS framework 或方式？（Tailwind、CSS Modules、styled-components 等）
- Design token 在哪里定义？（CSS variable、theme file、config file）
- 是否已有 color、typography 或 spacing token？

**Component Patterns：**

- 使用哪些命名约定？（PascalCase、kebab-case、prefix）
- Component props 通常如何组织？
- 是否有常见 composition pattern？

**Architecture Decisions：**

- 如何处理 state management？
- 使用什么 routing system？
- 是否有特定 import pattern 或 path alias？

### 步骤 3：生成项目专属规则

基于代码库分析，创建一套完整规则。包括：

#### 通用 Component Rules

```markdown
- IMPORTANT: Always use components from `[YOUR_PATH]` when possible
- Place new UI components in `[COMPONENT_DIRECTORY]`
- Follow `[NAMING_CONVENTION]` for component names
- Components must export as `[EXPORT_PATTERN]`
```

#### Styling Rules

```markdown
- Use `[CSS_FRAMEWORK/APPROACH]` for styling
- Design tokens are defined in `[TOKEN_LOCATION]`
- IMPORTANT: Never hardcode colors - always use tokens from `[TOKEN_FILE]`
- Spacing values must use the `[SPACING_SYSTEM]` scale
- Typography follows the scale defined in `[TYPOGRAPHY_LOCATION]`
```

#### Figma MCP Integration Rules

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

### Implementation Rules

- Treat the Figma MCP output (React + Tailwind) as a representation of design and behavior, not as final code style
- Replace Tailwind utility classes with `[YOUR_STYLING_APPROACH]` when applicable
- Reuse existing components from `[COMPONENT_PATH]` instead of duplicating functionality
- Use the project's color system, typography scale, and spacing tokens consistently
- Respect existing routing, state management, and data-fetch patterns
- Strive for 1:1 visual parity with the Figma design
- Validate the final UI against the Figma screenshot for both look and behavior
```

#### Asset Handling Rules

```markdown
## Asset Handling

- The Figma MCP server provides an assets endpoint which can serve image and SVG assets
- IMPORTANT: If the Figma MCP server returns a localhost source for an image or SVG, use that source directly
- IMPORTANT: DO NOT import/add new icon packages - all assets should be in the Figma payload
- IMPORTANT: DO NOT use or create placeholders if a localhost source is provided
- Store downloaded assets in `[ASSET_DIRECTORY]`
```

#### Project-Specific Conventions

```markdown
## Project-Specific Conventions

- [Add any unique architectural patterns]
- [Add any special import requirements]
- [Add any testing requirements]
- [Add any accessibility standards]
- [Add any performance considerations]
```

### 步骤 4：将规则保存到合适的规则文件

检测用户正在使用哪个 AI coding agent，并将生成的规则保存到对应文件：

| Agent | 规则文件 | 说明 |
|-------|-----------|-------|
| Claude Code | project root 中的 `CLAUDE.md` | Markdown format。也可以使用 `.claude/rules/figma-design-system.md` 做模块化组织。 |
| Codex CLI | project root 中的 `AGENTS.md` | Markdown format。如果文件已存在，追加为新 section。合并大小限制为 32 KiB。 |
| Cursor | `.cursor/rules/figma-design-system.mdc` | 带 YAML frontmatter 的 Markdown（`description`、`globs`、`alwaysApply`）。 |

如果不确定用户正在使用哪个 agent，检查项目中是否已有规则文件，或询问用户。

对于 Cursor，用 YAML frontmatter 包裹规则：

```markdown
---
description: Rules for implementing Figma designs using the Figma MCP server. Covers component organization, styling conventions, design tokens, asset handling, and the required Figma-to-code workflow.
globs: "src/components/**"
alwaysApply: false
---

[Generated rules here]
```

自定义 `globs` pattern，使其匹配项目中承载 Figma-derived code 的目录（例如 `"src/**/*.tsx"` 或 `["src/components/**", "src/pages/**"]`）。

保存后，规则会由 agent 自动加载，并应用到所有 Figma implementation task。

### 步骤 5：验证并迭代

创建规则后：

1. 用一个简单 Figma component implementation 进行测试
2. 验证 agent 是否正确遵循规则
3. 细化任何未按预期工作的规则
4. 分享给团队成员获取反馈
5. 随项目演进更新规则

## 规则类别和示例

### Essential Rules（始终包含）

**Component Discovery：**

```markdown
- UI components are located in `src/components/ui/`
- Feature components are in `src/components/features/`
- Layout primitives are in `src/components/layout/`
```

**Design Token Usage：**

```markdown
- Colors are defined as CSS variables in `src/styles/tokens.css`
- Never hardcode hex colors - use `var(--color-*)` tokens
- Spacing uses the 4px base scale: `--space-1` (4px), `--space-2` (8px), etc.
```

**Styling Approach：**

```markdown
- Use Tailwind utility classes for styling
- Custom styles go in component-level CSS modules
- Theme customization is in `tailwind.config.js`
```

### Recommended Rules（高价值）

**Component Patterns：**

```markdown
- All components must accept a `className` prop for composition
- Variant props should use union types: `variant: 'primary' | 'secondary'`
- Icon components should accept `size` and `color` props
```

**Import Conventions：**

```markdown
- Use path aliases: `@/components`, `@/styles`, `@/utils`
- Group imports: React, third-party, internal, types
- No relative imports beyond parent directory
```

**Code Quality：**

```markdown
- Add JSDoc comments for exported components
- Include PropTypes or TypeScript types for all props
- Extract magic numbers to named constants
```

### Optional Rules（项目专属）

**Accessibility：**

```markdown
- All interactive elements must have aria-labels
- Color contrast must meet WCAG AA standards
- Keyboard navigation required for all interactions
```

**Performance：**

```markdown
- Lazy load images with the `Image` component from `@/components/Image`
- Use React.memo for components that receive complex props
- Icons should be SVG components, not icon fonts
```

**Testing：**

```markdown
- Include unit tests for new components in `__tests__/` directory
- Use Testing Library queries (getByRole, getByLabelText)
- Test all interactive states and variants
```

## 示例

### 示例 1：React + Tailwind 项目

用户说："Create design system rules for my React project"

**操作：**

1. 运行 `create_design_system_rules(clientLanguages="typescript,javascript", clientFrameworks="react")`
2. 分析代码库结构
3. 生成规则：

```markdown
# Figma MCP Integration Rules

## Component Organization

- UI components are in `src/components/ui/`
- Page components are in `src/app/`
- Use Tailwind for styling

## Figma Implementation Flow

1. Run get_design_context for the node
2. Run get_screenshot for visual reference
3. Map Figma colors to Tailwind colors defined in `tailwind.config.js`
4. Reuse components from `src/components/ui/` when possible
5. Validate against screenshot before completing

## Styling Rules

- IMPORTANT: Use Tailwind utility classes, not inline styles
- Colors are defined in `tailwind.config.js` theme.colors
- Spacing uses Tailwind's default scale
- Custom components go in `src/components/ui/`

## Asset Rules

- IMPORTANT: Use localhost sources from Figma MCP server directly
- Store static assets in `public/assets/`
- DO NOT install new icon libraries
```

4. 保存到适合用户 agent 的规则文件
5. 用一个简单 button implementation 测试

**结果：** agent 现在会在实现 Figma design 时持续遵循项目约定。

### 示例 2：Vue + Custom CSS 项目

用户说："Set up Figma rules for my Vue app"

**操作：**

1. 运行 `create_design_system_rules(clientLanguages="typescript,javascript", clientFrameworks="vue")`
2. 发现项目使用 CSS modules 和自定义 design token system
3. 生成规则：

```markdown
# Figma MCP Integration Rules

## Component Organization

- Components are in `src/components/`
- Composables are in `src/composables/`
- Vue SFC structure: <script setup>, <template>, <style scoped>

## Design Tokens

- IMPORTANT: All colors defined in `src/styles/tokens.css` as CSS variables
- Use `var(--color-primary)`, `var(--color-secondary)`, etc.
- Spacing: `var(--space-xs)` through `var(--space-xl)`
- Typography: `var(--text-sm)` through `var(--text-2xl)`

## Figma Implementation Flow

1. Run get_design_context and get_screenshot
2. Translate React output to Vue 3 Composition API
3. Map Figma colors to CSS variables in `src/styles/tokens.css`
4. Use CSS Modules for component styles
5. Check for existing components in `src/components/` before creating new ones

## Styling Rules

- Use CSS Modules (`.module.css` files)
- IMPORTANT: Reference design tokens, never hardcode values
- Scoped styles with CSS modules
```

4. 保存到适合用户 agent 的规则文件
5. 用 card component 验证

**结果：** agent 现在会使用项目的 design token system，正确将 Figma design 转换为 Vue component。

### 示例 3：Design System 团队

用户说："Generate rules for our design system library"

**操作：**

1. 运行 `create_design_system_rules(clientLanguages="typescript", clientFrameworks="react")`
2. 分析带 design system package 的 monorepo 结构
3. 生成完整规则：

```markdown
# Design System Rules

## Repository Structure

- Design system components: `packages/design-system/src/components/`
- Documentation: `packages/docs/`
- Design tokens: `packages/tokens/src/`

## Component Development

- IMPORTANT: All components must be in `packages/design-system/src/components/`
- Component file structure: `ComponentName/index.tsx`, `ComponentName.stories.tsx`, `ComponentName.test.tsx`
- Export all components from `packages/design-system/src/index.ts`

## Design Tokens

- Colors: `packages/tokens/src/colors.ts`
- Typography: `packages/tokens/src/typography.ts`
- Spacing: `packages/tokens/src/spacing.ts`
- IMPORTANT: Never hardcode values - import from tokens package

## Documentation Requirements

- Add Storybook story for every component
- Include JSDoc with @example
- Document all props with descriptions
- Add accessibility notes

## Figma Integration

1. Get design context and screenshot from Figma
2. Map Figma tokens to design system tokens
3. Create or extend component in design system package
4. Add Storybook stories showing all variants
5. Validate against Figma screenshot
6. Update documentation
```

4. 保存到合适的规则文件并分享给团队
5. 添加到团队文档

**结果：** 整个团队在把 Figma 中的 component 加入 design system 时，会遵循一致 pattern。

## 最佳实践

### 从简单开始，逐步迭代

不要试图一开始就捕获所有规则。从最重要的约定开始，并在遇到不一致时添加规则。

### 具体明确

不要写："Use the design system"
应写："Always use Button components from `src/components/ui/Button.tsx` with variant prop ('primary' | 'secondary' | 'ghost')"

### 让规则可执行

每条规则都应明确告诉 agent 要做什么，而不只是说明要避免什么。

好："Colors are defined in `src/theme/colors.ts` - import and use these constants"
不好："Don't hardcode colors"

### 对关键规则使用 IMPORTANT

对绝不能违反的规则添加 "IMPORTANT:" 前缀，确保 agent 优先处理。

```markdown
- IMPORTANT: Never expose API keys in client-side code
- IMPORTANT: Always sanitize user input before rendering
```

### 记录原因

当规则看起来有些武断时，说明理由：

```markdown
- Place all data-fetching in server components (reduces client bundle size and improves performance)
- Use absolute imports with `@/` alias (makes refactoring easier and prevents broken relative paths)
```

## 常见问题与解决方案

### 问题：agent 没有遵循规则

**原因：** 规则可能过于模糊，或未被 agent 正确加载。
**解决方案：**

- 让规则更具体、更可执行
- 验证规则已保存到正确配置文件
- 重启 agent 或 IDE 以重新加载规则
- 对关键规则添加 "IMPORTANT:" 前缀

### 问题：规则互相冲突

**原因：** 规则矛盾或重叠。
**解决方案：**

- 检查所有规则中的冲突
- 建立清晰的优先级层次
- 移除冗余规则
- 将相关规则合并为单条清晰陈述

### 问题：规则太多导致延迟增加

**原因：** 过多规则会增加 context size 和处理时间。
**解决方案：**

- 聚焦能解决 80% 一致性问题的 20% 规则
- 移除很少适用的过度具体规则
- 合并相关规则
- 使用 progressive disclosure（基础规则优先，高级规则放入链接文件）

### 问题：规则随项目演进而过时

**原因：** 代码库发生变化，但规则没有更新。
**解决方案：**

- 定期安排规则审查（每月或每季度）
- 架构决策变化时更新规则
- 将规则文件纳入版本控制
- 在 commit message 中记录规则变更

## 理解 Design System Rules

Design system rules 会改变 AI coding agent 处理 Figma design 的方式：

**没有规则之前：**

- agent 会假设 component 结构
- 各实现中的 styling approach 不一致
- hardcoded value 与 design token 不匹配
- Component 被创建到随机位置
- 需要重复解释项目约定

**有规则之后：**

- agent 自动遵循你的约定
- Component 结构和 styling 保持一致
- 从一开始就正确使用 design token
- Component 组织正确
- 无需重复提示

投入时间创建高质量规则，会在每个 Figma implementation task 中获得成倍回报。

## 其他资源

- [Figma MCP Server Documentation](https://developers.figma.com/docs/figma-mcp-server/)
- [Figma Variables and Design Tokens](https://help.figma.com/hc/en-us/articles/15339657135383-Guide-to-variables-in-Figma)
