---
name: changelog-draft
description: 根据 release range 中合并的 PR 生成可审查 changelog draft。提取显式 CHANGELOG marker，分类未标记 PR，添加外部贡献者归属，并输出 markdown + JSON artifact。不会修改 channel_versions.json。
---

# Changelog Draft Generator

## 输入

| Parameter | Required | Description |
|-----------|----------|-------------|
| `channel` | yes | Release channel：`stable`、`preview` 或 `dev` |
| `release_tag` | yes | 要生成 changelog 的 release tag（例如 `v0.2026.05.06.09.12.stable_00`） |
| `output_dir` | no | 写入输出文件的目录。默认是 `$RUNNER_TEMP` 或 `/tmp/changelog-draft` |
| `attribution` | no | 归属模式：`external-only`（默认）、`all` 或 `none` |

## 工作流

### Step 1 — 确定 release range

推断用于比较的上一个 release **cut**。Release tag 遵循 `v0.YYYY.MM.DD.HH.MM.<channel>_NN` 模式，其中 `_NN` 是该 release cut 内的 RC/hotfix 编号。多个 tag 可以共享相同日期前缀（例如 `_00`、`_01`、`_02` 都属于同一个 release cut）。

Base tag 必须是**上一个** release cut（也就是不同日期）的 `_00` tag，而不只是上一个 tag。例如，如果要为 `v0.2026.04.29.08.57.stable_01` 生成 changelog，base 应为 `v0.2026.04.22.08.57.stable_00`，而不是 `v0.2026.04.29.08.57.stable_00`。

```bash
# 1. Extract the date prefix from the release_tag (everything before _NN)
release_date_prefix="${release_tag%_*}"

# 2. List all _00 tags for the channel (these are release cut points), sorted descending
git tag --list "v0.*.${channel}_00" --sort=-version:refname

# 3. Pick the first _00 tag whose date prefix differs from release_date_prefix
```

将 range 记录为 `previous_cut_tag..release_tag`。

### Step 2 — 获取 PR 数据

运行 `fetch_prs.py` script，收集 release range 中合并的全部 public-release PR，并提取显式 changelog marker。传入 workflow checkout 的 repository，不一定是 public repository。Release workflow 从 `warpdotdev/warp-internal` 运行，script 会在发出 JSON 前，确定性地将 `warp-repo-sync[bot]` PR 解析回其原始 public `warpdotdev/warp` PR metadata。从 `warpdotdev/warp-internal` 运行时，script 会有意省略并非 repo-sync bot 作者的 PR，因为这些是私有内部变更，不能暴露给 changelog agent 或生成的 artifact。

```bash
python3 .agents/skills/changelog-draft/scripts/fetch_prs.py \
  --repo "${GITHUB_REPOSITORY:-warpdotdev/warp}" \
  --base-ref <previous_tag> \
  --head-ref <release_tag>
```

script 向 stdout 输出如下结构的 JSON：

```json
{
  "range": { "base": "<previous_tag>", "head": "<release_tag>" },
  "prs": [
    {
      "number": 1234,
      "url": "https://github.com/warpdotdev/warp/pull/1234",
      "title": "...",
      "author": "username",
      "body": "...",
      "labels": ["..."],
      "merged_at": "2026-05-01T...",
      "explicit_entries": [
        { "category": "NEW-FEATURE", "text": "Added dark mode" }
      ],
      "linked_issues": [5678],
      "changed_files": ["app/src/ai/agent.rs", "crates/warp_features/src/lib.rs"],
      "source_repo": "warpdotdev/warp",
      "internal_pr": {
        "number": 25712,
        "url": "https://github.com/warpdotdev/warp-internal/pull/25712",
        "author": "warp-repo-sync[bot]",
        "title": "...",
        "repo": "warpdotdev/warp-internal"
      }
    }
  ]
}
```

使用顶层 `number`、`url`、`author`、`body`、`labels`、`changed_files` 和 `source_repo` 字段作为事实来源。`internal_pr` 仅用于审计，绝不能用于贡献者归属或面向用户的 changelog 链接。如果 `url` 为空，应在面向用户的 markdown 中省略 PR 链接，而不是合成链接。

