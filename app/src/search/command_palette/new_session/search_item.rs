use std::sync::Arc;

use fuzzy_match::FuzzyMatchResult;
use ordered_float::OrderedFloat;
use warpui::{AppContext, Element, SingletonEntity};

use super::new_session_option::NewSessionOption;
use crate::appearance::Appearance;
use crate::localization;
use crate::search::command_palette::mixer::CommandPaletteItemAction;
use crate::search::command_palette::render_util::render_search_item_icon;
use crate::search::result_renderer::ItemHighlightState;
use crate::ui_components::icons::Icon;

#[derive(Debug)]
pub struct SearchItem {
    match_result: FuzzyMatchResult,
    option: Arc<NewSessionOption>,
    accessibility_copy: NewSessionSearchItemAccessibilityCopy,
}

impl SearchItem {
    pub fn new(
        option: Arc<NewSessionOption>,
        match_result: FuzzyMatchResult,
        accessibility_copy: NewSessionSearchItemAccessibilityCopy,
    ) -> Self {
        Self {
            match_result,
            option,
            accessibility_copy,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewSessionSearchItemAccessibilityCopy {
    selected_template: String,
    help: String,
}

impl NewSessionSearchItemAccessibilityCopy {
    pub fn new(app: &AppContext) -> Self {
        Self {
            selected_template: localization::text_for_app(
                app,
                "search.command_palette.a11y.selected",
            ),
            help: localization::text_for_app(
                app,
                "search.command_palette.a11y.help.launch_session",
            ),
        }
    }

    fn selected_label(&self, name: &str) -> String {
        self.selected_template.replace("{name}", name)
    }
}

impl crate::search::item::SearchItem for SearchItem {
    type Action = CommandPaletteItemAction;

    fn is_multiline(&self) -> bool {
        true
    }

    fn render_icon(
        &self,
        highlight_state: ItemHighlightState,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        render_search_item_icon(
            appearance,
            Icon::Terminal,
            appearance.theme().foreground().into_solid(),
            highlight_state,
        )
    }

    fn render_item(
        &self,
        highlight_state: ItemHighlightState,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        self.option.render(
            app,
            appearance,
            highlight_state,
            self.match_result.matched_indices.clone(),
        )
    }

    fn score(&self) -> OrderedFloat<f64> {
        OrderedFloat::from(self.match_result.score as f64)
    }

    fn accept_result(&self) -> Self::Action {
        CommandPaletteItemAction::NewSession {
            source: self.option.clone(),
        }
    }

    fn execute_result(&self) -> Self::Action {
        self.accept_result()
    }

    fn accessibility_label(&self) -> String {
        self.accessibility_copy
            .selected_label(self.option.description())
    }

    fn accessibility_help_message(&self) -> Option<String> {
        Some(self.accessibility_copy.help.clone())
    }
}
