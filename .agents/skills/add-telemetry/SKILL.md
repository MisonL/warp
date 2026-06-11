---
name: add-telemetry
description: 在 Warp 代码库中添加 telemetry event，用于跟踪用户行为或系统事件。当需要为新功能加埋点、调试问题或衡量产品指标时使用。
---

# add-telemetry

## 概述

Warp 使用基于 trait 的 telemetry 系统，其中 feature-specific enum 实现 `TelemetryEvent` trait。这种方式按 domain 组织 telemetry event，而不是把所有 event 放进一个巨大的 enum。

**重要**：实现 telemetry 前，请与用户协作：
- 定义应该跟踪哪些 event，以及何时跟踪
- 确定每个 event 应包含哪些数据
- 明确 telemetry 的目的和预期用途

添加 telemetry 代码很直接，但设计有意义的 instrumentation 需要仔细思考。

## 步骤

### 1. 识别或创建 telemetry module

查找现有 feature-specific telemetry 文件（例如 `app/src/antivirus/telemetry.rs`），或为你的 feature area 创建一个新文件。

### 2. 定义 telemetry event enum

向实现 `TelemetryEvent` 的 enum 添加新 variant，或创建新 enum：

```rust
use serde_json::{json, Value};
use strum_macros::{EnumDiscriminants, EnumIter};
use warp_core::telemetry::{EnablementState, TelemetryEvent, TelemetryEventDesc};

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter))]
pub enum YourFeatureTelemetryEvent {
    ActionStarted {
        duration_ms: u64,
    },
    ActionCompleted {
        success: bool,
        error: Option<String>,
    },
}
```

### 3. 实现 TelemetryEvent trait

`EnablementState` 允许你控制何时发送 event：

- `EnablementState::Always` - 始终发送 event
- `EnablementState::Flag(FeatureFlag::YourFeature)` - 仅当 feature flag 启用时发送
- `EnablementState::Channel(Channel::Dev)` - 仅在特定 build channel 中发送

```rust
impl TelemetryEvent for YourFeatureTelemetryEvent {
    fn name(&self) -> &'static str {
        YourFeatureTelemetryEventDiscriminants::from(self).name()
    }

    fn payload(&self) -> Option<Value> {
        match self {
            Self::ActionStarted { duration_ms } => Some(json!({
                "duration_ms": duration_ms,
            })),
            Self::ActionCompleted { success, error } => Some(json!({
                "success": success,
                "error": error,
            })),
        }
    }

    fn description(&self) -> &'static str {
        YourFeatureTelemetryEventDiscriminants::from(self).description()
    }

    fn enablement_state(&self) -> EnablementState {
        YourFeatureTelemetryEventDiscriminants::from(self).enablement_state()
    }

    fn contains_ugc(&self) -> bool {
        match self {
            Self::ActionStarted { .. } => false,
            Self::ActionCompleted { .. } => false,
        }
    }

    fn event_descs() -> impl Iterator<Item = Box<dyn TelemetryEventDesc>> {
        warp_core::telemetry::enum_events::<Self>()
    }
}
```

### 4. 为 discriminant 实现 TelemetryEventDesc

```rust
impl TelemetryEventDesc for YourFeatureTelemetryEventDiscriminants {
    fn name(&self) -> &'static str {
        match self {
            Self::ActionStarted => "YourFeature.Action.Started",
            Self::ActionCompleted => "YourFeature.Action.Completed",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::ActionStarted => "User started the action",
            Self::ActionCompleted => "User completed the action",
        }
    }

    fn enablement_state(&self) -> EnablementState {
        match self {
            Self::ActionStarted | Self::ActionCompleted => EnablementState::Always,
            // Or gate behind a feature flag:
            // EnablementState::Flag(FeatureFlag::YourFeature)
        }
    }
}
```

### 5. 注册 telemetry event

在 telemetry module 末尾注册 event：

```rust
warp_core::register_telemetry_event!(YourFeatureTelemetryEvent);
```

### 6. 从代码发送 telemetry event

在带 `ViewContext` 或 `ModelContext` 的 view 或 model 中使用 `send_telemetry_from_ctx!`：

```rust
use warp_core::send_telemetry_from_ctx;

// In a view update or model method
send_telemetry_from_ctx!(
    YourFeatureTelemetryEvent::ActionStarted {
        duration_ms: 150,
    },
    ctx
);
```

对于只有 `AppContext` 的代码，改用 `send_telemetry_from_app_ctx!`。

### 7. 本地测试

使用 `log_named_telemetry_events` feature flag 运行 Warp，以在 console 中看到 telemetry event 日志：

```bash
cargo run --features log_named_telemetry_events
```

## 最佳实践

- 保持 telemetry enum 面向具体 feature，而不是添加到全局 enum
- 如果 payload 包含 user-generated content，将 `contains_ugc()` 设置为 `true`
- 使用描述性 event name，并遵循 `Feature.Action.Result` 模式
- payload 中只包含必要数据，以尽量减少带宽和存储
- 决定包含哪些数据时考虑隐私影响
- 避免使用通配符做穷尽匹配；显式处理所有 variant

## 示例参考

完整 feature-specific telemetry 实现示例见 `app/src/antivirus/telemetry.rs`。
