# Local zh-CN Localization Review Handoff

## Scope

This is a local-only handoff for the Warp zh-CN localization candidate on
`feat/localization-settings-upstream-rebuild`. No push, PR update, remote ref
update, or new branch was created.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Rebased upstream base: `upstream/master` at
  `a44b703060673b85d2641c051c53e4b6b1b00cc4`
- Source HEAD before this documentation-only handoff update:
  `ea36256c92e19464b6ddff70a266a77d5de24d2a`
  (`Localize onboarding callout copy`)
- Expected upstream comparison after committing this handoff update:
  `git rev-list --left-right --count upstream/master...HEAD` returns `0 23`
- Expected remote branch comparison after committing this handoff update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returns `3 99`
- Rebase state after the rebase: no `.git/rebase-merge`,
  `.git/rebase-apply`, `MERGE_HEAD`, or `CHERRY_PICK_HEAD`
- Remote writes in this pass: none
- PR work in this pass: none

## Latest Upstream Delta

`git fetch upstream` succeeded and advanced `upstream/master` from
`debe6d8104aed38afd7db6dedba668f8f8246818` to
`a44b703060673b85d2641c051c53e4b6b1b00cc4`.

The new upstream tip is `a44b7030 Fix team owner pill contrast (#11689)`. It
only touched `app/src/settings_view/teams_page.rs` in the localization-relevant
surface checked here, and did not modify bundled locale catalogs or
`crates/localization/src`.

The only rebase conflict was in `app/src/settings_view/teams_page.rs`. The
resolution kept the upstream owner-chip contrast fix and retained the localized
owner status label through:

```rust
teams_text(app, "settings.teams.status.owner")
```

## Change Summary

- Rebased the branch onto `upstream/master` at `a44b7030`.
- Kept the existing `warp_localization` crate and bundled `en-US` / `zh-CN`
  catalogs as the app UI localization surface.
- Localized AI context menu categories through catalog keys.
- Localized Workflows category labels, empty state, accessibility copy, and the
  workflow search placeholder through app localization.
- Localized high-risk AI settings page strings, including permissions, usage,
  API key inputs, custom endpoint copy, active/input sections, fallback credit
  copy, AWS Bedrock copy, and related toggles.
- Added a high-risk AI settings wrapper static test for multi-line wrapper
  calls that the previous line-oriented scan could miss.
- Localized onboarding callout titles, body copy, buttons, checkbox labels, and
  prompt text through `OnboardingCopy` and catalog keys.
- Added targeted static coverage for onboarding callout field literals and
  `OnboardingQuery` prompt literals.
- Made missing onboarding copy keys fail explicitly instead of returning an
  empty string.
- Regenerated the zh-CN visual smoke artifact after the onboarding callout fix.

## Catalog State

- `en-US` keys: 5752
- `zh-CN` keys: 5752
- Missing in `zh-CN`: 0
- Extra in `zh-CN`: 0
- Placeholder mismatches: 0
- Empty values: only `auth.empty`

## Verification Matrix

The following commands were run locally on the current rebased worktree.

```bash
git fetch upstream
git rebase upstream/master
```

Result: pass. The branch was rebased onto
`a44b703060673b85d2641c051c53e4b6b1b00cc4`; the single
`teams_page.rs` conflict was resolved as described above.

```bash
cargo build -p integration --bin integration
```

Result: pass, finished in 50m 36s.

```bash
ARTIFACT_DIR="$PWD/target/zh-cn-visual-artifacts-20260601T013621Z"
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$ARTIFACT_DIR" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Result: pass.

Current zh-CN visual smoke evidence:

- `target/zh-cn-visual-artifacts-20260601T013621Z/test_zh_cn_localization_visual_smoke/2026-06-01T09-36-36/recording.log`
- 8 screenshots in the same directory:
  `agent-input-zh-cn.png`, `command-palette-zh-cn.png`,
  `command-search-zh-cn.png`, `context-chips-zh-cn.png`,
  `dialog-launch-config-zh-cn.png`, `settings-appearance-language-zh-cn.png`,
  `terminal-input-zh-cn.png`, and `toast-zh-cn.png`
- Every PNG was verified with `sips -g pixelWidth -g pixelHeight` as
  `2560x1600`

```bash
cargo fmt --all -- --check
```

Result: pass.

```bash
git diff --check
```

Result: pass.

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

Result: pass. Output summary: `enKeys` 5752, `zhKeys` 5752, `missing` 0,
`extra` 0, `placeholderMismatch` 0, `empty` only `auth.empty`.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization -- --nocapture
```

Result: pass. 25 tests passed, 0 failed, 0 ignored.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p onboarding --message-format=short
```

Result: pass, finished in 3m 08s.

One default-target run of `cargo test -p warp_localization -- --nocapture` was
stopped after approximately 12 minutes while still compiling in `target/debug`.
The counted result is the successful `CARGO_TARGET_DIR=target/localization-audit`
run above.

## Existing Video Evidence

Current pre-existing video recording evidence:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.log`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_bootstrap.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_commands.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.mp4`

The video recording smoke was not regenerated in this pass.

## Branch Notes

- No PR was created or updated.
- No branch was created.
- No remote refs were modified.
- Local `target/` artifacts are intentionally untracked build and visual
  evidence artifacts.

## Remaining Risk

- No external CI or external code review has been performed after this latest
  local validation.
- Visual coverage is sampled and must not be described as complete product-wide
  UI human review or exhaustive platform coverage.
- The video recording smoke artifact was not regenerated in this pass.
