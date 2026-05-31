# zh-CN Localization Review Evidence

## Scope

This record captures the local rebase, fix, and validation state for the Warp
zh-CN localization work on branch `feat/localization-settings-upstream-rebuild`.
It is local-only: no PR, push, remote ref update, or new branch was created.

## Current Branch State

- Branch: `feat/localization-settings-upstream-rebuild`
- Verified upstream base: `upstream/master` at
  `5767910b5e41bda196baaea041862e9505e46e20`
- Upstream ancestry: `git merge-base --is-ancestor upstream/master HEAD`
  returned 0.
- Final upstream comparison after committing this evidence update:
  `git rev-list --left-right --count upstream/master...HEAD` returns `0 21`.
- Final remote branch comparison after committing this evidence update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returns `3 95`.
- Rebase state check before this evidence update: no `.git/rebase-merge`,
  `.git/rebase-apply`, `MERGE_HEAD`, or `CHERRY_PICK_HEAD`.

## Upstream Delta Classification

The latest upstream refresh moved from `21334d42` to `5767910b`.

`5767910b Add ./script/format for customized cargo fmt invocation. (#11747)`
is not only a build or container change. It adds `script/format`, updates CI,
skills, docs, and `script/presubmit`, and applies formatting changes across many
Rust files under `app/` and `crates/`. This affects app source, including some
UI-adjacent files, but it does not change `app/assets/bundled/locales/*.json`,
`crates/localization/src`, or the current zh-CN catalog source. It also does not
directly modify the current local files for AI settings, AI context menu,
Workflows categories, or Voltron placeholder localization.

## Current Fixes

- Localized AI context menu category labels through catalog keys.
- Localized Workflows category labels, empty state, accessibility copy, and the
  Voltron workflow search placeholder through app localization.
- Localized high-risk AI settings page labels, descriptions, section headers,
  API key inputs, custom endpoint copy, AWS Bedrock copy, permissions copy,
  usage copy, and related toggles through existing catalog-backed helpers.
- Added `en-US` and `zh-CN` catalog entries for the new AI context menu,
  Workflows, and AI settings strings.
- Added a regression scan for high-risk AI settings wrappers that the previous
  single-line direct-literal scan missed, including `build_sub_header`,
  `render_ai_setting_toggle`, `DropdownItem::new`, modal title paths, and related
  wrapper calls.
- Removed an unused `std::sync::LazyLock` import from `ai_page.rs` after the
  current `warp` lib localization test target exposed it as a warning.

## Catalog State

Current catalog parity command result:

- `en-US` keys: 5733
- `zh-CN` keys: 5733
- Missing in `zh-CN`: 0
- Extra in `zh-CN`: 0
- Placeholder mismatches: 0
- Empty values: only `auth.empty`

## Verification

Commands run locally on the current rebased worktree:

```bash
git fetch upstream master
git rebase --autostash upstream/master
```

Result: pass. The branch is on top of
`5767910b5e41bda196baaea041862e9505e46e20`. During this rebase, conflicts were
resolved in app settings/auth-secret localization paths while preserving the
local catalog-backed UI copy.

```bash
cargo fmt --all -- --check
```

Result: pass after removing the unused `LazyLock` import.

```bash
git diff --check
```

Result: pass.

```bash
node - <<'NODE'
const fs = require('fs');
const paths = ['app/assets/bundled/locales/en-US.json', 'app/assets/bundled/locales/zh-CN.json'];
const [en, zh] = paths.map((p) => JSON.parse(fs.readFileSync(p, 'utf8')));
const enKeys = Object.keys(en).sort();
const zhKeys = Object.keys(zh).sort();
const missing = enKeys.filter((k) => !(k in zh));
const extra = zhKeys.filter((k) => !(k in en));
const placeholderRe = /\{[A-Za-z0-9_]+\}/g;
const placeholders = (value) => [...String(value).matchAll(placeholderRe)].map((m) => m[0]).sort().join(',');
const placeholderMismatch = enKeys.filter((k) => k in zh && placeholders(en[k]) !== placeholders(zh[k]));
const empty = zhKeys.filter((k) => String(zh[k]).length === 0);
console.log(JSON.stringify({en_count: enKeys.length, zh_count: zhKeys.length, missing: missing.length, extra: extra.length, placeholder_mismatch: placeholderMismatch.length, empty}, null, 2));
if (missing.length || extra.length || placeholderMismatch.length || empty.some((k) => k !== 'auth.empty')) process.exit(1);
NODE
```

Result: pass, with `en_count` 5733, `zh_count` 5733, `missing` 0, `extra` 0,
`placeholder_mismatch` 0, and `empty` equal to `["auth.empty"]`.

```bash
cargo test -p warp_localization -- --nocapture
```

Result: pass after the final source edit, 24 tests passed, 0 failed, 0 ignored.

```bash
cargo check -p onboarding --message-format=short
```

Result: pass, finished in 3m 27s.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: pass after the final source edit, 8 tests passed, 0 failed, 4830
filtered out. The final rebuild finished in 14m 38s and the tests finished in
14.21s. The earlier run exposed the unused `LazyLock` import warning; the final
run completed without that warning.

```bash
cargo build -p integration --bin integration
```

Result: pass, finished in 56m 54s.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts-20260531T173639" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Result: pass, exit code 0. Current artifact directory:

- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22`

Generated files: 8 PNG screenshots plus `recording.log`. `sips -g pixelWidth -g
pixelHeight` reported `1280x800` for all 8 screenshots.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/video-recording-smoke-20260531T173639" \
  target/debug/integration test_video_recording
```

Result: pass, exit code 0. Current artifact directory:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56`

Generated files: `after_bootstrap.png`, `after_commands.png`, `recording.mp4`,
and `recording.log`. Both PNG files are `1280x800`. `recording.mp4` is 259965
bytes. A direct MP4 `avc1` sample-entry check reported dimensions `1280x800`.

```bash
cargo test -p warp_localization --test localization_tests direct_english -- --nocapture
```

Result: pass, 7 tests passed, 0 failed, 17 filtered out. This focused scan
covers app, onboarding, shared `ui_components`, context-chip tooltip, app-menu,
selected miscellaneous UI, and AI settings high-risk wrapper direct-English
literal checks.

## Visual Evidence

Current zh-CN visual smoke evidence:

- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/recording.log`
- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/*.png`

The recording log shows all smoke steps and assertions succeeded, including
settings language UI, terminal input, context chips, command search, Agent input
mode, command palette, workspace toast, and launch-config dialog focus.

Current video recording evidence:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.log`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_bootstrap.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_commands.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.mp4`

## Completion Audit

- Rebase to latest upstream: satisfied locally for `upstream/master` at
  `5767910b5e41bda196baaea041862e9505e46e20`.
- No push or PR update: satisfied.
- Catalog JSON parse and parity: satisfied, missing 0, extra 0, placeholder
  mismatches 0, and only `auth.empty` empty.
- Direct-English high-risk scan: satisfied by the filtered
  `warp_localization` direct-English test run.
- Required Rust checks and tests: satisfied by the command results above.
- Required integration build and smoke/video runs: satisfied by the command
  results and artifact checks above.

## Remaining Risk

- External CI and external code review have not been run in this pass.
- No PR evidence exists because the user explicitly instructed not to submit or
  update a PR.
- Visual coverage is sampled and does not prove exhaustive coverage of every UI
  surface, platform state, runtime configuration, or translated context.
- Local `target/` artifacts are build and review evidence, not tracked source
  files.
