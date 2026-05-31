# Local zh-CN Localization Review Handoff

## Scope

This is a local-only handoff for the Warp zh-CN localization candidate on
`feat/localization-settings-upstream-rebuild`. The user explicitly instructed
not to create or update a PR, not to push, and not to create a new branch.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Upstream base verified locally: `upstream/master` at `21334d42`
- Local source validation and current-code real-display smoke were run after the
  latest source-affecting rebase. This pass also rebased onto `21334d42`; the
  only new upstream delta from `74d25664` to `21334d42` is
  `docker/agent-dev/Dockerfile`, with no app UI or localization source changes.
- Upstream comparison after this handoff update:
  `git rev-list --left-right --count upstream/master...HEAD` returned `0 19`
- Remote branch comparison after this handoff update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returned `3 92`
- Rebase state: no `.git/rebase-merge`, `.git/rebase-apply`, `MERGE_HEAD`, or
  `CHERRY_PICK_HEAD`
- Remote writes in this pass: none
- PR work in this pass: none

## Change Summary

- Keeps the existing `warp_localization` crate and bundled `en-US` / `zh-CN`
  catalogs as the localization surface for app UI copy.
- Wires the latest AI settings and Agent management UI literals through
  catalog-backed strings.
- Restores localized search filter placeholders and chip labels without letting
  `warp_search_core` depend on app-localization state.
- Restores localized notebook find-bar toggle tooltips.
- Extends `FilterableDropdown` so owned localized menu headers can be used by
  MCP server pickers and the directory color picker.
- Wires onboarding through `OnboardingCopy` so the app runtime can pass
  catalog-backed onboarding text across intro, intention, customize, Agent,
  third-party, theme, project, free-user-no-AI, and shared toggle-card UI. The
  standalone onboarding binary keeps English defaults.
- Shares onboarding feature key arrays with the auth disable-confirm modal so
  AI and Warp Drive feature bullets are localized through the same catalog keys.
- Localizes shared lightbox empty/loading labels via app-injected catalog-backed
  copy while preserving explicit English labels in standalone examples.
- Localizes editor image-context tooltips, auth web handoff copy, inline menu
  and cloud slash-command empty states, context-chip display menus, Agent
  Management no-results copy, Command Search Warp AI data-source errors, and
  search-bar a11y announcements.
- Extends localization regression tests to scan shared `ui_components/src`
  direct-English UI constructor calls.
- Fixes local compile drift found after upstream rebase.
- Fixes macOS integration real-display propagation so tests that call
  `Builder::with_real_display()` now create real-display windows and can save
  PNG frame captures without relying on
  `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS`.
- Updates this handoff and `TECH.md` with current local validation evidence and
  current real-display smoke evidence, including current-code PNG screenshots.

## Verification Matrix

The following commands were run locally on the current worktree during this
handoff update.

```bash
git fetch upstream master
git rebase upstream/master
```

Result: pass. `upstream/master` advanced to `21334d42`; this latest rebase had
no conflicts. The new upstream commit only changes `docker/agent-dev/Dockerfile`.
The previous source-affecting rebase through `74d25664` completed after
resolving conflicts in the localization commit set, onboarding customize slide
copy, and shared lightbox loading UI.

```bash
python3 -m json.tool app/assets/bundled/locales/en-US.json
python3 -m json.tool app/assets/bundled/locales/zh-CN.json
```

Result: pass.

Catalog parity script result: `en-US` keys 5695, `zh-CN` keys 5695, missing 0,
extra 0, placeholder mismatches 0, empty zh-CN values only `auth.empty`.

```bash
rg -n 'paragraph\("[^"]*[A-Za-z][^"]*"|span\("[^"]*[A-Za-z][^"]*"|link\("[^"]*[A-Za-z][^"]*"|Text::new\("[^"]*[A-Za-z][^"]*"|Text::new_inline\("[^"]*[A-Za-z][^"]*"|FormattedTextElement::from_str\("[^"]*[A-Za-z][^"]*"|button::Content::Label\("[^"]*[A-Za-z][^"]*"|wrappable_text\("[^"]*[A-Za-z][^"]*"' crates/onboarding/src -g '*.rs'
```

Result: pass. The scan produced no direct user-visible English literals in
onboarding UI constructor calls.

Supplemental high-risk literal scan result: only test assertions and
`crates/ui_components/examples/library.rs` example labels matched the sample
English strings; no app runtime path matched the newly fixed strings.

