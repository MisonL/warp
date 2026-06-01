# 简体中文本地化本地交接记录

## 范围

本文是 `feat/localization-settings-upstream-rebuild` 分支上 Warp 简体中文本地化候选版本的本地交接记录。本轮没有 push、没有更新 PR、没有更新远端引用，也没有创建新分支。

## 候选版本状态

- 当前本地分支：`feat/localization-settings-upstream-rebuild`
- 本地已 rebase 的 upstream 基线：`upstream/master`
  `a44b703060673b85d2641c051c53e4b6b1b00cc4`
- 本文档更新前的源码 HEAD：
  `ba75c70cdd4b2c3494030311bd2c49c924baf3d7`
  `Record zh-CN video smoke evidence`
- 本次文档提交后的预期 upstream 对比：
  `git rev-list --left-right --count upstream/master...HEAD` 返回 `0 25`
- 本次文档提交后的预期远端分支对比：
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  返回 `3 101`
- rebase 完成后状态：不存在 `.git/rebase-merge`、`.git/rebase-apply`、
  `MERGE_HEAD` 或 `CHERRY_PICK_HEAD`
- 本轮远端写入：无
- 本轮 PR 操作：无

## 最新上游变更

`git fetch upstream` 曾成功把 `upstream/master` 从
`debe6d8104aed38afd7db6dedba668f8f8246818` 更新到
`a44b703060673b85d2641c051c53e4b6b1b00cc4`。

新的本地 upstream 顶点是 `a44b7030 Fix team owner pill contrast (#11689)`。
它在本地化相关检查范围内只触及 `app/src/settings_view/teams_page.rs`，没有修改内置语言文案目录，也没有修改 `crates/localization/src`。

唯一 rebase 冲突位于 `app/src/settings_view/teams_page.rs`。冲突解决保留了上游 owner 状态标签对比度修复，并通过以下调用保留本地化后的 owner 状态标签：

```rust
teams_text(app, "settings.teams.status.owner")
```

后续远端最新性重试无法访问 GitHub。`git fetch upstream`、
`git ls-remote upstream refs/heads/master` 和
`git ls-remote origin refs/heads/feat/localization-settings-upstream-rebuild`
均返回 HTTP 403。当前最新的本地可用 upstream 仍是
`a44b703060673b85d2641c051c53e4b6b1b00cc4`。

## 改动摘要

- 分支已 rebase 到 `upstream/master` 的本地 `a44b7030`。
- 保留 `warp_localization` crate，以及内置 `en-US` / `zh-CN` 文案目录作为应用 UI 本地化承载面。
- AI context menu 分类已通过文案目录 key 本地化。
- Workflows 分类、空态、无障碍文案和 workflow search 占位提示已通过应用本地化处理。
- AI Settings 页面高风险字符串已本地化，包括权限、用量、API key 输入、custom endpoint、active/input 分区、fallback credit、AWS Bedrock 和相关开关。
- 已新增 AI Settings 高风险封装调用静态测试，覆盖此前逐行扫描可能漏掉的多行封装调用。
- onboarding callout 标题、正文、按钮、checkbox 标签和 prompt 文案已通过 `OnboardingCopy` 与文案目录 key 本地化。
- 已新增 onboarding callout field literal 和 `OnboardingQuery` prompt literal 的定向静态覆盖。
- 缺失 onboarding copy key 时会显式失败，不再返回空字符串。
- onboarding callout 修复后已重新生成 zh-CN 视觉冒烟测试证据文件。
- 本轮已重新生成视频录制冒烟测试证据文件。

## 文案目录状态

- `en-US` keys：5752
- `zh-CN` keys：5752
- `zh-CN` 缺失：0
- `zh-CN` 额外：0
- placeholder 不匹配：0
- 空值：仅 `auth.empty`

## 验证矩阵

以下命令均在当前 rebased worktree 本地执行。

```bash
git fetch upstream
git rebase upstream/master
```

结果：通过。分支已 rebase 到
`a44b703060673b85d2641c051c53e4b6b1b00cc4`；唯一的 `teams_page.rs` 冲突已按上文方式解决。

```bash
git fetch upstream
git ls-remote upstream refs/heads/master
git ls-remote origin refs/heads/feat/localization-settings-upstream-rebuild
```

结果：不计为远端最新性通过。三个命令均返回 GitHub HTTP 403。