### Step 3 — 分类贡献者

使用 Step 2 中唯一作者 login 运行 `classify_contributors.py` script：

```bash
python3 .agents/skills/changelog-draft/scripts/classify_contributors.py \
  --org warpdotdev \
  --authors author1,author2,author3
```

输出 JSON：

```json
{
  "internal": ["author1"],
  "external": ["author3"],
  "bot": ["author2"],
  "unknown": []
}
```

### Step 4 — 提取 feature flag

运行 `extract_feature_flags.py` script，获取当前 flag gate list：

```bash
python3 .agents/skills/changelog-draft/scripts/extract_feature_flags.py \
  --file crates/warp_features/src/lib.rs
```

输出 JSON：

```json
{
  "release_flags": ["Autoupdate", "Changelog", ...],
  "preview_flags": ["Orchestration", ...],
  "dogfood_flags": ["LogExpensiveFramesInSentry", ...]
}
```

### Step 5 — 获取 issue reporter

收集 Step 2 中所有唯一的 `linked_issues`，并获取每个 issue 的原始 reporter。传入 `--org`，让 script 检查 org membership 并自动过滤内部 reporter：

```bash
python3 .agents/skills/changelog-draft/scripts/fetch_issue_reporters.py \
  --repo warpdotdev/warp \
  --org warpdotdev \
  --issues 5678,9012
```

输出 JSON（只包含外部 reporter）：

```json
{
  "issue_reporters": [
    {
      "issue_number": 5678,
      "title": "Crash when opening large file",
      "reporter": "community-user",
      "reporter_url": "https://github.com/community-user",
      "url": "https://github.com/warpdotdev/warp/issues/5678"
    }
  ]
}
```

`--org` flag 会通过 GitHub API 检查每个 reporter 的 org membership，过滤内部成员，避免把他们错误归属为外部社区 reporter。这些 reporter 会在 changelog 的 "Community" section 中被致谢。
每当 markdown draft 致谢 PR author、contributor 或 issue reporter 时，请将 username 渲染为 GitHub profile link，例如 `[@username](https://github.com/username)`。

### Step 6 — 分类未标记 PR

对每个没有显式 `CHANGELOG-*` 条目的 PR，决定是否包含，以及归入哪个类别。

遵循 `.agents/skills/classify-changelog-pr/SKILL.md` 中的分类指南。

对每个未标记 PR，生成一个分类：

```json
{
  "pr_number": 1234,
  "include": true,
  "category": "IMPROVEMENT",
  "text": "Proposed changelog line",
  "confidence": "high",
  "rationale": "...",
  "feature_flag": null,
  "needs_review": false
}
```

**关键规则：**
- 只触及 CI、测试、文档或内部工具的 PR -> `include: false`
- dogfood-only feature flag 背后的 PR -> stable channel 中 `include: false`
- preview flag 背后的 PR -> stable 中 `include: false`，preview 中 `include: true`
- 不确定时，设置 `needs_review: true` 和 `confidence: "low"`
- Bot PR（dependabot、renovate 等）-> `include: false`

**Feature-flag detection：** 使用 Step 2 的 `changed_files` 列表检查 PR 是否触及 `crates/warp_features/src/lib.rs`，或标题/body 中是否引用 `FeatureFlag` variant。与 Step 4 的 flag list 交叉引用，以确定 channel 可见性。

**Unknown contributors：** `unknown` bucket 中的作者（由于 auth 导致 org membership 检查失败）应保守处理，不要将其归属为外部贡献者。在输出中记录它们以供人工验证。

### Step 7 — 组装 draft

将显式条目（Step 2）和推断条目（Step 6）合并成最终报告。按以下顺序按类别分组：

1. `NEW-FEATURE` — New Features
2. `IMPROVEMENT` — Improvements
3. `BUG-FIX` — Bug Fixes
4. `OZ` — Oz Updates

