use std::fmt::Debug;
use std::path::PathBuf;

use fuzzy_match::FuzzyMatchResult;
use ordered_float::OrderedFloat;
use warpui::elements::{ConstrainedBox, Container, Icon};
use warpui::{AppContext, Element};

use crate::appearance::Appearance;
use crate::search::ai_context_menu::mixer::AIContextMenuSearchableAction;
use crate::search::ai_context_menu::styles;
use crate::search::files::icon::icon_from_file_path;
use crate::search::item::SearchItem;
use crate::search::result_renderer::ItemHighlightState;
use crate::ui_components::render_file_search_row::{render_file_search_row, FileSearchRowOptions};

#[derive(Debug)]
pub struct FileSearchItem {
    pub path: PathBuf,
    pub match_result: FuzzyMatchResult,
    pub is_directory: bool,
}

impl SearchItem for FileSearchItem {
    type Action = AIContextMenuSearchableAction;

    fn render_icon(
        &self,
        highlight_state: ItemHighlightState,
        appearance: &Appearance,
    ) -> Box<dyn Element> {
        Container::new(
            ConstrainedBox::new(if self.is_directory {
                Icon::new(
                    "bundled/svg/completion-folder.svg",
                    highlight_state.icon_fill(appearance).into_solid(),
                )
                .finish()
            } else {
                icon_from_file_path(&self.path.to_string_lossy(), appearance, highlight_state)
            })
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
        render_file_search_row(
            &self.path,
            FileSearchRowOptions {
                match_result: Some(&self.match_result),
                highlight_state,
                ..Default::default()
            },
            app,
        )
    }

    fn score(&self) -> OrderedFloat<f64> {
        OrderedFloat(self.match_result.score as f64)
    }

    fn accept_result(&self) -> Self::Action {
        AIContextMenuSearchableAction::InsertFilePath {
            file_path: self.path.to_string_lossy().to_string(),
        }
    }

    fn execute_result(&self) -> Self::Action {
        self.accept_result()
    }

    fn accessibility_label(&self) -> String {
        let key = if self.is_directory {
            "search.files.a11y.directory"
        } else {
            "search.files.a11y.file"
        };
        crate::localization::text_for_locale_with_args(
            warp_localization::LocaleId::EnUs,
            key,
            &[("path", &self.path.display().to_string())],
        )
    }

    fn accessibility_label_for_app(&self, app: &AppContext) -> String {
        let key = if self.is_directory {
            "search.files.a11y.directory"
        } else {
            "search.files.a11y.file"
        };
        crate::localization::text_for_app_with_args(
            app,
            key,
            &[("path", &self.path.display().to_string())],
        )
    }
}
