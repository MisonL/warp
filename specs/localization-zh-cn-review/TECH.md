# zh-CN Localization Review Evidence

## Scope

This record captures the local rebase, audit, fixes, and validation state for
the Warp zh-CN localization work on branch
`feat/localization-settings-upstream-rebuild`.

This is local-only evidence. No push, PR update, remote ref update, or new
branch was created.

## Current Branch State

- Branch: `feat/localization-settings-upstream-rebuild`
- Rebased upstream base: `upstream/master` at
  `a44b703060673b85d2641c051c53e4b6b1b00cc4`
- Source HEAD before this documentation-only evidence update:
  `f3de40efd992a51102790e94f522a299485b5e1e`
  (`Refresh zh-CN evidence after latest upstream`)
- Expected upstream comparison after committing this evidence update:
  `git rev-list --left-right --count upstream/master...HEAD` returns `0 24`
- Expected remote branch comparison after committing this evidence update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returns `3 100`
- Rebase state after the rebase: no `.git/rebase-merge`,
  `.git/rebase-apply`, `MERGE_HEAD`, or `CHERRY_PICK_HEAD`

## Upstream Delta Classification

`git fetch upstream` succeeded in this pass and advanced `upstream/master` from
`debe6d8104aed38afd7db6dedba668f8f8246818` to
`a44b703060673b85d2641c051c53e4b6b1b00cc4`.

The new upstream tip is `a44b7030 Fix team owner pill contrast (#11689)`. It
modifies `app/src/settings_view/teams_page.rs` to improve the owner status chip
contrast. It does not modify bundled locale catalogs or
`crates/localization/src`.

The only rebase conflict in this pass was in
`app/src/settings_view/teams_page.rs`. The resolution preserved the upstream
owner-chip contrast helper and opacity constant while keeping the localized
owner label:

```rust
teams_text(app, "settings.teams.status.owner")
```

A later freshness retry in this pass could not reach GitHub:

```text
git fetch upstream
fatal: unable to access 'https://github.com/warpdotdev/warp/': The requested URL returned error: 403

git ls-remote upstream refs/heads/master
fatal: unable to access 'https://github.com/warpdotdev/warp/': The requested URL returned error: 403

git ls-remote origin refs/heads/feat/localization-settings-upstream-rebuild
fatal: unable to access 'https://github.com/MisonL/warp/': The requested URL returned error: 403
```

The latest locally available upstream remains
`a44b703060673b85d2641c051c53e4b6b1b00cc4`.

## Current Fixes

- Localized AI context menu category labels through catalog keys.
- Localized Workflows category labels, empty state, accessibility copy, and the
  Voltron workflow search placeholder through app localization.
- Localized high-risk AI settings page labels, descriptions, section headers,
  API key inputs, custom endpoint copy, AWS Bedrock copy, permissions copy,
  usage copy, and related toggles through existing catalog-backed helpers.
- Added matching `en-US` and `zh-CN` catalog entries for AI context menu,
  Workflows, and AI settings strings.
- Added a regression scan for high-risk AI settings wrappers that the previous
  single-line direct-literal scan missed, including `build_sub_header`,
  `render_ai_setting_toggle`, `DropdownItem::new`, modal title paths, and
  related wrapper calls.
- Removed an unused `std::sync::LazyLock` import from `ai_page.rs` after the
  `warp` lib localization test target exposed it as a warning.
- Localized onboarding callout titles, body copy, buttons, checkbox labels, and
  terminal/agent prompt placeholders.
- Passed `OnboardingCopy` from `TerminalView` into `OnboardingCalloutView`, so
  callout copy resolves through `terminal_text(ctx, key)` instead of hardcoded
  English strings.
- Added catalog-backed onboarding prompt keys for:
  `onboarding.callout.talk_to_agent.prompt`,
  `onboarding.callout.agent_prompt.placeholder`, and
  `onboarding.callout.terminal_command.placeholder`.
- Replaced model-level placeholder variants with catalog-key variants, keeping
  command literals such as `git status` and `/init` as command values rather
  than UI copy.
- Made missing `OnboardingCopy` keys fail explicitly through `default_text`
  instead of silently returning an empty string.
- Added a targeted static test for onboarding callout field literals and
  `OnboardingQuery` prompt literals, covering the omission class that the
  previous wrapper scans did not catch.

## Catalog State

Current catalog parity command result:

- `en-US` keys: 5752
- `zh-CN` keys: 5752
- Missing in `zh-CN`: 0
- Extra in `zh-CN`: 0
- Placeholder mismatches: 0
- Empty values: only `auth.empty`

