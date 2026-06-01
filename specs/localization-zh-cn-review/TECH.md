# 简体中文本地化技术证据

## 范围

本文记录 `feat/localization-settings-upstream-rebuild` 分支上 Warp 简体中文本地化工作的本地 rebase、审计、修复与验证状态。

这是本地证据文档。本轮没有 push、没有更新 PR、没有更新远端引用，也没有创建新分支。

## 当前分支状态

- 分支：`feat/localization-settings-upstream-rebuild`
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

## 上游变更分类

`git fetch upstream` 曾成功把 `upstream/master` 从
`debe6d8104aed38afd7db6dedba668f8f8246818` 更新到
`a44b703060673b85d2641c051c53e4b6b1b00cc4`。

新的本地 upstream 顶点为 `a44b7030 Fix team owner pill contrast (#11689)`。
该提交修改 `app/src/settings_view/teams_page.rs`，用于改善 owner 状态标签
的对比度。它没有修改内置语言文案目录，也没有修改
`crates/localization/src`。

本轮 rebase 的唯一冲突位于 `app/src/settings_view/teams_page.rs`。冲突解决保留了上游 owner 状态标签的对比度 helper 和 opacity 常量，同时保留本地化后的 owner 文案：

```rust
teams_text(app, "settings.teams.status.owner")
```

后续远端最新性重试无法访问 GitHub：

```text
git fetch upstream
fatal: unable to access 'https://github.com/warpdotdev/warp/': The requested URL returned error: 403

git ls-remote upstream refs/heads/master
fatal: unable to access 'https://github.com/warpdotdev/warp/': The requested URL returned error: 403

git ls-remote origin refs/heads/feat/localization-settings-upstream-rebuild
fatal: unable to access 'https://github.com/MisonL/warp/': The requested URL returned error: 403
```

因此当前最新的本地可用 upstream 仍是
`a44b703060673b85d2641c051c53e4b6b1b00cc4`。

## 已完成修复

- AI context menu 分类标签已接入文案目录 key。
- Workflows 分类标签、空态、无障碍文案和 Voltron workflow 搜索占位提示已接入应用本地化。
- AI Settings 页面高风险文案已本地化，包括标签、说明、分区标题、API key 输入、custom endpoint、AWS Bedrock、权限、用量和相关开关。
- 已为 AI context menu、Workflows 和 AI Settings 字符串补齐匹配的 `en-US` 与 `zh-CN` 文案目录条目。
- 已新增高风险 AI Settings 封装调用回归扫描，覆盖此前单行直接字符串扫描可能漏掉的 `build_sub_header`、`render_ai_setting_toggle`、`DropdownItem::new`、弹窗标题等路径。
- `ai_page.rs` 中由 `warp` lib 本地化测试目标暴露出的未使用 `std::sync::LazyLock` import 已删除。
- onboarding callout 的标题、正文、按钮、checkbox 标签、terminal prompt 占位提示和 agent prompt 占位提示已本地化。
- `TerminalView` 已把 `OnboardingCopy` 传入 `OnboardingCalloutView`，callout 文案通过 `terminal_text(ctx, key)` 解析，不再使用硬编码英文 UI 文案。
- 已新增 onboarding prompt 文案目录 key：
  `onboarding.callout.talk_to_agent.prompt`、
  `onboarding.callout.agent_prompt.placeholder`、
  `onboarding.callout.terminal_command.placeholder`。
- model 层 placeholder variant 已替换为 catalog-key variant；`git status`、`/init` 等命令文本保留为命令值，不作为 UI 文案翻译。
- `OnboardingCopy` 缺 key 时会通过 `default_text` 显式失败，不再静默返回空字符串。
- 已新增 onboarding callout field literal 和 `OnboardingQuery` prompt literal 的定向静态测试，覆盖此前封装调用扫描未覆盖的遗漏类别。

## 文案目录状态

当前文案目录一致性检查结果：

