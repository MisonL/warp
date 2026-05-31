# zh-CN Localization Review Evidence

## Scope

This record captures the local rebase, fix, and validation state for the Warp
zh-CN localization work on branch `feat/localization-settings-upstream-rebuild`.
It is intentionally local-only: no PR, push, remote ref update, or new branch was
created in this pass.

## Current Branch State

- Branch: `feat/localization-settings-upstream-rebuild`
- Verified upstream base: `upstream/master` at `21334d42`
- Local source validation and current-code real-display smoke were run after the
  latest source-affecting rebase. This pass also rebased onto `21334d42`; the
  only new upstream delta from `74d25664` to `21334d42` is
  `docker/agent-dev/Dockerfile`, with no app UI or localization source changes.
- Upstream ancestry: `git merge-base --is-ancestor upstream/master HEAD`
  returned 0
- Upstream comparison after this evidence update:
  `git rev-list --left-right --count upstream/master...HEAD` returned `0 19`
- Remote branch comparison after this evidence update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returned `3 92`
- Rebase state: no `.git/rebase-merge`, `.git/rebase-apply`, `MERGE_HEAD`, or
  `CHERRY_PICK_HEAD`

## Current Fixes

This pass addressed gaps found while revalidating the branch after upstream
refreshes through `74d25664`, then rebased the result onto `21334d42`:

- AI settings hardcoded UI copy is now catalog-backed, including execution
  profile actions, toolbar layout copy, remote-session policy text, sign-up and
  billing CTAs, custom inference controls, voice input copy, CLI agent toolbar
  copy, API key actions, and related setting labels.
- Agent management's branch-copy toast now uses
  `agent_management.toast.copied_branch_name`.
- Search filter placeholder and chip labels now use app-side localization while
  `warp_search_core` exposes stable catalog key methods.
- Notebook find-bar regex and case-sensitive toggle tooltips now resolve through
  catalog-backed strings.
- `FilterableDropdown` now accepts owned menu header text so localized dynamic
  headers can be used where the old API only accepted `&'static str`.
- MCP server picker headers and the directory color picker header now use
  localized catalog values instead of direct English fallback text.
- Onboarding now uses an `OnboardingCopy` injection path. The app runtime passes
  catalog-backed copy into onboarding, while the standalone onboarding binary
  keeps English defaults. Intro, intention, customize, Agent, third-party,
  theme, project, free-user-no-AI, and shared toggle-card UI copy no longer
  rely on direct user-visible English literals in the current static audit.
- The auth login disable-confirm feature lists now share onboarding feature key
  arrays so AI and Warp Drive feature bullets are localized through the same
  catalog keys.
- Shared lightbox empty and loading labels are now injected by the app runtime
  with catalog-backed strings; `ui_components` examples keep explicit English
  example labels.
- Editor image-context button tooltips now resolve through catalog keys,
  including query and conversation count placeholders.
- Auth web handoff loading/error copy, inline terminal menus, cloud-mode slash
  command empty states, context-chip display menu empty states, and Agent
  Management no-results copy now use catalog-backed strings.
- Command Search Warp AI data-source errors now expose optional catalog keys
  through `DataSourceRunError::user_facing_error_text_key`, while retaining the
  existing English fallback via the `en-US` catalog. Search-bar a11y loading,
  error, and selected-item announcements also use catalog keys.
- The localization regression tests now include a shared `ui_components/src`
  direct-English UI constructor scan and skip test fixture files.
- Compile drift from the latest upstream rebase was fixed in the app
  localization path without changing non-localization business semantics, and
  the branch rebased onto `upstream/master` at `74d25664` after resolving
  upstream refresh conflicts in the localization commit set, onboarding
  customize slide copy, and shared lightbox loading UI. The later rebase onto
  `21334d42` had no conflicts and introduced only a Dockerfile change from
  upstream.
- macOS integration real-display intent is now propagated from
  `Builder::with_real_display()` into the app/window-manager startup path. This
  lets real-display integration tests create real-display windows and save PNG
  frame captures without requiring callers to set
  `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS`.

## Catalog State

Current catalog stats:

- `en-US` keys: 5695
- `zh-CN` keys: 5695
- Missing in `zh-CN`: 0
- Extra in `zh-CN`: 0
- Placeholder mismatches: 0
- Identical values: 63
- Empty values: `auth.empty` in both catalogs
- ASCII-only zh-CN values: 70
- ASCII-with-CJK zh-CN values: 2260

Term scan counts:

- `prompt`: 6
- `pane`: 0
- `handoff`: 0
- `snapshot`: 0
- `payload`: 0
- `pull request`: 0
- `workflow`: 0
- `workspace`: 1
- `Workflows`: 0
- `Notebooks`: 0
- `Use Agent`: 0
- `Copyright`: 0
- `智能体`: 0

