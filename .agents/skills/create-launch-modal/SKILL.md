---
name: create-launch-modal
description: 在 Warp client 中创建一次性 launch modal（feature announcement、onboarding 等）。当新增启动时每个用户只应看到一次、由 feature flag 门控，并且颜色来自 Warp theme tokens 和 terminal theme colors 的 modal 时使用。
---

# create-launch-modal

创建一次性 launch modal，即用于 "Orchestrate any agent, anywhere" 或 "Warp is now open-source." 等发布的 feature-announcement 设计。

## 参考实现

`app/src/workspace/view/orchestration_launch_modal/` 是此模式的规范且最新示例。

## 检查清单

- [ ] `warp_features/src/lib.rs` 中的 feature flag
- [ ] `app/src/settings/ai.rs` 中的 settings 字段
- [ ] `app/src/workspace/one_time_modal_model.rs` 中的触发逻辑
- [ ] `app/src/workspace/view/<name>_launch_modal/` 下的 view 文件
- [ ] `app/src/workspace/view.rs` 和 `app/src/workspace/mod.rs` 中的 workspace 接线
- [ ] `app/src/workspace/action.rs` 中的 debug action
- [ ] `app/assets/async/png/onboarding/<name>_launch_banner.png` 处的 hero 图片
- [ ] 添加到 `crates/warp_core/src/ui/icons.rs` 的任何自定义 icon，以及 `app/assets/bundled/svg/` 中的 SVG

---

## 步骤 0 - 自定义 icon（如需要）

如果 modal 使用的 icon 尚未出现在 `Icon` enum 中，应在编写 view 前添加它们。

在 `crates/warp_core/src/ui/icons.rs` 中：

```rust
// Add to enum
YourIconName,

// Add to From<Icon> for &'static str match
Icon::YourIconName => "bundled/svg/your-icon-name.svg",
```

将 SVG 文件放到 `app/assets/bundled/svg/your-icon-name.svg`。使用与现有 icon 相同的 24x24 viewBox format。

---

## 步骤 1 - Feature flag 门控

添加到 `crates/warp_features/src/lib.rs`：

```rust
/// Enables the <name> launch modal.
<YourModalName>LaunchModal,
```

为 dogfood 启用：

```rust
pub const DOGFOOD_FLAGS: &[FeatureFlag] = &[
    FeatureFlag::<YourModalName>LaunchModal,
    // ...
];
```

---

## 步骤 2 - Settings 字段

添加到 `app/src/settings/ai.rs` 中的 `define_settings_group!(AISettings, ...)` 内。
模式：每个 modal 一个 boolean 字段，全局同步（不遵守 user sync），private。

```rust
// This is not a user-visible setting - it's merely a one-time flag to track if the
// <name> launch modal has been shown to the user.
//
// We model it as a setting so it's only shown once to a given user regardless of the number of
// devices they use.
did_check_to_trigger_<name>_launch_modal: DidShow<Name>LaunchModal {
    type: bool,
    default: false,
    supported_platforms: SupportedPlatforms::ALL,
    sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::No),
    private: true,
}
```

---

## 步骤 3 - OneTimeModalModel

文件：`app/src/workspace/one_time_modal_model.rs`

### 3a. 向 struct 添加字段

```rust
is_<name>_launch_modal_open: bool,
```

### 3b. 在 `new()` 中初始化为 false

```rust
is_<name>_launch_modal_open: false,
```

### 3c. 为新用户预先 dismiss（关键）

在 `AuthComplete` -> `!is_existing_user` 分支中，将其添加到 `AISettings::handle` update block，与其他预先 dismiss 逻辑放在一起。**如果没有这一步，新用户会在 onboarding 后第二次启动时看到 modal。**

```rust
if let Err(e) = settings
    .did_check_to_trigger_<name>_launch_modal
    .set_value(true, ctx)
{
    log::warn!("Failed to mark <name> launch modal as dismissed: {e}");
}
```

### 3d. Public API 方法

```rust
pub fn is_<name>_launch_modal_open(&self) -> bool {
    self.is_<name>_launch_modal_open && self.target_window_id.is_some()
}

pub fn mark_<name>_launch_modal_dismissed(&mut self, ctx: &mut ModelContext<Self>) {
    self.set_<name>_launch_modal_open(false, ctx);
}

#[cfg(debug_assertions)]
pub fn force_open_<name>_launch_modal(&mut self, ctx: &mut ModelContext<Self>) {
    self.set_<name>_launch_modal_open(true, ctx);
}
```

### 3e. 私有 setter

