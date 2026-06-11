mod legacy;
mod maa;
mod static_prompt_suggestions;

pub use legacy::{
    PassiveSuggestionsEvent as LegacyPassiveSuggestionsEvent,
    PassiveSuggestionsModel as LegacyPassiveSuggestionsModel,
};
pub use maa::{
    PassiveSuggestionsEvent as MaaPassiveSuggestionsEvent,
    PassiveSuggestionsModel as MaaPassiveSuggestionsModel,
};
#[cfg(feature = "integration_tests")]
pub(crate) use static_prompt_suggestions::{
    apply_captures as apply_static_prompt_captures_for_integration_test,
    static_suggested_query as static_suggested_query_for_integration_test,
};
use warpui::ModelHandle;

#[derive(Clone)]
pub struct PassiveSuggestionsModels {
    pub legacy: ModelHandle<LegacyPassiveSuggestionsModel>,
    pub maa: ModelHandle<MaaPassiveSuggestionsModel>,
}
