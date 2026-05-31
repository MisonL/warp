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
  `debe6d8104aed38afd7db6dedba668f8f8246818`
- Upstream comparison after committing this evidence update:
  `git rev-list --left-right --count upstream/master...HEAD` returns `0 22`
- Remote branch comparison after committing this evidence update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returns `3 97`
- Rebase state after the rebase: no `.git/rebase-merge`,
  `.git/rebase-apply`, `MERGE_HEAD`, or `CHERRY_PICK_HEAD`

## Upstream Delta Classification

The branch was rebased from the previously recorded upstream
`5767910b5e41bda196baaea041862e9505e46e20` to
`debe6d8104aed38afd7db6dedba668f8f8246818`.

The new upstream tip includes remote skill location resolution work in agent
tool output paths and related tests. It did not modify bundled locale catalogs
or `crates/localization/src`. The only rebase conflict was in
`app/src/ai/blocklist/block/view_impl/output_tests.rs`; the resolution preserved
both upstream test additions and the local settings initialization used by the
localization branch.

After the successful rebase, later `git fetch upstream` retries failed with:

```text
fatal: unable to access 'https://github.com/warpdotdev/warp/': LibreSSL SSL_connect: SSL_ERROR_SYSCALL in connection to github.com:443
```

That is recorded as a network/TLS fetch failure. It does not change the fact
that the successful rebase in this pass was onto local `upstream/master` at
`debe6d8104aed38afd7db6dedba668f8f8246818`.

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

## Additional Fixes From This Audit

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
`debe6d8104aed38afd7db6dedba668f8f8246818`.

```bash
cargo fmt --all -- --check
```

Result: pass after running `cargo fmt --all`.

```bash
git diff --check
```

Result: pass.

```bash
node - <<'NODE'
const fs=require('fs');
const en=JSON.parse(fs.readFileSync('app/assets/bundled/locales/en-US.json','utf8'));
const zh=JSON.parse(fs.readFileSync('app/assets/bundled/locales/zh-CN.json','utf8'));
const ek=Object.keys(en).sort();
const zk=Object.keys(zh).sort();
const missing=ek.filter(k=>!(k in zh));
const extra=zk.filter(k=>!(k in en));
const re=/\{[A-Za-z_][A-Za-z0-9_]*\}/g;
const placeholders=o=>Object.fromEntries(Object.entries(o).map(([k,v])=>[k,[...String(v).matchAll(re)].map(m=>m[0]).sort().join(',')]));
const ep=placeholders(en), zp=placeholders(zh);
const mismatch=ek.filter(k=>k in zh && ep[k]!==zp[k]).map(k=>({key:k,en:ep[k],zh:zp[k]}));
const empty=ek.filter(k=>en[k]===''||zh[k]==='');
console.log(JSON.stringify({en_count:ek.length, zh_count:zk.length, missing:missing.length, extra:extra.length, placeholder_mismatch:mismatch.length, empty}, null, 2));
if (missing.length||extra.length||mismatch.length) process.exit(1);
NODE
```

Result: pass, with `en_count` 5752, `zh_count` 5752, `missing` 0, `extra` 0,
`placeholder_mismatch` 0, and `empty` equal to `["auth.empty"]`.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization --test localization_tests \
  onboarding_callout_direct_english_literals_are_localized -- --nocapture
```

Result: pass, 1 test passed.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization --test localization_tests direct_english -- --nocapture
```

Result: pass, 8 tests passed, 0 failed, 17 filtered out. This focused scan
covers app, onboarding, shared `ui_components`, context-chip tooltip, app-menu,
selected miscellaneous UI, AI settings high-risk wrappers, and the new
onboarding callout direct-English check.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization -- --nocapture
```

Result: pass, 25 tests passed, 0 failed, 0 ignored.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p onboarding --message-format=short
```

Result: pass, finished in 3m 20s.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p warp --lib --message-format=short
```

Result: pass, finished in 14m 39s. This covers the app lib compile path,
including `app/src/terminal/view.rs` and the updated onboarding callout API.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp --lib localization::tests -- --nocapture
```

Result: not counted as pass. The command compiled through the app test target
for more than 30 minutes and was manually terminated while rustc was still in
the `warp` test binary build stage. The replacement app-level compile evidence
is the successful `cargo check -p warp --lib --message-format=short` run above.

## Existing Visual Evidence

Pre-existing zh-CN visual smoke evidence remains available from the prior pass:

- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/recording.log`
- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/*.png`

Pre-existing video recording evidence remains available from the prior pass:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.log`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_bootstrap.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_commands.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.mp4`

Those artifacts were not regenerated after the onboarding callout fix in this
audit. The onboarding callout fix is covered by catalog parity, static
direct-English tests, `cargo check -p onboarding`, and
`cargo check -p warp --lib`.

## Completion Audit

- Rebase to latest locally fetched upstream: satisfied for
  `debe6d8104aed38afd7db6dedba668f8f8246818`.
- Follow-up fetch freshness check: blocked by GitHub TLS connection failure,
  recorded above.
- No push or PR update: satisfied.
- Catalog JSON parse and parity: satisfied, missing 0, extra 0, placeholder
  mismatches 0, and only `auth.empty` empty.
- Direct-English high-risk scans: satisfied by the focused and full
  `warp_localization` test runs.
- Onboarding callout omission: fixed and covered by targeted static test plus
  onboarding/app compile checks.
- App compile path: satisfied by `cargo check -p warp --lib`.

## Remaining Risk

- External CI and external code review have not been run in this pass.
- A later upstream fetch could not be completed because GitHub returned a TLS
  connection failure.
- The pre-existing visual smoke evidence was not regenerated after the
  onboarding callout fix.
- Visual coverage is sampled and does not prove exhaustive coverage of every UI
  surface, platform state, runtime configuration, or translated context.
- Local `target/` artifacts are build and review evidence, not tracked source
  files.