```rust
fn set_<name>_launch_modal_open(&mut self, is_open: bool, ctx: &mut ModelContext<Self>) -> bool {
    if self.is_<name>_launch_modal_open != is_open {
        self.is_<name>_launch_modal_open = is_open;
        ctx.emit(OneTimeModalEvent::VisibilityChanged { is_open });
        return true;
    }
    false
}
```

### 3f. 添加到 `is_any_modal_open`

```rust
|| self.is_<name>_launch_modal_open
```

### 3g. 触发函数

```rust
fn check_and_trigger_<name>_launch_modal(&mut self, ctx: &mut ModelContext<Self>) -> bool {
    if !FeatureFlag::<Name>LaunchModal.is_enabled() {
        return false;
    }

    let ai_settings = AISettings::as_ref(ctx);
    if *ai_settings.did_check_to_trigger_<name>_launch_modal {
        return false;
    }

    AISettings::handle(ctx).update(ctx, |settings, ctx| {
        if let Err(e) = settings
            .did_check_to_trigger_<name>_launch_modal
            .set_value(true, ctx)
        {
            log::warn!("Failed to mark <name> launch modal as dismissed: {e}");
        }
    });

    let should_show = !matches!(ChannelState::channel(), Channel::Integration);
    self.set_<name>_launch_modal_open(should_show, ctx);
    should_show
}
```

### 3h. 从 `check_and_trigger_all_modals` 调用

插入到 `check_and_trigger_hoa_onboarding` 之前：

```rust
if self.check_and_trigger_<name>_launch_modal(ctx) {
    return;
}
```

---

## 步骤 4 - View 层

创建 `app/src/workspace/view/<name>_launch_modal/mod.rs`：

```rust
mod view;
pub use view::{init, <Name>LaunchModal, <Name>LaunchModalEvent};
```

创建 `app/src/workspace/view/<name>_launch_modal/view.rs`。从 `orchestration_launch_modal/view.rs` 复制并调整。关键细节：

### 颜色来源（重要）

- modal background、text、overlay 和 border 优先使用 Warp theme tokens：
  - background surface：`appearance.theme().surface_3()`（或需要时使用其他 `surface_*` token）
  - primary/subtext：`appearance.theme().main_text_color(...)` 和 `appearance.theme().sub_text_color(...)`
  - overlays/hover fills：`appearance.theme().surface_overlay_1()` / `surface_overlay_2()`
  - subtle borders：`appearance.theme().outline()`
- 对 terminal-color accent 使用 terminal theme colors（例如 magenta launch badge accent）：
  - `appearance.theme().terminal_colors().normal.magenta`
  - `appearance.theme().ansi_overlay_1(magenta)` 用于 low-alpha background
- 避免 hardcoded hex colors。

### Hero 图片

- 存放在 `app/assets/async/png/onboarding/<name>_launch_banner.png`
- **Aspect ratio 很重要**：如果图片宽于 `MODAL_WIDTH/HERO_HEIGHT`（420/92 约 4.57），用 `Clipped::new(...)` 包裹 hero `ConstrainedBox`，避免 `cover()` 缩放时产生水平溢出。
- 预先精确裁成 420x92 的图片不需要 `Clipped`；只是更高的图片（aspect ratio < 4.57）不使用它也可以。

```rust
const MODAL_WIDTH: f32 = 420.;
const HERO_HEIGHT: f32 = 92.;
const HERO_IMAGE_PATH: &str = "async/png/onboarding/<name>_launch_banner.png";

fn render_hero(&self) -> Box<dyn Element> {
    let hero = Clipped::new(          // only needed if image ratio > 4.57
        ConstrainedBox::new(
            Image::new(AssetSource::Bundled { path: HERO_IMAGE_PATH }, CacheOption::Original)
                .with_corner_radius(CornerRadius::with_top(Radius::Pixels(8.)))
                .cover()
                .top_aligned()
                .finish(),
        )
        .with_width(MODAL_WIDTH)
        .with_height(HERO_HEIGHT)
        .finish(),
    )
    .finish();
    // ... close button overlay via Stack + add_positioned_child
}
```

### "New" badge

使用标准 badge：高度 24 px，水平 padding 8 px，font 14 px，pill corners，magenta 来自 terminal theme colors：

```rust
fn render_badge(appearance: &Appearance) -> Box<dyn Element> {
    let magenta = appearance.theme().terminal_colors().normal.magenta;
    let text = Text::new_inline("New".to_string(), appearance.ui_font_family(), 14.)
        .with_color(magenta.into())
        .finish();
    ConstrainedBox::new(
        Container::new(
            Flex::row()
                .with_cross_axis_alignment(CrossAxisAlignment::Center)
                .with_main_axis_size(MainAxisSize::Min)
                .with_child(text)
                .finish(),
        )
        .with_horizontal_padding(8.)
        .with_background(Fill::Solid(appearance.theme().ansi_overlay_1(magenta)))
        .with_corner_radius(CornerRadius::with_all(Radius::Percentage(50.)))
        .finish(),
    )
    .with_height(24.)
    .finish()
}
```