```bash
cargo build -p integration --bin integration
```

结果：通过，用时 50m 36s。

```bash
ARTIFACT_DIR="$PWD/target/zh-cn-visual-artifacts-20260601T013621Z"
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$ARTIFACT_DIR" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

结果：通过。

当前 zh-CN 视觉冒烟测试证据：

- `target/zh-cn-visual-artifacts-20260601T013621Z/test_zh_cn_localization_visual_smoke/2026-06-01T09-36-36/recording.log`
- 同目录下 8 张截图：
  `agent-input-zh-cn.png`、`command-palette-zh-cn.png`、
  `command-search-zh-cn.png`、`context-chips-zh-cn.png`、
  `dialog-launch-config-zh-cn.png`、`settings-appearance-language-zh-cn.png`、
  `terminal-input-zh-cn.png`、`toast-zh-cn.png`
- 每张 PNG 均已通过 `sips -g pixelWidth -g pixelHeight` 确认为
  `2560x1600`

```bash
cargo fmt --all -- --check
```

结果：通过。

```bash
git diff --check
```

结果：通过。

```bash
node - <<'NODE'
const fs = require('fs');
const en = JSON.parse(fs.readFileSync('app/assets/bundled/locales/en-US.json', 'utf8'));
const zh = JSON.parse(fs.readFileSync('app/assets/bundled/locales/zh-CN.json', 'utf8'));
const enKeys = Object.keys(en).sort();
const zhKeys = Object.keys(zh).sort();
const missing = enKeys.filter(k => !(k in zh));
const extra = zhKeys.filter(k => !(k in en));
const ph = s => [...String(s).matchAll(/\{[A-Za-z0-9_]+\}/g)].map(m => m[0]).sort();
const placeholderMismatch = [];
for (const k of enKeys) {
  if (!(k in zh)) continue;
  const a = ph(en[k]).join(',');
  const b = ph(zh[k]).join(',');
  if (a !== b) placeholderMismatch.push({ key: k, en: a, zh: b });
}
const empty = enKeys.filter(k => String(en[k]).length === 0 || String(zh[k] ?? '').length === 0);
console.log(JSON.stringify({ enKeys: enKeys.length, zhKeys: zhKeys.length, missing: missing.length, extra: extra.length, placeholderMismatch: placeholderMismatch.length, empty }, null, 2));
if (missing.length || extra.length || placeholderMismatch.length) process.exit(1);
NODE
```

结果：通过。输出摘要：`enKeys` 5752，`zhKeys` 5752，`missing` 0，`extra` 0，`placeholderMismatch` 0，`empty` 仅 `auth.empty`。

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization -- --nocapture
```

结果：通过。25 tests passed，0 failed，0 ignored；`app_ui_calls_do_not_use_direct_english_literals` 已成功完成。

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p onboarding --message-format=short
```

结果：通过，用时 3m 08s。

一次默认 target 的 `cargo test -p warp_localization -- --nocapture` 曾在 `target/debug` 中编译约 12 分钟后停止，当时仍未进入测试执行。计入证据的是成功的
`CARGO_TARGET_DIR=target/localization-audit` 运行。

```bash
ARTIFACT_DIR="$PWD/target/video-recording-smoke-20260601T020835Z"
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$ARTIFACT_DIR" \
  target/debug/integration test_video_recording
```

结果：通过。

## 当前视频证据

当前视频录制证据：

- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/recording.log`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/after_bootstrap.png`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/after_commands.png`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/recording.mp4`

证据文件核查：

- `after_bootstrap.png`：`2560x1600`
- `after_commands.png`：`2560x1600`
- `recording.mp4`：425276 bytes
- 从 MP4 `tkhd` atom 解析到的视频轨尺寸：`2560x1600`

## 分支备注

- 未创建或更新 PR。
- 未创建新分支。
- 未修改远端引用。
- 本地 `target/` 证据文件是有意保留的构建与视觉证据，不纳入源码提交。

## 剩余风险

- 本轮最新本地验证之后，未执行外部 CI 或外部代码审查。
- 本地 `a44b7030` 之后可能存在更新的 upstream commit，但本轮 fetch 和 ls-remote 远端最新性检查被 GitHub HTTP 403 阻断。
- 视觉覆盖是采样覆盖，不能描述为完整产品级 UI 人审或穷尽平台覆盖。
