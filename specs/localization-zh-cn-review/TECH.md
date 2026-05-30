# zh-CN Localization Review Evidence

## Scope

This record captures the local rebase, fix, and validation state for the Warp
zh-CN localization work on branch `feat/localization-settings-upstream-rebuild`.
It is intentionally local-only: no PR, push, remote ref update, or new branch was
created in this pass.

## Current Branch State

- Branch: `feat/localization-settings-upstream-rebuild`
- Verified upstream base: `upstream/master` at `ce73fe07`
- Local source validation and current-code real-display smoke were run after
  the rebase; this evidence is included in this local evidence commit.
- Upstream ancestry: `git merge-base --is-ancestor upstream/master HEAD`
  returned 0
- Upstream comparison after this evidence update:
  `git rev-list --left-right --count upstream/master...HEAD` returned `0 17`
- Remote branch comparison after this evidence update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returned `3 65`
- Rebase state: no `.git/rebase-merge`, `.git/rebase-apply`, `MERGE_HEAD`, or
  `CHERRY_PICK_HEAD`

## Current Fixes

This pass addressed gaps found while revalidating the branch after upstream
refreshes through `ce73fe07`:

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
  the branch rebased cleanly onto `upstream/master` at `ce73fe07` with no
  conflicts.
- macOS integration real-display intent is now propagated from
  `Builder::with_real_display()` into the app/window-manager startup path. This
  lets real-display integration tests create real-display windows and save PNG
  frame captures without requiring callers to set
  `WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS`.

## Catalog State

Current catalog stats:

- `en-US` keys: 5693
- `zh-CN` keys: 5693
- Missing in `zh-CN`: 0
- Extra in `zh-CN`: 0
- Placeholder mismatches: 0
- Identical values: 63
- Empty values: `auth.empty` in both catalogs
- ASCII-only zh-CN values: 69
- ASCII-with-CJK zh-CN values: 2272

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

Result: pass. `upstream/master` remained `ce73fe07`; rebase reported that the
branch was up to date.

```bash
python3 -m json.tool app/assets/bundled/locales/en-US.json
python3 -m json.tool app/assets/bundled/locales/zh-CN.json
```

Result: pass.

Catalog parity script result: `en-US` keys 5693, `zh-CN` keys 5693, missing 0,
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

Result: pass, finished in 1m 37s.

```bash
cargo test -p warp_localization -- --nocapture
```

Result: pass, 23 tests passed. The run compiled in 3m 53s and finished tests in
16.73s. Coverage includes bundled key/placeholder parity, app/onboarding/shared
ui-components direct-English UI constructor scans, and selected surface
regression checks.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: pass, 8 tests passed with 4715 filtered tests. The app test binary
compiled in 24m 47s and the selected tests finished in 1.17s.

```bash
cargo check -p warp --lib --message-format=short
```

Result: pass, 18m 41s.

```bash
cargo build -p integration --bin integration
```

Result: pass, 31m 01s after the real-display propagation fix.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts-current" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Earlier result before the real-display propagation fix: pass, exit code 0, but
only `recording.log` was generated.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts-fixed" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Current result after the fix: pass, exit code 0. Current artifact directory:
`target/zh-cn-visual-artifacts-fixed/test_zh_cn_localization_visual_smoke/2026-05-30T10-44-33`.

PNG artifact check: 8 screenshots were generated. `sips -g pixelWidth -g
pixelHeight` reported `1280x800` for all 8 PNG files.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/video-recording-smoke-fixed" \
  target/debug/integration test_video_recording
```

Result: pass, exit code 0. The run produced `after_bootstrap.png`,
`after_commands.png`, `recording.mp4`, and `recording.log` under
`target/video-recording-smoke-fixed/test_video_recording/2026-05-30T10-47-41`.
Both PNG files are `1280x800`; `recording.mp4` is 221 KB.

## Visual Evidence

Current real-display smoke evidence:

- `target/zh-cn-visual-artifacts-fixed/test_zh_cn_localization_visual_smoke/2026-05-30T10-44-33/recording.log`
- `target/zh-cn-visual-artifacts-fixed/test_zh_cn_localization_visual_smoke/2026-05-30T10-44-33/*.png`

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
  `ce73fe07`.
- No new branch: satisfied.
- No push or PR update: satisfied.
- Key and placeholder integrity: satisfied by catalog stats and
  `warp_localization` tests, including onboarding copy keys extracted from
  `crates/onboarding/src/copy.rs`.
- Hardcoded direct-English regression checks: satisfied for the current static
  audit scope. App, onboarding, and shared `ui_components/src` UI constructor
  scans passed. The supplemental high-risk scan only matched test assertions
  and example labels.
- App localization tests: satisfied by
  `cargo test -p warp --lib localization::tests -- --nocapture`, 8 passed with
  4715 filtered tests.
- App compile check: satisfied by `cargo check -p warp --lib`.
- Real-display zh-CN smoke flow: satisfied by the current run at
  `target/zh-cn-visual-artifacts-fixed/test_zh_cn_localization_visual_smoke/2026-05-30T10-44-33/recording.log`.
- Real-display PNG screenshot capture: satisfied by the current run at
  `target/zh-cn-visual-artifacts-fixed/test_zh_cn_localization_visual_smoke/2026-05-30T10-44-33`.

## Historical Notes

Earlier local validation points included upstream bases such as `37df9ef2`,
`af886f7c`, `a4d19abd`, and `df02914a`. Those are historical only. The current
evidence basis is the branch rebased on `ce73fe07` with the fixes and
validation recorded above.

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
- Current real-display smoke assertions passed locally after the latest rebase,
  but PNG screenshot capture did not produce current-code screenshots on this
  machine; this remains a local visual artifact gate before claiming
  screenshot-backed visual coverage.
- Local `target/` artifacts are build and review evidence, not tracked source
  files.
