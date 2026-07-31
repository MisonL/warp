use fuzzy_match::FuzzyMatchResult;
use ordered_float::OrderedFloat;
use warp_localization::LocaleId;
use warpui::elements::{
    ConstrainedBox, Container, CrossAxisAlignment, Flex, Icon, ParentElement, Text,
};
use warpui::{AppContext, Element, SingletonEntity};

use crate::appearance::Appearance;
use crate::code_review::diff_state::DiffMode;
use crate::search::ai_context_menu::mixer::AIContextMenuSearchableAction;
use crate::search::ai_context_menu::styles;
use crate::search::item::SearchItem;
use crate::search::result_renderer::ItemHighlightState;

#[derive(Debug, Clone)]
pub struct DiffSetSearchItem {
    pub diff_mode: DiffMode,
    pub match_result: FuzzyMatchResult,
}

impl DiffSetSearchItem {
    pub fn name(&self) -> String {
        self.name_for_locale(LocaleId::EnUs)
    }

    fn name_for_locale(&self, locale: LocaleId) -> String {
        match &self.diff_mode {
            DiffMode::Head => {
                crate::localization::text_for_locale(locale, "search.diffset.name.uncommitted")
            }
            DiffMode::MainBranch => {
                crate::localization::text_for_locale(locale, "search.diffset.name.main_branch")
            }
            DiffMode::OtherBranch(branch) => crate::localization::text_for_locale_with_args(
                locale,
                "search.diffset.name.other_branch",
                &[("branch", branch)],
            ),
        }
    }

    pub fn description(&self) -> String {
        self.description_for_locale(LocaleId::EnUs)
    }

    fn description_for_locale(&self, locale: LocaleId) -> String {
        match &self.diff_mode {
            DiffMode::Head => crate::localization::text_for_locale(
                locale,
                "search.diffset.description.uncommitted",
            ),
            DiffMode::MainBranch => crate::localization::text_for_locale(
                locale,
                "search.diffset.description.main_branch",
            ),
            DiffMode::OtherBranch(branch) => crate::localization::text_for_locale_with_args(
                locale,
                "search.diffset.description.other_branch",
                &[("branch", branch)],
            ),
        }
    }

    fn localized_name(&self, app: &AppContext) -> String {
        self.name_for_locale(crate::localization::current_locale(app))
    }

    fn localized_description(&self, app: &AppContext) -> String {
        self.description_for_locale(crate::localization::current_locale(app))
    }
}

impl SearchItem for DiffSetSearchItem {
    type Action = AIContextMenuSearchableAction;

    fn render_icon(
        &self,
        highlight_state: ItemHighlightState,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        Container::new(
            ConstrainedBox::new(
                Icon::new(
                    "bundled/svg/diff.svg",
                    highlight_state.icon_fill(appearance).into_solid(),
                )
                .finish(),
            )
            .with_width(styles::ICON_SIZE)
            .with_height(styles::ICON_SIZE)
            .finish(),
        )
        .with_margin_right(styles::MARGIN_RIGHT)
        .finish()
    }

    fn render_item(
        &self,
        highlight_state: ItemHighlightState,
        app: &AppContext,
    ) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);

        let name_text = Text::new(
            self.localized_name(app),
            appearance.ui_font_family(),
            appearance.monospace_font_size() - 1.0,
        )
        .with_color(highlight_state.main_text_fill(appearance).into_solid());

        let description_text = Text::new(
            self.localized_description(app),
            appearance.ui_font_family(),
            appearance.monospace_font_size() - 2.0,
        )
        .with_color(highlight_state.sub_text_fill(appearance).into_solid());

        Flex::row()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(name_text.finish())
            .with_child(
                Container::new(description_text.finish())
                    .with_padding_left(6.)
                    .finish(),
            )
            .finish()
    }

    fn priority_tier(&self) -> u8 {
        // Prioritize diffsets above other items.
        1
    }

    fn score(&self) -> OrderedFloat<f64> {
        OrderedFloat(self.match_result.score as f64)
    }

    fn accept_result(&self) -> Self::Action {
        AIContextMenuSearchableAction::InsertDiffSet {
            diff_mode: self.diff_mode.clone(),
        }
    }

    fn execute_result(&self) -> Self::Action {
        self.accept_result()
    }

    fn accessibility_label(&self) -> String {
        crate::localization::text_for_locale_with_args(
            LocaleId::EnUs,
            "search.diffset.a11y.label",
            &[("name", &self.name()), ("description", &self.description())],
        )
    }

    fn accessibility_label_for_app(&self, app: &AppContext) -> String {
        crate::localization::text_for_app_with_args(
            app,
            "search.diffset.a11y.label",
            &[
                ("name", &self.localized_name(app)),
                ("description", &self.localized_description(app)),
            ],
        )
    }
}

#[cfg(test)]
#[path = "search_item_tests.rs"]
mod tests;
