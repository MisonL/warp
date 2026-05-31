# Local zh-CN Localization Review Handoff

## Scope

This is a local-only handoff for the Warp zh-CN localization candidate on
`feat/localization-settings-upstream-rebuild`. No push, PR update, remote ref
update, or new branch was created.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Rebased upstream base: `upstream/master` at
  `debe6d8104aed38afd7db6dedba668f8f8246818`
- Upstream comparison after committing this handoff update:
  `git rev-list --left-right --count upstream/master...HEAD` returns `0 22`
- Remote branch comparison after committing this handoff update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returns `3 97`
- Rebase state after the rebase: no `.git/rebase-merge`,
  `.git/rebase-apply`, `MERGE_HEAD`, or `CHERRY_PICK_HEAD`
- Remote writes in this pass: none
- PR work in this pass: none

## Latest Upstream Delta

The branch was rebased onto local `upstream/master` at
`debe6d8104aed38afd7db6dedba668f8f8246818`.

The new upstream delta after the previous handoff includes remote skill
location resolution in agent tool output paths and related tests. It did not
modify bundled locale catalogs or `crates/localization/src`. The only rebase
conflict was in
`app/src/ai/blocklist/block/view_impl/output_tests.rs`; the resolution kept
both upstream test additions and the local settings initialization needed by
this branch.

A later freshness retry of `git fetch upstream` failed with:

```text
fatal: unable to access 'https://github.com/warpdotdev/warp/': LibreSSL SSL_connect: SSL_ERROR_SYSCALL in connection to github.com:443
```

## Change Summary

- Rebased the branch onto `upstream/master` at `debe6d81`.
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

Result: pass. Output summary: `en_count` 5752, `zh_count` 5752, `missing` 0,
`extra` 0, `placeholder_mismatch` 0, `empty` only `auth.empty`.

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

Result: pass, 8 tests passed.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp_localization -- --nocapture
```

Result: pass, 25 tests passed.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p onboarding --message-format=short
```

Result: pass, finished in 3m 20s.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo check -p warp --lib --message-format=short
```

Result: pass, finished in 14m 39s.

```bash
CARGO_TARGET_DIR=target/localization-audit \
  cargo test -p warp --lib localization::tests -- --nocapture
```

Result: not counted as pass. The command compiled for more than 30 minutes and
was manually terminated while rustc was still building the `warp` test binary.
The app compile path is covered by the successful `cargo check -p warp --lib`
run.

## Existing Visual Evidence

Current pre-existing zh-CN visual smoke evidence:

- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/recording.log`
- `target/zh-cn-visual-artifacts-20260531T173639/test_zh_cn_localization_visual_smoke/2026-05-31T17-37-22/*.png`

Current pre-existing video recording evidence:

- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.log`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_bootstrap.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/after_commands.png`
- `target/video-recording-smoke-20260531T173639/test_video_recording/2026-05-31T17-37-56/recording.mp4`

These artifacts were not regenerated after the onboarding callout fix.

## Branch Notes

- No PR was created or updated.
- No branch was created.
- No remote refs were modified.
- Local `target/` artifacts are intentionally untracked build and visual
  evidence artifacts.

## Remaining Risk

- No external CI or external code review has been performed after this latest
  local validation.
- A later upstream fetch could not be completed because GitHub returned a TLS
  connection failure.
- Visual coverage is sampled and was not regenerated after the onboarding
  callout fix.
- This local evidence must not be described as full UI human review or complete
  product-wide exhaustive coverage.