The remaining ASCII-only zh-CN values were reviewed as intentional
brand/product names, commands, table fields, placeholders, IDs, provider names,
or formatting fragments such as `Agent`, `Notebook`, `Warp Drive`, `Cloud Oz`,
`GitHub Action`, `nvm install node`, `Slug`, `ID`, `UUID`, `JSON`, `Shell`,
`\n`, and `{credits} / {price}`.

## Verification

Commands run locally on the current worktree during this evidence update:

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

High-risk app/ui-components literal scan result: only test assertions and
`crates/ui_components/examples/library.rs` example labels still matched the
sample English strings (`Loading...`, `No images`). No app runtime path matched
the newly fixed strings.

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
`target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02`.

PNG artifact check: 8 screenshots were generated. `sips -g pixelWidth -g
pixelHeight` reported `1280x800` for all 8 PNG files.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/video-recording-smoke-rebased-20260530" \
  target/debug/integration test_video_recording
```

Result: pass, exit code 0. The run produced `after_bootstrap.png`,
`after_commands.png`, `recording.mp4`, and `recording.log` under
`target/video-recording-smoke-rebased-20260530/test_video_recording/2026-05-30T12-55-41`.
Both PNG files are `1280x800`; `recording.mp4` is 218 KB.

## Visual Evidence

Current real-display smoke evidence:

- `target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02/recording.log`
- `target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02/*.png`

The current run returned exit code 0 and the recording log shows every smoke
step and assertion succeeded, including app menu/Dock menu localization,
Settings and Appearance language UI, Terminal input focus, context chip
presence, command search, Agent input mode, command palette, workspace toast,
and launch-config dialog focus.

The current run also saved 8 PNG screenshots: settings appearance language,
terminal input, context chips, command search, Agent input, command palette,
workspace toast, and launch-config dialog. The previous missing-PNG root cause
was that `Builder::with_real_display()` was not propagated to macOS app/window
startup, leaving integration windows in test mode without a real Metal device
unless `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS` was set externally.

## Completion Audit

Goal item status against the current local branch:

- Rebase to latest upstream: satisfied locally for `upstream/master` at
  `21334d42`.
- No new branch: satisfied.
- No push or PR update: satisfied.
- Key and placeholder integrity: satisfied by catalog stats and
  `warp_localization` tests, including onboarding copy keys extracted from
  `crates/onboarding/src/copy.rs`.
- Hardcoded direct-English regression checks: satisfied for the current static
  audit scope. App, onboarding, and shared `ui_components/src` UI constructor
  scans passed. The supplemental high-risk scan only matched test assertions
  and example labels.
- App localization tests: previously satisfied by
  `cargo test -p warp --lib localization::tests -- --nocapture`, 8 passed with
  4830 filtered tests on the latest source-affecting rebase. The same command
  was attempted after rebasing to `21334d42`, but did not produce an exit code
  because the app test binary compile remained in local `rustc` for about 55
  minutes and was stopped.
- Real-display zh-CN smoke flow: satisfied by the current run at
  `target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02/recording.log`.
- Real-display PNG screenshot capture: satisfied by the current run at
  `target/zh-cn-visual-artifacts-rebased-20260530/test_zh_cn_localization_visual_smoke/2026-05-30T12-53-02`.

## Historical Notes

Earlier local validation points included upstream bases such as `37df9ef2`,
`af886f7c`, `a4d19abd`, `df02914a`, `ce73fe07`, and `74d25664`. Those are
historical only. The current branch is rebased on `21334d42`; the current
source-validation basis remains the `74d25664` source-affecting rebase plus the
verified fact that the only later upstream delta is `docker/agent-dev/Dockerfile`.

Previously fixed upstream-refresh gaps include:

- async find setting catalog keys
- queued prompt edit/delete tooltips
- cloud-agent fast-forward locked tooltip
- auth secret delete confirmation dialog
- Agent conversation list section headers and relative timestamps
- app menu and Dock menu runtime visual assertions
- upstream onboarding flow app-runtime localization through `OnboardingCopy`

## Remaining Risk

- External CI and external code review have not been run in this pass.
- No PR evidence exists because the user explicitly instructed not to submit or
  update a PR.
- Visual coverage is sampled and does not prove exhaustive coverage of every UI
  surface, platform state, runtime configuration, or translated context.
- The app-side `cargo test -p warp --lib localization::tests -- --nocapture`
  rerun after `21334d42` did not complete locally because the app test binary
  compile stayed in `rustc` for about 55 minutes without an exit code. The
  narrower `warp_localization` and onboarding checks did complete and pass on
  `21334d42`.
- Local `target/` artifacts are build and review evidence, not tracked source
  files.