带 `CHANGELOG-NONE` 标记的 PR 是显式 opt out，绝不能出现在 changelog markdown 中。

创建 entry 时，从 normalized PR record 中复制 `pr_number`、`url`、`author`、`source_repo` 和 `internal_pr`。Release JSON converter 会直接使用 `url`；不要根据 PR number 发明 public PR URL。

### Step 8 — 写入输出文件

向 `output_dir` 写入两个文件：

**`changelog-draft.md`** — 可供人工审查的 markdown，可直接用于 Slack/Notion：

```markdown
# Changelog Draft
**Channel:** stable
**Range:** v0.2026.05.01... → v0.2026.05.06...
**Generated:** 2026-05-06T15:00:00Z

## New Features
- Added dark mode ([#1234](https://github.com/warpdotdev/warp/pull/1234)) — [@external-contributor](https://github.com/external-contributor) ✨

## Improvements
- Faster tab switching ([#1235](https://github.com/warpdotdev/warp/pull/1235))

## Bug Fixes
- Fixed crash on startup ([#1236](https://github.com/warpdotdev/warp/pull/1236))

## Oz Updates
- Improved agent memory ([#1237](https://github.com/warpdotdev/warp/pull/1237))

## Community
### Contributors
- [@contributor1](https://github.com/contributor1) — [#1234](https://github.com/warpdotdev/warp/pull/1234)  ✨

### Issue Reporters
Thanks to the community members who reported issues fixed in this release:
- [@reporter1](https://github.com/reporter1) — [#5678](https://github.com/warpdotdev/warp/issues/5678) "Crash when opening large file"
```

Markdown draft **不得**包含 "Needs Review" 或 "Skipped PRs" section；这些是内部细节，只应存在于 JSON audit artifact 中。

**`changelog-draft.json`** — 机器可读 audit artifact（仅内部使用）：

```json
{
  "channel": "stable",
  "range": { "base": "v0...", "head": "v0..." },
  "generated_at": "2026-05-06T15:00:00Z",
  "entries": [
    {
      "pr_number": 1234,
      "url": "https://github.com/warpdotdev/warp/pull/1234",
      "category": "NEW-FEATURE",
      "text": "Added dark mode",
      "source": "explicit",
      "author": "external-contributor",
      "is_external": true,
      "confidence": "high",
      "rationale": null,
      "feature_flag": null,
      "source_repo": "warpdotdev/warp",
      "internal_pr": null
    }
  ],
  "skipped": [...],
  "needs_review": [...],
  "issue_reporters": [...]
}
```

JSON artifact 会保留 `skipped`、`needs_review` 和 `issue_reporters` 供审计使用；range 中的每个 PR 都必须出现在 `entries`、`skipped` 或 `needs_review` 之一中。

### Step 9 — 生成 release-pipeline JSON

运行转换 script，从 audit artifact 确定性生成 `changelog-release.json`：

```bash
python3 .agents/skills/changelog-draft/scripts/convert_to_release_json.py \
  --input <output_dir>/changelog-draft.json \
  --output <output_dir>/changelog-release.json
```

这会生成 `create_release` workflow 用于 Slack 和应用内 "What's New" dialog 的扁平 JSON 结构。**不要**手动生成该文件，始终使用 script，以确保输出确定且一致。

## 约束

- **绝不**写入 `channel_versions.json` 或任何 production config 文件。
- **绝不**push commit、创建 branch 或打开 PR。
- 所有输出只写入 `output_dir`。
- Markdown draft 应可复制粘贴到 Slack 或 Notion 供审查。
- 保持 JSON artifact 足够完整以支持审计：range 中的每个 PR 都应出现在 `entries`、`skipped` 或 `needs_review` 之一中。

## 验证

生成输出后，验证：
1. Range 中每个 PR 都已计入（entries + skipped + needs_review = total PRs）。
2. 显式 marker entry 与 `fetch_prs.py` 提取结果一致（没有丢失 marker）。
3. 各 section 之间没有重复 PR number。
4. Markdown 渲染正常（没有坏链接或格式问题）。
