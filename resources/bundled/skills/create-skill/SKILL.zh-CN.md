---
name: create-skill
description: 创建新技能、修改和优化现有技能，并衡量技能效果。适用于从零创建技能、编辑或优化现有技能、运行评测、做性能基准分析，或优化技能描述以提升触发准确率。
description_zh_CN: 创建新技能、修改和优化现有技能，并衡量技能效果。适用于从零创建技能、编辑或优化技能、运行评测、做性能基准分析，或优化技能描述以提升触发准确率。
license: Complete terms in LICENSE.txt
---

# Skill Creator

本技能用于创建新技能，并通过评测和用户反馈持续改进。

整体流程：

- 明确技能要做什么，以及大致该怎么做。
- 写出技能草稿。
- 创建若干真实测试 prompt，并让 agent 在有技能和无技能的条件下分别运行。
- 帮助用户从定性和定量两方面评估结果。
- 在后台运行时，补齐可量化评测；如果已有评测，就审阅并解释它们。
- 用 `eval-viewer/generate_review.py` 打开结果查看器，让用户查看输出和指标。
- 根据用户反馈和 benchmark 中暴露的问题修改技能。
- 重复直到结果满意。
- 扩大测试集，再做更大规模验证。

使用本技能时，先判断用户处在流程中的哪一步。如果用户只是说想为某事创建技能，就先帮助收敛目标、写草稿、写测试用例、确定评估方式并运行。若用户已有草稿，就直接进入评测和迭代阶段。若用户明确不想跑评测，只想一起打磨内容，可以按用户要求简化流程。

技能完成后，可以再运行 description improver，优化 frontmatter `description` 的触发准确率。

## 与用户沟通

本技能会被不同技术背景的用户使用。默认可以使用 "evaluation"、"benchmark" 这类词；使用 "JSON"、"assertion" 前要看用户是否理解。必要时用一句话解释术语。

## 创建技能

### 捕获意图

先理解用户目标。若当前会话已经包含要沉淀成技能的流程，就先从会话中提取工具、步骤、用户纠正、输入输出格式，再让用户确认。

需要明确：

1. 这个技能要让 agent 做什么。
2. 什么时候触发，包括用户常用说法和上下文。
3. 期望输出格式是什么。
4. 是否需要测试用例验证技能效果。可客观验证的技能应建议测试；偏主观的写作、风格、艺术类技能通常不必强行量化。

### 访谈和研究

主动询问边界条件、输入输出格式、示例文件、成功标准和依赖。不要在关键信息缺失时急着写测试 prompt。若 MCP 可用于研究文档或类似技能，可以使用；没有子 agent 时就内联完成。

### 编写 SKILL.md

根据访谈结果填写：

- `name`: 技能标识符。
- `description`: 触发机制的核心。必须包含技能做什么，以及什么时候使用。触发条件写在 description，不要只写在正文里。description 可以稍微积极一些，避免技能 undertrigger。
- `compatibility`: 可选，只有确实需要列出工具或依赖时才写。
- 正文说明。

### 技能结构

```text
skill-name/
├── SKILL.md
└── bundled resources
    ├── scripts/
    ├── references/
    └── assets/
```

技能有三层加载：

1. Metadata: name + description，始终在上下文中。
2. `SKILL.md` 正文，技能触发后加载。
3. 资源文件，按需读取或执行。

正文尽量保持在 500 行以内。接近上限时，把长文档拆到 `references/` 并在正文中清楚指向何时读取。多领域技能按变体组织引用文件，只读取当前任务相关部分。

### 安全边界

技能不得包含恶意、欺骗、未授权访问、数据外传或会让用户意外的内容。不要创建用于误导用户或协助攻击的技能。

### 写作方式

优先用祈使句和清晰解释。避免过度使用大写 MUST。说明为什么要这么做，让模型理解意图而不是机械遵循。

## 测试用例

草稿完成后，设计 2 到 3 个真实测试 prompt，并给用户确认。保存到 `evals/evals.json`。此时先写 prompt，不要急着写 assertion。

```json
{
  "skill_name": "example-skill",
  "evals": [
    {
      "id": 1,
      "prompt": "User's task prompt",
      "expected_output": "Description of expected result",
      "files": []
    }
  ]
}
```

完整 schema 见 `references/schemas.md`。

## 运行和评估测试

这是一个连续流程，不要中途停下。不要使用 `/skill-test` 或其他测试技能。

把结果放在与技能目录同级的 `<skill-name>-workspace/`。按 iteration 和 eval 分目录：

```text
<skill-name>-workspace/
└── iteration-1/
    └── eval-0/
        ├── with_skill/
        └── without_skill/
```

### 第 1 步：同时启动 with-skill 和 baseline

每个测试用例都要在同一轮里启动两个运行：

- with-skill: 使用当前技能。
- baseline: 新建技能时不使用技能；改进已有技能时使用旧版本快照。

