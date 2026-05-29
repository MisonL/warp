# zh-CN Localization Review Evidence

## Scope

This record captures the local rebase, fix, and validation state for the Warp
zh-CN localization work on branch `feat/localization-settings-upstream-rebuild`.
It is intentionally local-only: no PR, push, remote ref update, or new branch was
created in this pass.

## Current Branch State

- Branch: `feat/localization-settings-upstream-rebuild`
- Verified upstream base: `upstream/master` at `ce73fe07`
- Local source validation was run after the rebase; this evidence is included
  in the amended branch head.
- Upstream ancestry: `git merge-base --is-ancestor upstream/master HEAD`
  returned 0
- Upstream comparison after this evidence update:
  `git rev-list --left-right --count upstream/master...HEAD` returned `0 16`
- Remote branch comparison after this evidence update:
  `git rev-list --left-right --count origin/feat/localization-settings-upstream-rebuild...HEAD`
  returned `3 64`
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
- Compile drift from the latest upstream rebase was fixed in the app
  localization path without changing non-localization business semantics, and
  the branch rebased cleanly onto `upstream/master` at `ce73fe07` with no
  conflicts.

## Catalog State

Current catalog stats:

- `en-US` keys: 5684
- `zh-CN` keys: 5684
- Missing in `zh-CN`: 0
- Extra in `zh-CN`: 0
- Placeholder mismatches: 0
- Identical values: 63
- Empty values: `auth.empty` in both catalogs
- ASCII-only zh-CN values: 69
- ASCII-with-CJK zh-CN values: 2494

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
jq empty app/assets/bundled/locales/en-US.json app/assets/bundled/locales/zh-CN.json
```

Result: pass.

```bash
rg -n 'paragraph\("[^"]*[A-Za-z][^"]*"|span\("[^"]*[A-Za-z][^"]*"|link\("[^"]*[A-Za-z][^"]*"|Text::new\("[^"]*[A-Za-z][^"]*"|Text::new_inline\("[^"]*[A-Za-z][^"]*"|FormattedTextElement::from_str\("[^"]*[A-Za-z][^"]*"|button::Content::Label\("[^"]*[A-Za-z][^"]*"|wrappable_text\("[^"]*[A-Za-z][^"]*"' crates/onboarding/src -g '*.rs'
```

Result: pass. The scan produced no direct user-visible English literals in
onboarding UI constructor calls.

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

Result: pass, finished in 7m 11s.

```bash
cargo test -p warp_localization -- --nocapture
```

Result: pass, 22 tests passed. The post-rebase run compiled in 18.12s and
includes onboarding copy key coverage and the onboarding direct-English UI
literal scan.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: the test binary build finished in 25m 45s, but the binary did not
reach test output before manual termination. A follow-up run after confirming
the test binary could list tests passed as recorded below.

```bash
cargo test -p warp --lib localization::tests -- --list
```

Result: pass. The command listed 8 localization tests and 0 benchmarks after a
1.02s cached target check.

```bash
cargo test -p warp --lib localization::tests -- --nocapture
```

Result: pass, 8 tests passed with 4715 filtered tests. The cached target check
finished in 1.47s and the tests finished in 3.13s.

```bash
cargo check -p warp --lib --message-format=short
```

Result: pass, 6m 59s.

```bash
rg -n 'set_menu_header_to_static\(|Select MCP servers|Add folder' app/src crates -g '*.rs'; test $? -eq 1
```

Earlier in this pass this returned pass with no direct English header fallback
matches. It should be re-run after the final fetch/rebase check if additional
source edits are made.

## Visual Evidence

Historical real-display smoke evidence:

- `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T21-56-59/recording.log`

The historical run returned exit code 0 and the recording log shows every smoke
step and assertion succeeded, including app menu/Dock menu localization,
Settings, Terminal input focus, context chip presence, command search, Agent
input mode, command palette, workspace toast, and launch-config dialog focus.

Historical limitation: no PNG screenshots were saved in the artifact directory.
The latest complete PNG screenshot set remains the historical local artifact
directory `target/zh-cn-visual-artifacts/test_zh_cn_localization_visual_smoke/2026-05-28T16-14-58`,
but that screenshot set predates the current rebase to `ce73fe07` and is not
used as proof for the current code base.

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
  audit scope. The onboarding UI constructor scan passed with no matches, and
  the latest direct header fallback scan passed with no matches earlier in this
  pass.
- App localization tests: satisfied by
  `cargo test -p warp --lib localization::tests -- --nocapture`, 8 passed with
  4715 filtered tests.
- App compile check: satisfied by `cargo check -p warp --lib`.
- Real-display zh-CN smoke flow: historical evidence exists, but it was not
  rerun after the latest rebase.
- Real-display PNG screenshot capture: not satisfied for the current code base;
  the historical runs generated `recording.log` only.

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
- Current real-display smoke assertions were not rerun after the latest rebase,
  and PNG screenshot capture did not produce current-code screenshots on this
  machine; this remains a local visual artifact gate before claiming
  screenshot-backed visual coverage.
- Local `target/` artifacts are build and review evidence, not tracked source
  files.
