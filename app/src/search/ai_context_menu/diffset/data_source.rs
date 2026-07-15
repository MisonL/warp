use warpui::AppContext;

use super::search_item::DiffSetSearchItem;
use crate::code_review::diff_state::DiffMode;
use crate::search::ai_context_menu::mixer::AIContextMenuSearchableAction;
use crate::search::data_source::{Query, QueryResult};
use crate::search::mixer::{DataSourceRunErrorWrapper, SyncDataSource};

pub struct DiffSetDataSource;

fn searchable_label(app: &AppContext, key: &str) -> String {
    let localized = crate::localization::text_for_app(app, key);
    let english = crate::localization::text_for_locale(warp_localization::LocaleId::EnUs, key);
    if localized == english {
        localized
    } else {
        format!("{english} {localized}")
    }
}

impl SyncDataSource for DiffSetDataSource {
    type Action = AIContextMenuSearchableAction;

    fn run_query(
        &self,
        query: &Query,
        app: &AppContext,
    ) -> Result<Vec<QueryResult<Self::Action>>, DataSourceRunErrorWrapper> {
        // Filter based on query if provided
        let query_text = &query.text.to_lowercase();
        let mut results: Vec<QueryResult<Self::Action>> = vec![];

        // Add uncommitted changes option
        let uncommitted_changes = searchable_label(app, "search.diffset.name.uncommitted");
        if let Some(match_result) =
            fuzzy_match::match_indices_case_insensitive(&uncommitted_changes, query_text)
        {
            results.push(
                DiffSetSearchItem {
                    diff_mode: DiffMode::Head,
                    match_result,
                }
                .into(),
            );
        }

        // Add main branch comparison option
        let main_branch_changes = searchable_label(app, "search.diffset.name.main_branch");
        if let Some(match_result) =
            fuzzy_match::match_indices_case_insensitive(&main_branch_changes, query_text)
        {
            results.push(
                DiffSetSearchItem {
                    diff_mode: DiffMode::MainBranch,
                    match_result,
                }
                .into(),
            );
        }

        Ok(results)
    }
}

impl warpui::Entity for DiffSetDataSource {
    type Event = ();
}