为每个 eval 写 `eval_metadata.json`，目录名要描述测试点，不要只叫 `eval-0`。

```json
{
  "eval_id": 0,
  "eval_name": "descriptive-name-here",
  "prompt": "The user's task prompt",
  "assertions": []
}
```

### 第 2 步：运行期间编写 assertion

不要只是等待。利用运行时间编写客观、可验证、命名清晰的 assertion，并向用户解释它们检查什么。主观技能更适合定性评估，不要强行量化。

把 assertion 更新到 `eval_metadata.json` 和 `evals/evals.json`。

### 第 3 步：保存 timing

每个子 agent 完成时，通知里会包含 `total_tokens` 和 `duration_ms`。立即写入对应运行目录下的 `timing.json`：

```json
{
  "total_tokens": 84852,
  "duration_ms": 23332,
  "total_duration_seconds": 23.3
}
```

这是唯一能捕获这些数据的机会。

### 第 4 步：评分、聚合、打开查看器

1. 为每个运行评分。读取 `agents/grader.md`，用 grader 子 agent 或内联评分。`grading.json` 中 expectation 字段必须是 `text`、`passed`、`evidence`。
2. 聚合 benchmark：

```bash
python -m scripts.aggregate_benchmark <workspace>/iteration-N --skill-name <name>
```

3. 做 analyst pass，读取 benchmark，指出非区分性 assertion、高方差 eval、耗时或 token tradeoff 等。
4. 启动查看器：

```bash
nohup python <skill-creator-path>/eval-viewer/generate_review.py \
  <workspace>/iteration-N \
  --skill-name "my-skill" \
  --benchmark <workspace>/iteration-N/benchmark.json \
  > /dev/null 2>&1 &
VIEWER_PID=$!
```

第二轮及之后加 `--previous-workspace <workspace>/iteration-<N-1>`。无图形界面时用 `--static <output_path>` 写静态 HTML。

告诉用户已打开结果页面，并说明 `Outputs` 用于逐个查看和反馈，`Benchmark` 用于看量化比较。

### 第 5 步：读取反馈

用户完成 review 后，读取 `feedback.json`。空反馈表示对应结果可以接受。优先改有具体意见的测试用例。完成后关闭 viewer：

```bash
kill $VIEWER_PID 2>/dev/null
```

## 改进技能

改进时遵循：

- 从反馈中泛化，不要为少数测试样本过拟合。
- 保持 prompt 精炼，删除没有作用的指令。
- 解释为什么，而不是只堆叠强制规则。
- 如果多个测试都重复写同类脚本，把脚本沉淀到 `scripts/` 并让技能复用。

迭代流程：

1. 修改技能。
2. 在新的 `iteration-<N+1>/` 下重新跑所有测试和 baseline。
3. 带 `--previous-workspace` 打开 reviewer。
4. 等用户 review。
5. 读反馈并继续改。

停止条件：

- 用户满意。
- 反馈全空。
- 已无法取得有意义进展。

## 盲评比较

当用户要求严谨比较两个版本时，读取 `agents/comparator.md` 和 `agents/analyzer.md`。基本方式是让独立 agent 在不知道版本身份的情况下比较两个输出，并分析胜出原因。这是可选流程。

## Description 优化

技能完成后，可以优化 `description` 提升触发准确率。

1. 生成 20 条真实触发评测 query，包含 8 到 10 条 should-trigger 和 8 到 10 条 should-not-trigger。负例要选择接近但不该触发的场景。
2. 用 `assets/eval_review.html` 给用户 review eval set。
3. 保存 eval set 后运行：

```bash
python -m scripts.run_loop \
  --eval-set <path-to-trigger-eval.json> \
  --skill-path <path-to-skill> \
  --model <model-id-powering-this-session> \
  --max-iterations 5 \
  --verbose
```

4. 从结果 JSON 取 `best_description`，更新 frontmatter，并报告前后对比和分数。

## 打包

如果有 `present_files` 工具，可以运行：

```bash
python -m scripts.package_skill <path/to/skill-folder>
```

然后把生成的 `.skill` 文件路径交给用户。

## 更新已有技能

- 保留原始 `name` 和目录名。
- 安装路径可能只读，先复制到 `/tmp/skill-name/` 再编辑。
- 手动打包时先在 `/tmp/` staging，再复制到输出目录。

## 引用文件

- `agents/grader.md`: 如何按 assertion 评分。
- `agents/comparator.md`: 如何做盲评 A/B。
- `agents/analyzer.md`: 如何分析胜出原因。
- `references/schemas.md`: `evals.json`、`grading.json` 等结构。

核心循环：明确目标，写草稿或编辑技能，用测试 prompt 跑 agent，与用户一起评估输出和 benchmark，迭代到满意，打包返回。

如果可用 TodoList，请加入事项：创建 evals JSON 并运行 `eval-viewer/generate_review.py`，让用户 review 测试用例。
