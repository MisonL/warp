---
name: claude-api
description: "构建、调试和优化 Claude API / Anthropic SDK 应用。应用应包含 prompt caching。也适用于 Claude 模型版本迁移、Managed Agents、工具调用、批处理、文件、引用、记忆和相关 SDK 功能。跳过 OpenAI 或其他 provider 的通用代码。"
description_zh_CN: "构建、调试和优化 Claude API / Anthropic SDK 应用。适用于 Claude API、Anthropic SDK、Managed Agents、Claude 功能或模型迁移相关任务；不适用于 OpenAI 或其他 provider 的通用代码。"
license: Complete terms in LICENSE.txt
---

# 使用 Claude 构建 LLM 应用

本技能用于构建基于 Claude 的 LLM 应用。先判断要使用的 API surface 和项目语言，再读取对应语言目录下的文档与示例。

## 开始前

先扫描目标文件、用户提示和项目上下文，确认它不是其他 provider 的代码。出现以下信号时要停止并说明此技能只生成 Claude/Anthropic SDK 代码，询问用户是要切换到 Claude，还是要一个非 Claude 实现：

- `import openai`、`from openai`、`langchain_openai`、`OpenAI(`
- `gpt-4`、`gpt-5`
- 文件名类似 `agent-openai.py` 或 `*-generic.py`
- 用户明确要求保持 provider-neutral

不要把 Anthropic SDK 调用写进非 Anthropic 文件。

## 输出要求

当用户要求添加、修改或实现 Claude 功能时，代码必须通过以下二者之一调用 Claude：

1. 项目语言的官方 Anthropic SDK，例如 `anthropic`、`@anthropic-ai/sdk`、`com.anthropic.*`。只要该语言有官方 SDK，这就是默认选择。
2. Raw HTTP，例如 `curl`、`requests`、`fetch`、`httpx`。仅当用户明确要求 cURL、REST、raw HTTP，项目本身是 shell/cURL 项目，或该语言没有官方 SDK 时使用。

不要混用 SDK 和 raw HTTP。不要因为 Python 或 TypeScript 项目里 raw HTTP 看起来更轻，就绕过官方 SDK。不要退回 OpenAI-compatible shim。

不要猜 SDK 用法。函数名、类名、namespace、方法签名和 import path 必须来自明确文档：本技能的 `{lang}/` 文件，或 `shared/live-sources.md` 中列出的官方 SDK 仓库和文档。若所需 binding 没有被明确记录，先从 `shared/live-sources.md` 抓取对应 SDK 仓库或文档，再写代码。不要从 cURL 形状或其他语言 SDK 推断 Ruby、Java、Go、PHP、C# API。

## 默认值

除非用户另有要求：

- 默认模型使用 `claude-opus-4-7`。
- 任何稍复杂的任务都默认启用 adaptive thinking：`thinking: {type: "adaptive"}`。
- 长输入、长输出或高 `max_tokens` 请求默认使用 streaming，避免请求超时。
- 如果不需要处理逐个 stream event，使用 SDK 的 `.get_final_message()` 或 `.finalMessage()` 获取完整结果。

## 子命令

如果用户请求本身就是一个裸子命令字符串，搜索本文所有 Subcommands 表，并直接执行匹配的 Action。没有匹配时按普通自然语言请求处理。

## 语言识别

在读取示例前，先判断项目语言：

- `*.py`、`requirements.txt`、`pyproject.toml`、`setup.py`、`Pipfile`: Python，读 `python/`
- `*.ts`、`*.tsx`、`package.json`、`tsconfig.json`: TypeScript，读 `typescript/`
- `*.js`、`*.jsx` 且无 TypeScript 文件: TypeScript，JS 使用同一个 SDK
- `*.java`、`pom.xml`、`build.gradle`: Java，读 `java/`
- `*.kt`、`*.kts`、`build.gradle.kts`: Java，Kotlin 使用 Java SDK
- `*.scala`、`build.sbt`: Java，Scala 使用 Java SDK
- `*.go`、`go.mod`: Go，读 `go/`
- `*.rb`、`Gemfile`: Ruby，读 `ruby/`
- `*.cs`、`*.csproj`: C#，读 `csharp/`
- `*.php`、`composer.json`: PHP，读 `php/`

如果检测到多种语言，优先看用户当前文件或问题所指语言；仍不明确时询问。无法判断语言时，如果可以使用 AskUserQuestion，就给 Python、TypeScript、Java、Go、Ruby、cURL/raw HTTP、C#、PHP 选项；否则默认给 Python 示例并说明可按需切换。遇到 Rust、Swift、C++、Elixir 等不支持官方 SDK 的语言，建议使用 `curl/` 中的 raw HTTP 示例，并可提供 Python 或 TypeScript 参考实现。

## 功能支持

| 语言 | Tool Runner | Managed Agents | 备注 |
| --- | --- | --- | --- |
| Python | Yes (beta) | Yes (beta) | `@beta_tool` decorator |
| TypeScript | Yes (beta) | Yes (beta) | `betaZodTool` + Zod |
| Java | Yes (beta) | Yes (beta) | annotated classes |
| Go | Yes (beta) | Yes (beta) | `BetaToolRunner` in `toolrunner` |
| Ruby | Yes (beta) | Yes (beta) | `BaseTool` + beta `tool_runner` |
| C# | No | No | 官方 SDK |
| PHP | Yes (beta) | Yes (beta) | `BetaRunnableTool` + `toolRunner()` |
| cURL | N/A | Yes (beta) | Raw HTTP |