### URL

始终使用 `https://`，不要使用 `http://`：

```rust
const LEARN_MORE_URL: &str = "https://warp.dev/your-blog-link";
```

---

## 步骤 5 - Workspace 接线

### `app/src/workspace/view.rs`

```rust
// Module declaration (top)
pub(crate) mod <name>_launch_modal;

// Import
use crate::workspace::view::<name>_launch_modal::{<Name>LaunchModal, <Name>LaunchModalEvent};

// Struct field
<name>_launch_modal: ViewHandle<<Name>LaunchModal>,

// In Workspace::new()
let <name>_launch_view = ctx.add_typed_action_view(<Name>LaunchModal::new);
ctx.subscribe_to_view(&<name>_launch_view, |me, _, event, ctx| {
    me.handle_<name>_launch_modal_event(event, ctx);
});

// In struct initialization
<name>_launch_modal: <name>_launch_view,

// In OneTimeModalModel subscription handler
} else if model_ref.is_<name>_launch_modal_open() {
    me.focus_<name>_launch_modal(ctx);

// In View::render (inside the should_show_modal block)
if should_show_modal && one_time_modal_model.is_<name>_launch_modal_open() {
    stack.add_child(ChildView::new(&self.<name>_launch_modal).finish());
}
```

添加 event handler 和 focus helper：

```rust
fn handle_<name>_launch_modal_event(&mut self, event: &<Name>LaunchModalEvent, ctx: &mut ViewContext<Self>) {
    match event {
        <Name>LaunchModalEvent::Close => {
            OneTimeModalModel::handle(ctx).update(ctx, |model, ctx| {
                model.mark_<name>_launch_modal_dismissed(ctx);
            });
            self.focus_active_tab(ctx);
            ctx.notify();
        }
    }
}

fn focus_<name>_launch_modal(&mut self, ctx: &mut ViewContext<Self>) {
    ctx.focus(&self.<name>_launch_modal);
}
```

### `app/src/workspace/mod.rs`

```rust
// In pub fn init()
view::<name>_launch_modal::init(app);

// In debug bindings block
EditableBinding::new(
    "workspace:open_<name>_launch_modal",
    "[Debug] Open <Name> Launch Modal",
    WorkspaceAction::Open<Name>LaunchModal,
)
.with_context_predicate(id!("Workspace")),
EditableBinding::new(
    "workspace:reset_<name>_launch_modal_state",
    "[Debug] Reset <Name> Launch Modal State",
    WorkspaceAction::Reset<Name>LaunchModalState,
)
.with_context_predicate(id!("Workspace")),
```

---

## 步骤 6 - Debug action

在 `app/src/workspace/action.rs` 中：

```rust
/// Open the <Name> Launch Modal (for debugging)
#[cfg(debug_assertions)]
Open<Name>LaunchModal,
/// Reset the <name> launch modal dismissed state (for debugging)
#[cfg(debug_assertions)]
Reset<Name>LaunchModalState,
```

将两个 variant 都添加到 `is_visible_in_command_palette` 的 `false` arm。

在 `app/src/workspace/view.rs` 的 `TypedActionView::handle_action` 中：

```rust
#[cfg(debug_assertions)]
Open<Name>LaunchModal => {
    OneTimeModalModel::handle(ctx).update(ctx, |model, ctx| {
        model.force_open_<name>_launch_modal(ctx);
    });
    ctx.notify();
}
#[cfg(debug_assertions)]
Reset<Name>LaunchModalState => {
    AISettings::handle(ctx).update(ctx, |settings, ctx| {
        if let Err(e) = settings
            .did_check_to_trigger_<name>_launch_modal
            .set_value(false, ctx)
        {
            log::warn!("Failed to reset <name> launch modal state: {e}");
        }
    });
}
```

---

## 行为摘要

| 用户类型 | 是否看到 modal？ |
|---|---|
| 新注册用户 | 否 - 在 `AuthComplete` 新用户分支中预先 dismiss |
| 未登录用户 | 否 - 没有 `AuthComplete` 就不会触发 |
| 已有用户，flag 已启用 | 是 - cloud prefs 加载后的第一次启动 |
| `Channel::Integration` 渠道 | 否 - 由 `Channel::Integration` 检查抑制 |
| 已经看过 | 否 - setting 会在设备间全局持久化 |
