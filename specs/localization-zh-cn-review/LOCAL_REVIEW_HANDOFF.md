# Local zh-CN Localization Review Handoff

## Scope

This is a local-only review handoff for the Warp zh-CN localization candidate.
It exists because the candidate is user-visible and must carry explicit
validation and visual evidence, while the user has explicitly instructed not to
submit or update a PR.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Validated candidate code head after rebase: `1fef94b0`. The post-rebase
  real-display visual smoke was run at local evidence head `1fef94b0`; commits
  after `1fef94b0` update local evidence documents only.
- App code changes since the latest code-validation point: none in the checked
  app and crate paths. `git diff --quiet 1fef94b0 HEAD -- app crates Cargo.toml
  Cargo.lock` returned 0 before this evidence update.
- Upstream base: `upstream/master` at `98af7b65`
- Upstream comparison at validation point `1fef94b0`: `0 12` from
  `git rev-list --left-right --count upstream/master...HEAD`
- Remote branch comparison at validation point `1fef94b0`: `3 16` from
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
- Working tree before this handoff: clean
- Open PRs for `feat/localization-settings-upstream-rebuild`: none
- Mistakenly opened PR: `#11739`, state `CLOSED`
- Remote writes in this pass: none

## Change Summary

- Adds `warp_localization` and bundled `en-US` / `zh-CN` locale catalogs.
- Adds app language settings for System, English, and Simplified Chinese.
- Wires `appearance.interface.language` through settings and app localization.
- Migrates user-visible UI copy across Agent, Settings, Terminal, Search,
  Workspace, menus, dialogs, toasts, and shared UI components to catalog-backed
  strings.
- Wires the upstream queued prompt edit/delete tooltips to
  `terminal.queued_prompts.tooltip.*` catalog keys after the `98af7b65` rebase.
- Adds catalog integrity tests, direct-English regression checks, app-level
  localization tests, and a real-display integration visual smoke test.

## Verification Matrix

The following commands were run locally against `1fef94b0` unless otherwise
noted.

```bash
git fetch upstream master
```

Result: pass. `upstream/master` remained `98af7b65`.

```bash
git diff --check upstream/master...HEAD && git diff --check && git diff --cached --check
```

Result: pass.

```bash
jq empty app/assets/bundled/locales/en-US.json app/assets/bundled/locales/zh-CN.json
```

Result: pass.

```bash
cargo fmt --all -- --check
```

Result: pass.

```bash
cargo test -p warp_localization -- --nocapture
```

Result: pass, 20 tests passed. Latest run compiled in 3.56s and the test
binary finished in 6.86s.

```bash
cargo test -p warp --lib localization_tests -- --nocapture
```

Result: pass as a compile/filter check, but it runs 0 tests because the current
app test module is registered as `localization::tests`. Latest run was on
pre-rebase `c2d9ddf8`, compiled in 1m 19s, and reported 4664 filtered tests.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: pass, 8 tests passed with 4683 filtered tests. Latest run compiled in
13m 16s and the filtered test run finished in 1.30s.

```bash
cargo check -p warp --lib --message-format=short
```

Result: pass, 5m 43s.

```bash
cargo build -p integration --bin integration
```

Result: pass on local evidence head `1fef94b0`, 14m 55s.

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts" \
WARP_INTEGRATION=1 \
target/debug/integration test_zh_cn_localization_visual_smoke
```

Result: pass on local evidence head `1fef94b0`, exit code 0. The run used a
real display, executed 15 steps, and asserted localized app menu and Dock menu
titles.

## Visual Evidence

Latest local screenshot artifacts:

- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/settings-appearance-language-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/terminal-input-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/context-chips-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/command-search-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/agent-input-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/command-palette-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/toast-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T00-20-47/dialog-launch-config-zh-cn.png`

Each artifact is a `1280 x 800` PNG. These files are local artifacts under
`target/`; they were not pushed or attached to any PR.

## Catalog And Terminology Checks

- `en-US` keys: 5578
- `zh-CN` keys: 5578
- Missing keys: 0
- Extra keys: 0
- Placeholder mismatches: 0
- Identical values: 61
- ASCII-only zh-CN values: 67
- ASCII-with-CJK zh-CN values: 2461

Reviewed terminology keeps product and protocol terms such as `Agent`,
`Warp Drive`, `Notebook`, `Cloud Oz`, `MCP`, `API`, `CLI`, `ID`, `JSON`,
`Shell`, `Slug`, and `Drive` in English intentionally. Catalog term scans
reported zero remaining zh-CN value matches for `智能体`, `Workflows`,
`Notebooks`, `Use Agent`, `Copyright`, `pane`, `handoff`, `snapshot`,
`payload`, and `pull request`.

## Branch Notes

- Local cleanup removed the obsolete local `feat/localization-settings` and
  `feat/localization-settings-upstream-validated` branches after pruning stale
  `/private/tmp/warp-i18n-port-test*` worktree records.
- Remote refs were not modified. `origin/feat/localization-settings`,
  `origin/feat/localization-settings-reviewed`, and
  `origin/feat/localization-settings-upstream-validated` remain available as
  remote-only historical references.

Recommendation: use `feat/localization-settings-upstream-rebuild` as the local
review candidate. Treat `feat/localization-settings-reviewed` as historical
only. Do not merge, delete, force-update, push, or create PRs for these branches
without explicit user instruction.

## Remaining Risk

- No external review has been performed after the latest local validation.
- No PR is open and no PR evidence has been attached, by explicit user
  instruction.
- Visual coverage is broad but not exhaustive across every platform surface.
- The May 27 rebase autostash remains in the stash list as a conservative
  backup.
