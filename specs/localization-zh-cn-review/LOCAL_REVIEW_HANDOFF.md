# Local zh-CN Localization Review Handoff

## Scope

This is a local-only review handoff for the Warp zh-CN localization candidate.
It exists because the candidate is user-visible and must carry explicit
validation and visual evidence, while the user has explicitly instructed not to
submit or update a PR.

## Candidate State

- Active local branch: `feat/localization-settings-upstream-rebuild`
- Validated candidate code head after rebase: `d4195647`. The post-rebase
  real-display visual smoke was run at local evidence head `36b9717d`; commits
  after `d4195647` update local evidence documents only.
- App code changes since the latest code-validation point: none in the checked
  app and crate paths. `git diff --quiet d4195647 HEAD -- app crates Cargo.toml
  Cargo.lock` returned 0 before this evidence update.
- Upstream base: `upstream/master` at `c37c1cd6`
- Upstream comparison after the latest evidence update: `0 10` from
  `git rev-list --left-right --count upstream/master...HEAD`
- Remote branch comparison after the latest evidence update: `3 11` from
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
- Adds catalog integrity tests, direct-English regression checks, app-level
  localization tests, and a real-display integration visual smoke test.

## Verification Matrix

The following commands were run locally against `d4195647` unless otherwise
noted.

```bash
git fetch upstream master
```

Result: pass. `upstream/master` remained `c37c1cd6`.

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

Result: pass, 20 tests passed. Latest run compiled in 3m 15s and the test
binary finished in 14.49s.

```bash
cargo test -p warp --lib localization_tests -- --nocapture
```

Result: pass as a compile/filter check, but it runs 0 tests because the current
app test module is registered as `localization::tests`. Latest run was on
pre-rebase `c2d9ddf8`, compiled in 1m 19s, and reported 4664 filtered tests.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: pass, 8 tests passed with 4656 filtered tests. Latest run compiled in
26m 44s and the filtered test run finished in 2.79s.

```bash
cargo check -p warp --lib --message-format=short
```

Result: pass, 5m 31s.

```bash
cargo build -p integration --bin integration
```

Result: pass on local evidence head `36b9717d`, 46m 26s.

```bash
WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 \
WARP_INTEGRATION_TEST_ARTIFACTS_DIR="$PWD/target/zh-cn-visual-artifacts" \
WARP_INTEGRATION=1 \
target/debug/integration test_zh_cn_localization_visual_smoke
```

Result: pass on local evidence head `36b9717d`, exit code 0. The run used a
real display, executed 15 steps, and asserted localized app menu and Dock menu
titles.

## Visual Evidence

Latest local screenshot artifacts:

- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/settings-appearance-language-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/terminal-input-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/context-chips-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/command-search-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/agent-input-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/command-palette-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/toast-zh-cn.png`
- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-27T22-44-21/dialog-launch-config-zh-cn.png`

Each artifact is a `1280 x 800` PNG. These files are local artifacts under
`target/`; they were not pushed or attached to any PR.

## Catalog And Terminology Checks

- `en-US` keys: 5576
- `zh-CN` keys: 5576
- Missing keys: 0
- Extra keys: 0
- Placeholder mismatches: 0
- Identical values: 60
- ASCII-only zh-CN values: 71
- ASCII-with-CJK zh-CN values: 1745

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