```bash
cargo fmt --all -- --check
```

Result: pass.

```bash
git diff --check
```

Result: pass.

```bash
cargo check -p onboarding --message-format=short
```

Result: pass, finished in 5m 03s on the `21334d42` rebase.

```bash
cargo test -p warp_localization -- --nocapture
```

Result: pass, 23 tests passed. The run compiled in 5m 59s and finished tests in
58.66s on the `21334d42` rebase. Coverage includes bundled key/placeholder parity, app/onboarding/shared
ui-components direct-English UI constructor scans, and selected surface
regression checks.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Previous complete source-equivalent result: pass, 8 tests passed with 4830
filtered tests on the `74d25664` source-affecting rebase. This command was
attempted again after rebasing to `21334d42`, but the local app test binary
compile remained in `rustc` for about 55 minutes without an exit code and was
stopped. Because `21334d42` only changes `docker/agent-dev/Dockerfile`, this is
recorded as a local toolchain/run-time limitation rather than a new
localization failure.

```bash
cargo build -p integration --bin integration
```

Result: pass, 26m 57s after the real-display propagation fix.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts-current" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Earlier result before the real-display propagation fix: pass, exit code 0, but
only `recording.log` was generated.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts-rebased-20260530" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Current result after the fix: pass, exit code 0. Current artifact directory:

- `target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02`

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/video-recording-smoke-rebased-20260530" \
  target/debug/integration test_video_recording
```

Result: pass, exit code 0. Regression artifact directory:

- `target/video-recording-smoke-rebased-20260530/test_video_recording/2026-05-30T12-55-41`

The latest video regression run produced `after_bootstrap.png`,
`after_commands.png`, `recording.mp4`, and `recording.log`. Both PNG files are
`1280x800`; `recording.mp4` is 218 KB.

Historical artifact logs:

- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T21-56-59/recording.log`

The historical run did not save PNG screenshots.

## Visual Evidence

Current visual smoke status:

- Real-display route and assertion smoke: passed after the latest
  source-affecting rebase to `74d25664`; see
  `target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02/recording.log`.
- The subsequent rebase to `21334d42` only changed
  `docker/agent-dev/Dockerfile`, so no app UI source changed after this visual
  evidence was captured.
- Current-code PNG screenshot capture: satisfied. The current artifact
  directory contains 8 PNG screenshots plus `recording.log`.
- PNG dimension check: all 8 screenshots are `1280x800` per `sips -g
  pixelWidth -g pixelHeight`.

Historical local screenshots still exist under
`target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T16-14-58`,
but the current proof is the rebased run under
`target/zh-cn-visual-artifacts-rebased-20260530`.

Root cause of the previous missing PNGs: `Builder::with_real_display()` was
recorded on the lower-level integration builder but was not propagated to the
macOS app/window-manager startup path. Without the external
`WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS` environment variable, the macOS
integration window stayed in test mode with no real Metal device, so
`request_frame_capture` timed out after screenshot steps.

## Catalog And Terminology Checks

- `en-US` keys: 5695
- `zh-CN` keys: 5695
- Missing keys: 0
- Extra keys: 0
- Placeholder mismatches: 0
- Identical values: 63
- Empty values: `auth.empty` in both catalogs
- ASCII-only zh-CN values: 70
- ASCII-with-CJK zh-CN values: 2260

Reviewed terminology keeps product and protocol terms such as `Agent`,
`Warp Drive`, `Notebook`, `Cloud Oz`, `MCP`, `API`, `CLI`, `ID`, `JSON`,
`Shell`, `Slug`, and `Drive` in English intentionally. Catalog term scans
reported zero remaining zh-CN value matches for `智能体`, `Workflows`,
`Notebooks`, `Use Agent`, `Copyright`, `pane`, `handoff`, `snapshot`,
`payload`, and `pull request`.

## Branch Notes

- No PR was created or updated in this pass.
- No branch was created in this pass.
- No remote refs were modified in this pass.
- Local `target/` artifacts are intentionally untracked build and visual
  evidence artifacts.

## Remaining Risk

- No external CI or external review has been performed after the latest local
  validation.
- Visual coverage is sampled and not exhaustive across every platform surface,
  runtime configuration, and OS environment.
- This local evidence must not be described as full UI human review or complete
  product-wide exhaustive coverage.
