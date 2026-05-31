# Local zh-CN Localization Review Handoff

## Scope

This is a local-only handoff for the Warp zh-CN localization candidate on
`feat/localization-settings-upstream-rebuild`. The user explicitly instructed
not to push, not to create or update a PR, and not to create a new branch.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Verified upstream base: `upstream/master` at
  `5767910b5e41bda196baaea041862e9505e46e20`
- Final upstream comparison after committing this handoff update:
  `git rev-list --left-right --count upstream/master...HEAD` returns `0 21`
- Final remote branch comparison after committing this handoff update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returns `3 95`
- Rebase state before this handoff update: no `.git/rebase-merge`,
  `.git/rebase-apply`, `MERGE_HEAD`, or `CHERRY_PICK_HEAD`
- Remote writes in this pass: none
- PR work in this pass: none

## Latest Upstream Delta

The current upstream tip is `5767910b Add ./script/format for customized cargo
fmt invocation. (#11747)`.

This upstream delta is broader than build/container files. It adds
`script/format`, updates CI, skills, docs, and presubmit scripts, and formats
many Rust files under `app/` and `crates/`. It affects app source, including
UI-adjacent source files, but it does not change bundled locale catalogs,
`crates/localization/src`, or the current local AI settings, AI context menu,
Workflows, and Voltron localization files.

## Change Summary

- Rebased the branch onto `upstream/master` at `5767910b`.
- Kept the existing `warp_localization` crate and bundled `en-US` / `zh-CN`
  catalogs as the app UI localization surface.
- Localized AI context menu categories through catalog keys.
- Localized Workflows category labels, empty state, accessibility copy, and the
  workflow search placeholder through app localization.
- Localized high-risk AI settings page strings, including permissions, usage,
  API key inputs, custom endpoint copy, active/input sections, fallback credit
  copy, AWS Bedrock copy, and related toggles.
- Added matching `en-US` and `zh-CN` catalog entries; final catalog parity is
  5733 keys in both catalogs, missing 0, extra 0, placeholder mismatches 0, and
  only `auth.empty` empty.
- Added a high-risk AI settings wrapper static test for multi-line wrapper
  calls that the previous line-oriented scan could miss.
- Removed an unused `LazyLock` import exposed by the current `warp` lib
  localization test target.

## Verification Matrix

The following commands were run locally on the current rebased worktree.

```bash
git fetch upstream master
git rebase --autostash upstream/master
```

Result: pass. The branch is now on top of
`5767910b5e41bda196baaea041862e9505e46e20`.

```bash
cargo fmt --all -- --check
```

Result: pass after the unused import fix.

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

Result: pass. Output summary: `en_count` 5733, `zh_count` 5733, `missing` 0,
`extra` 0, `placeholder_mismatch` 0, `empty` only `auth.empty`.

```bash
cargo test -p warp_localization -- --nocapture
```

Result: pass, 24 tests passed.

```bash
cargo check -p onboarding --message-format=short
```

Result: pass, finished in 3m 27s.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: pass after the final source edit, 8 tests passed with 4830 filtered out.
The final run rebuilt in 14m 38s and finished tests in 14.21s.

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

Artifact check: 8 PNG screenshots plus `recording.log`; all screenshots are
`1280x800`.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/video-recording-smoke-20260531T173639" \
  target/debug/integration test_video_recording
```

Result: pass, exit code 0. Current artifact directory:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56`

Artifact check: `after_bootstrap.png` and `after_commands.png` are `1280x800`.
`recording.mp4` is 259965 bytes and its MP4 `avc1` sample-entry dimensions are
`1280x800`.

```bash
cargo test -p warp_localization --test localization_tests direct_english -- --nocapture
```

Result: pass, 7 tests passed. This focused direct-English scan covers app,
onboarding, shared `ui_components`, context-chip tooltip, app-menu, selected
miscellaneous UI, and AI settings high-risk wrapper paths.

## Visual Evidence

Current zh-CN visual smoke evidence:

- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/recording.log`
- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/*.png`

Current video recording evidence:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.log`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_bootstrap.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_commands.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.mp4`

## Branch Notes

- No PR was created or updated.
- No branch was created.
- No remote refs were modified.
- Local `target/` artifacts are intentionally untracked build and visual
  evidence artifacts.

## Remaining Risk

- No external CI or external code review has been performed after this latest
  local validation.
- Visual coverage is sampled and not exhaustive across every platform surface,
  runtime configuration, and OS environment.
- This local evidence must not be described as full UI human review or complete
  product-wide exhaustive coverage.
