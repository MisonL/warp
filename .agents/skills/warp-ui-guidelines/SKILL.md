---
name: warp-ui-guidelines
description: 编写 Warp client UI 代码的指南目录。在本仓库做任何 UI 工作时，都应在编写变更前先阅读，让相关指南影响实现方式。
---

# warp-ui-guidelines

本技能是 Warp UI 代码工作指南的持续积累目录。每条指南都记录了原本会在 review 中重新学到的经验，通常是因为 agent 或 contributor 重新发明了组件、偏离 design system，或绕过了共享 abstraction。

**如何使用本技能：**

- 在开始任何 UI task 时，先通读下面的 guidelines 一遍，然后在实现时记住它们。这个列表足够短，可以快速扫描。
- 每条 guideline 都是自包含的。不是每一条都适用于每个 task，请自行判断。但如果某条 guideline *确实*适用，就遵循它。
- 拿不准时，优先复用现有 abstraction，而不是引入新的。Warp UI 已积累了一组拆分良好的 shared components 和 themes；新的 one-off 几乎总会漂移。

新的 guidelines 会随时间加入这里。如果你发现某个反复出现的 UI 错误本可被书面规则捕获，请把它加进来。

---

## Guideline: 复用 button themes

Button colors 来自 `app/src/view_components/action_button.rs` 中一组共享的 `ActionButtonTheme` impls（以及 `crates/ui_components/src/button/themes.rs` 中并行的 `Theme` impls），例如 `PrimaryTheme`、`SecondaryTheme`、`NakedTheme`、`DangerPrimaryTheme` 等。这些 theme 编码了 design system，并让整个 app 的 button colors 保持一致。

给 button 设置样式时，**原样使用现有 themes 中的一个**。这些 shared themes 已经稳定并经过审查；如果某个 theme 对你的使用场景看起来 "wrong"，最可能的解释是你选错了 theme，而不是 theme 本身有 bug。

不要主动修改 shared theme。修改 `PrimaryTheme`、`SecondaryTheme` 等会影响 app 中的每个 button；一个修好当前 screen 的微调可能会静默回归其他 screen。只有在用户明确确认 design-system component 本身需要改变时，才编辑 shared theme。

你即将让 buttons 不一致的危险信号：

- 编写新的 `impl ActionButtonTheme for FooPrimaryTheme`，委托给 `PrimaryTheme`，并且只微调一个 method（通常是 `text_color`）。几乎总是应该直接使用 `PrimaryTheme` 并接受结果。
- 硬编码 `ColorU::new(...)`，而不是使用 `appearance.theme()` accessors（`accent`、`font_color(bg)`、`foreground` 等）。
- 将 `should_opt_out_of_contrast_adjustment` 设置为 `true`，以强制指定 label color。
- 按 feature 或 view 命名 theme（`FooPrimaryTheme`、`BarSubmitTheme`），而不是按 design-system role 命名。

如果现有 theme 确实不适合，并且你认为 shared theme 应该改变，请在编辑前向用户说明，而不是单方面编辑，或用 one-off 掩盖问题。