## Verification

Commands run locally on the current rebased worktree:

```bash
git fetch upstream
git rebase upstream/master
```

Result: pass. The branch was rebased onto
`a44b703060673b85d2641c051c53e4b6b1b00cc4`. The conflict in
`app/src/settings_view/teams_page.rs` was resolved as described above.

```bash
git fetch upstream
git ls-remote upstream refs/heads/master
git ls-remote origin refs/heads/feat/localization-settings-upstream-rebuild
```

Result: not counted as a freshness pass. All three commands returned GitHub
HTTP 403 errors as recorded above.

```bash
cargo build -p integration --bin integration
```

Result: pass, finished in 50m 36s.

```bash
ARTIFACT_DIR="$PWD/target/zh-cn-visual-artifacts-20260601T013621Z"
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$ARTIFACT_DIR" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Result: pass. The run produced:

- `target/zh-cn-visual-artifacts-20260601T013621Z/test_zh_cn_localization_visual_smoke/2026-06-01T09-36-36/recording.log`
- 8 PNG screenshots under the same directory:
  `agent-input-zh-cn.png`, `command-palette-zh-cn.png`,
  `command-search-zh-cn.png`, `context-chips-zh-cn.png`,
  `dialog-launch-config-zh-cn.png`, `settings-appearance-language-zh-cn.png`,
  `terminal-input-zh-cn.png`, and `toast-zh-cn.png`
- `sips -g pixelWidth -g pixelHeight` confirmed every PNG is `2560x1600`

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

Result: pass, with `enKeys` 5752, `zhKeys` 5752, `missing` 0, `extra` 0,
`placeholderMismatch` 0, and `empty` equal to `["auth.empty"]`.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization -- --nocapture
```

Result: pass in this pass as well. The suite reused the isolated target, then
ran 25 tests in 36.54s; 25 passed, 0 failed, 0 ignored. The heaviest static
scan, `app_ui_calls_do_not_use_direct_english_literals`, finished
successfully.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p onboarding --message-format=short
```

Result: pass, finished in 3m 08s.

An earlier default-target attempt of:

```bash
cargo test -p warp_localization -- --nocapture
```

was manually terminated after approximately 12 minutes while still compiling in
`target/debug`. The counted result is the successful isolated-target run above.

```bash
ARTIFACT_DIR="$PWD/target/video-recording-smoke-20260601T020835Z"
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$ARTIFACT_DIR" \
  target/debug/integration test_video_recording
```

Result: pass.

## Current Video Evidence

Current video recording evidence from this pass:

- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/recording.log`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/after_bootstrap.png`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/after_commands.png`
- `target/video-recording-smoke-20260601T020835Z/test_video_recording/2026-06-01T10-09-00/recording.mp4`

Artifact checks:

- `after_bootstrap.png`: `2560x1600`
- `after_commands.png`: `2560x1600`
- `recording.mp4`: 425276 bytes
- MP4 track parsed from the `tkhd` atom: `2560x1600`

Prior video evidence from `target/video-recording-smoke-20260531T173639/`
remains available but is superseded by the artifact above.

## Completion Audit

- Rebase to latest fetched upstream: satisfied for
  `a44b703060673b85d2641c051c53e4b6b1b00cc4`.
- Freshness check beyond local `a44b7030`: not satisfied because GitHub
  returned HTTP 403 for both fetch and ls-remote.
- No push or PR update: satisfied.
- Catalog JSON parse and parity: satisfied, missing 0, extra 0, placeholder
  mismatches 0, and only `auth.empty` empty.
- Direct-English high-risk scans: satisfied by the full `warp_localization`
  test run.
- Onboarding callout omission: fixed and covered by targeted static test,
  catalog parity, onboarding compile check, and refreshed zh-CN visual smoke.
- App compile path: satisfied by `cargo build -p integration --bin integration`
  and the successful visual smoke runner.
- Video recording smoke: satisfied by
  `target/video-recording-smoke-20260601T020835Z`.

## Remaining Risk

- External CI and external code review have not been run in this pass.
- A newer upstream commit may exist after local `a44b7030`, but GitHub returned
  HTTP 403 for the attempted fetch and ls-remote freshness checks.
- Visual coverage is sampled and does not prove exhaustive coverage of every UI
  surface, platform state, runtime configuration, or translated context.
- Local `target/` artifacts are build and review evidence, not tracked source
  files.