- `en-US` keys：5752
- `zh-CN` keys：5752
- `zh-CN` 缺失：0
- `zh-CN` 额外：0
- placeholder 不匹配：0
- 空值：仅 `auth.empty`

## 验证记录

以下命令均在当前 rebased worktree 本地执行。

```bash
git fetch upstream
git rebase upstream/master
```

结果：通过。分支已 rebase 到
`a44b703060673b85d2641c051c53e4b6b1b00cc4`。`app/src/settings_view/teams_page.rs`
冲突已按上文方式解决。

```bash
git fetch upstream
git ls-remote upstream refs/heads/master
git ls-remote origin refs/heads/feat/localization-settings-upstream-rebuild
```

结果：不计为远端最新性通过。三个命令均返回 GitHub HTTP 403，错误如上文记录。

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
- `sips -g pixelWidth -g pixelHeight` 已确认每张 PNG 均为 `2560x1600`

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

结果：通过。`enKeys` 为 5752，`zhKeys` 为 5752，`missing` 为 0，`extra` 为 0，`placeholderMismatch` 为 0，`empty` 为 `["auth.empty"]`。

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization -- --nocapture
```

结果：通过。复用隔离 target 后，25 个测试在 36.54s 内完成；25 passed，0 failed，0 ignored。最重的静态扫描 `app_ui_calls_do_not_use_direct_english_literals` 已成功完成。

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p onboarding --message-format=short
```

结果：通过，用时 3m 08s。

此前一次默认 target 命令：

```bash
cargo test -p warp_localization -- --nocapture
```

在 `target/debug` 中编译约 12 分钟后仍未进入测试执行，因此被手动终止。计入证据的是上方成功的隔离 target 测试。

```bash
ARTIFACT_DIR="$PWD/target/video-recording-smoke-20260601T020835Z"
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$ARTIFACT_DIR" \
  target/debug/integration test_video_recording
```

结果：通过。

## 当前视频证据

本轮生成的视频录制证据：

- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/recording.log`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/after_bootstrap.png`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/after_commands.png`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/recording.mp4`

证据文件核查：

- `after_bootstrap.png`：`2560x1600`
- `after_commands.png`：`2560x1600`
- `recording.mp4`：425276 bytes
- 从 MP4 `tkhd` atom 解析到的视频轨尺寸：`2560x1600`

之前 `target/video-recording-smoke-20260531T173639/` 中的视频证据仍然保留，但已被上方本轮证据文件取代。

## 完成度审计

- rebase 到最新本地可用 upstream：已满足，基线为
  `a44b703060673b85d2641c051c53e4b6b1b00cc4`。
- 超出本地 `a44b7030` 的远端最新性检查：未满足，因为 GitHub 对 fetch 和 ls-remote 返回 HTTP 403。
- 未 push、未更新 PR：已满足。
- 文案目录 JSON 解析与一致性检查：已满足，missing 0、extra 0、placeholder mismatch 0，且仅 `auth.empty` 为空。
- 直接英文高风险扫描：已由完整 `warp_localization` 测试覆盖。
- onboarding callout 遗漏：已修复，并由定向静态测试、文案目录一致性检查、onboarding 编译检查和刷新后的 zh-CN 视觉冒烟测试覆盖。
- 应用编译路径：已由 `cargo build -p integration --bin integration` 和成功执行的视觉冒烟测试 runner 覆盖。
- 视频录制冒烟测试：已由
  `target/video-recording-smoke-20260601T020835Z` 覆盖。

## 剩余风险

- 本轮未执行外部 CI，也未做外部代码审查。
- 本地 `a44b7030` 之后可能存在更新的 upstream commit，但本轮 fetch 和 ls-remote 远端最新性检查被 GitHub HTTP 403 阻断。
- 视觉覆盖是采样覆盖，不代表完整产品 UI 人审，也不代表穷尽所有平台状态、运行配置或翻译上下文。
- 本地 `target/` 证据文件是构建和审查证据，不纳入源码提交。