Managed Agents 示例在 Python、TypeScript、Go、Ruby、PHP、Java 和 cURL 下都有专门 README。还要阅读 `shared/managed-agents-*.md` 中的语言无关概念文档。Agent 是持久对象：创建一次，保存 `agents.create` 返回的 agent ID，后续每次 `sessions.create` 都引用它。不要在请求路径里重复创建 agent。若语言 README 没有展示所需 binding，从 `shared/live-sources.md` 抓取官方来源，不要猜。

## 选择哪个 API surface

从满足需求的最简单层级开始。

| 使用场景 | 层级 | 推荐 surface | 原因 |
| --- | --- | --- | --- |
| 分类、摘要、抽取、问答 | 单次 LLM 调用 | Claude API | 一请求一响应 |
| 批处理或 embeddings | 单次 LLM 调用 | Claude API | 专用端点 |
| 代码控制的多步 pipeline | Workflow | Claude API + tool use | 应用自己编排循环 |
| 自定义 agent 和自有工具 | Agent | Claude API + tool use | 灵活性最高 |
| Anthropic 托管状态和执行环境 | Agent | Managed Agents | 服务端会话、workspace、工具执行 |

Managed Agents 只适合一方 Anthropic 平台。部署到 Amazon Bedrock、Google Vertex AI 或 Microsoft Foundry 时，Managed Agents 不可用，应使用 Claude API + tool use。

选择 agent 前检查四点：

- 复杂度：任务是否多步且难以完全预先规定。
- 价值：结果是否值得更高成本和延迟。
- 可行性：Claude 是否适合该任务类型。
- 出错成本：错误是否能被测试、review 或 rollback 捕获。

任一点为否，就留在更简单的层级。

## 架构

核心请求都走 `POST /v1/messages`。Tools 和输出约束都是该端点的功能，不是独立 API。

- User-defined tools: 由你定义工具，SDK tool runner 负责调用 API、执行函数并循环到完成；也可以手写循环。
- Server-side tools: Anthropic 托管工具在 Anthropic 基础设施中执行。
- Structured outputs: 使用 `output_config.format` 或工具参数校验；推荐 `client.messages.parse()` 自动验证 schema。旧的 `output_format` 已弃用。
- Supporting endpoints: Batches、Files、Token Counting、Models 用于支撑 Messages API。

## 当前模型缓存

缓存日期：2026-04-15。

| Model | Model ID | Context | Input $/1M | Output $/1M |
| --- | --- | --- | --- | --- |
| Claude Opus 4.7 | `claude-opus-4-7` | 1M | $5.00 | $25.00 |
| Claude Opus 4.6 | `claude-opus-4-6` | 1M | $5.00 | $25.00 |
| Claude Sonnet 4.6 | `claude-sonnet-4-6` | 1M | $3.00 | $15.00 |
| Claude Haiku 4.5 | `claude-haiku-4-5` | 200K | $1.00 | $5.00 |

始终使用完整模型 ID，不要追加日期后缀。除非用户明确点名其他模型，默认使用 `claude-opus-4-7`。如果用户询问某模型上下文窗口或能力，查询 Models API；不要依赖缓存表。

## Thinking、Effort、Compaction

- Opus 4.7 只支持 adaptive thinking：`thinking: {type: "adaptive"}`。`budget_tokens` 会 400。
- Opus 4.6 和 Sonnet 4.6 推荐 adaptive thinking；新代码不要使用 `budget_tokens`。
- Effort 放在 `output_config` 下，例如 `output_config: {effort: "high"}`，可选 `low`、`medium`、`high`、`max`，Opus 4.7 还支持 `xhigh`。
- Opus 4.7 默认省略 thinking 文本。若要向用户显示摘要，设置 `thinking: {type: "adaptive", display: "summarized"}`。
- 长对话可启用服务端 compaction，beta header 为 `compact-2026-01-12`。每轮必须把完整 `response.content` 追加回 messages，不能只追加 text，否则会丢失 compaction 状态。

## Prompt Caching

缓存按前缀匹配。任何 byte 变化都会让变化点之后的缓存失效。渲染顺序是 `tools`、`system`、`messages`。把稳定内容放前面，把时间戳、请求 ID、用户问题等易变内容放在最后一个 cache breakpoint 之后。

最简单方式是顶层 `cache_control: {type: "ephemeral"}`。每个请求最多 4 个 breakpoint。用 `usage.cache_read_input_tokens` 验证缓存命中；如果重复请求仍为 0，检查 system prompt 中的时间、未排序 JSON、可变工具列表等静默失效因素。

## Managed Agents

Managed Agents 是第三种 surface：服务端托管的有状态 agent。流程是 Agent 一次创建，Session 每次运行。`model`、`system`、`tools` 属于 agent，不属于 session。每个 session 会分配容器 workspace，bash、文件操作和代码执行都在其中运行，事件通过 stream 返回。

Beta header：`managed-agents-2026-04-01`。SDK 对 `client.beta.{agents,environments,sessions,vaults}.*` 调用会自动设置。

Subcommands：

| Subcommand | Action |
| --- | --- |
| `managed-agents-onboard` | 立即阅读 `shared/managed-agents-onboarding.md`，按访谈脚本引导用户完成从零创建 Managed Agent。 |

当用户要从零设置 Managed Agent 时，读取 onboarding 文档并执行访谈，不要只总结。
