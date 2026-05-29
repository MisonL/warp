# Local zh-CN Localization Review Handoff

## Scope

This is a local-only handoff for the Warp zh-CN localization candidate on
`feat/localization-settings-upstream-rebuild`. The user explicitly instructed
not to create or update a PR, not to push, and not to create a new branch.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Upstream base verified locally: `upstream/master` at `ce73fe07`
- Local source validation and current-code real-display smoke were run after
  the rebase; this evidence is included in this local evidence commit.
- Upstream comparison after this handoff update:
  `git rev-list --left-right --count upstream/master...HEAD` returned `0 17`
- Remote branch comparison after this handoff update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returned `3 65`
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
- Updates this handoff and `TECH.md` with current local validation evidence and
  current real-display smoke evidence plus the remaining PNG screenshot gate.

## Verification Matrix

The following commands were run locally on the current worktree during this
handoff update.

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

Result: pass, 21m 38s.

```bash
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts-current" \
  target/debug/integration test_zh_cn_localization_visual_smoke
```

Result: pass, exit code 0. Current artifact log:

- `target/zh-cn-visual-artifacts-current/test_zh_cn_localization_visual_smoke/2026-05-30T01-30-03/recording.log`

Historical artifact logs:

- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T21-56-59/recording.log`

The historical run did not save PNG screenshots.

## Visual Evidence

Current visual smoke status:

- Real-display route and assertion smoke: passed after the latest rebase to
  `ce73fe07`; see
  `target/zh-cn-visual-artifacts-current/test_zh_cn_localization_visual_smoke/2026-05-30T01-30-03/recording.log`.
- Current-code PNG screenshot capture: not satisfied. The current artifact
  directory contains `recording.log` only.

Historical local screenshots still exist under
`target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T16-14-58`,
but that complete PNG set predates the latest rebase to `ce73fe07` and should not
be cited as current-code screenshot proof.

## Catalog And Terminology Checks

- `en-US` keys: 5693
- `zh-CN` keys: 5693
- Missing keys: 0
- Extra keys: 0
- Placeholder mismatches: 0
- Identical values: 63
- Empty values: `auth.empty` in both catalogs
- ASCII-only zh-CN values: 69
- ASCII-with-CJK zh-CN values: 2272

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
- Current real-display smoke assertions passed, but screenshot capture did not
  produce current-code PNGs on this machine.
- This local evidence must not be described as full UI human review, complete
  product-wide exhaustive coverage, or screenshot-backed current-code visual
  proof.
