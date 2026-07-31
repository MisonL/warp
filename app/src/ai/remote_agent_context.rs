use std::collections::HashMap;

use ::ai::project_context::model::{ProjectContextModel, ProjectRule};
use ::ai::skills::{
    ParsedSkill, SkillProvider, SkillScope, get_provider_for_path, parse_skill_content_at_location,
};
use remote_server::manager::{RemoteServerManager, RemoteServerManagerEvent};
use remote_server::proto::{
    RemoteAgentContextSnapshot, RemoteContextFileProto, RemoteSkillProto, remote_skill_proto,
};
use warp_core::features::FeatureFlag;
use warp_core::safe_warn;
use warp_localization::LocaleId;
use warp_util::host_id::HostId;
use warp_util::local_or_remote_path::LocalOrRemotePath;
use warp_util::remote_path::RemotePath;
use warp_util::standardized_path::StandardizedPath;
use warpui::{Entity, ModelContext, SingletonEntity};

use super::mcp::McpIntegration;
use super::skills::{
    BundledSkill, BundledSkillActivation, SkillManager, localized_bundled_skill_description,
};
use crate::localization::{LocalizationUpdater, current_locale};

/// Home skills parsed from a remote agent context snapshot.
struct HomeSkills {
    home_dir: LocalOrRemotePath,
    skills: Vec<ParsedSkill>,
}

/// Valid application state parsed from a remote agent context snapshot.
struct RemoteAgentContextState {
    bundled_skills: Option<BundledSkill>,
    home_skills: Option<HomeSkills>,
    global_rules: Vec<ProjectRule>,
}

pub(crate) struct RemoteAgentContext {
    bundled_skill_protos_by_host: HashMap<HostId, Vec<RemoteSkillProto>>,
}

impl RemoteAgentContext {
    pub(crate) fn new(ctx: &mut ModelContext<Self>) -> Self {
        let remote_server_manager = RemoteServerManager::handle(ctx);
        ctx.subscribe_to_model(&remote_server_manager, |me, _, event, ctx| {
            if let RemoteServerManagerEvent::RemoteAgentContextSnapshot { host_id, snapshot } =
                event
            {
                me.reconcile_snapshot(host_id.clone(), snapshot.clone(), ctx);
                return;
            }
            if let RemoteServerManagerEvent::HostDisconnected { host_id } = event {
                me.remove_host_context(host_id, ctx);
            }
        });
        if ctx.has_singleton_model::<LocalizationUpdater>() {
            let localization_updater = LocalizationUpdater::handle(ctx);
            ctx.subscribe_to_model(&localization_updater, |me, _, _, ctx| {
                me.refresh_remote_bundled_skill_descriptions(ctx);
            });
        }
        Self {
            bundled_skill_protos_by_host: HashMap::new(),
        }
    }

    fn reconcile_snapshot(
        &mut self,
        host_id: HostId,
        snapshot: RemoteAgentContextSnapshot,
        ctx: &mut ModelContext<Self>,
    ) {
        self.bundled_skill_protos_by_host.insert(
            host_id.clone(),
            snapshot
                .skills
                .iter()
                .filter(|proto| {
                    matches!(proto.source, Some(remote_skill_proto::Source::Bundled(_)))
                })
                .cloned()
                .collect(),
        );
        let RemoteAgentContextState {
            bundled_skills,
            home_skills,
            global_rules,
        } = parse_snapshot_for_locale(&host_id, snapshot, current_locale(ctx));
        SkillManager::handle(ctx).update(ctx, |manager, ctx| {
            manager.replace_remote_agent_context(
                host_id.clone(),
                bundled_skills,
                home_skills.map(|home| (home.home_dir, home.skills)),
                ctx,
            );
        });
        ProjectContextModel::handle(ctx).update(ctx, |model, _| {
            model.set_remote_global_rules(host_id, global_rules);
        });
    }

    fn remove_host_context(&mut self, host_id: &HostId, ctx: &mut ModelContext<Self>) {
        self.bundled_skill_protos_by_host.remove(host_id);
        SkillManager::handle(ctx).update(ctx, |manager, ctx| {
            manager.remove_remote_agent_context(host_id, ctx);
        });
        ProjectContextModel::handle(ctx).update(ctx, |model, _| {
            model.remove_remote_global_rules(host_id);
        });
    }

    fn refresh_remote_bundled_skill_descriptions(&mut self, ctx: &mut ModelContext<Self>) {
        if !FeatureFlag::BundledSkills.is_enabled() {
            return;
        }

        let locale = current_locale(ctx);
        let catalogs = self
            .bundled_skill_protos_by_host
            .iter()
            .map(|(host_id, skills)| {
                (
                    host_id.clone(),
                    bundled_skill_from_protos(host_id, skills, locale),
                )
            })
            .collect::<Vec<_>>();
        SkillManager::handle(ctx).update(ctx, |manager, ctx| {
            manager.replace_remote_bundled_skill_catalogs(catalogs, ctx);
        });
    }
}

fn parse_snapshot_for_locale(
    host_id: &HostId,
    snapshot: RemoteAgentContextSnapshot,
    locale: LocaleId,
) -> RemoteAgentContextState {
    let bundled_skills = FeatureFlag::BundledSkills
        .is_enabled()
        .then(|| bundled_skill_from_protos(host_id, &snapshot.skills, locale));
    let Some(home_dir) = remote_path(host_id, &snapshot.home_dir) else {
        safe_warn!(
            safe: ("Ignoring remote home context with an invalid home directory"),
            full: ("Ignoring remote home context with an invalid home directory for {host_id}")
        );
        return RemoteAgentContextState {
            bundled_skills,
            home_skills: None,
            global_rules: Vec::new(),
        };
    };
    let skills = snapshot
        .skills
        .iter()
        .filter(|proto| matches!(proto.source, Some(remote_skill_proto::Source::Home(_))))
        .filter_map(|proto| {
            parse_remote_skill(
                host_id,
                proto,
                SkillScope::Home,
                Some(&home_dir),
                get_provider_for_path,
            )
        })
        .collect();
    let global_rules = snapshot
        .global_rules
        .into_iter()
        .filter_map(|file| project_rule_within_home(host_id, file, &home_dir))
        .collect();
    RemoteAgentContextState {
        bundled_skills,
        home_skills: Some(HomeSkills { home_dir, skills }),
        global_rules,
    }
}

fn parse_remote_skill(
    host_id: &HostId,
    proto: &RemoteSkillProto,
    scope: SkillScope,
    required_root: Option<&LocalOrRemotePath>,
    provider_for_path: impl FnOnce(&LocalOrRemotePath) -> Option<SkillProvider>,
) -> Option<ParsedSkill> {
    let Some(path) = remote_path(host_id, &proto.path) else {
        safe_warn!(
            safe: ("Skipping remote skill with an invalid path"),
            full: ("Skipping remote skill with an invalid path: {}", proto.path)
        );
        return None;
    };
    if required_root.is_some_and(|root| !path.starts_with(root)) {
        return None;
    }
    let provider = provider_for_path(&path)?;
    match parse_skill_content_at_location(path, &proto.content, provider, scope) {
        Ok(skill) => Some(skill),
        Err(err) => {
            safe_warn!(
                safe: ("Skipping remote skill that failed to parse"),
                full: ("Skipping remote skill at {} that failed to parse: {err:#}", proto.path)
            );
            None
        }
    }
}

fn bundled_skill_from_protos(
    host_id: &HostId,
    skills: &[RemoteSkillProto],
    locale: LocaleId,
) -> BundledSkill {
    let definitions = skills.iter().filter_map(|proto| {
        let remote_skill_proto::Source::Bundled(metadata) = proto.source.as_ref()? else {
            return None;
        };
        let mut skill = parse_remote_skill(
            host_id,
            proto,
            SkillScope::Bundled,
            None,
            |_| Some(SkillProvider::Warp),
        )?;
        if let Some(description) = localized_bundled_skill_description(&skill.content, locale) {
            skill.description = description;
        }
        let activation = match metadata.requires_mcp.as_deref() {
            None => BundledSkillActivation::Always,
            Some(wire_id) => match mcp_integration_from_wire_id(wire_id) {
                Some(integration) => BundledSkillActivation::RequiresMcp(integration),
                None => {
                    safe_warn!(
                        safe: ("Skipping bundled skill with an unknown MCP integration"),
                        full: ("Skipping bundled skill {} with an unknown MCP integration: {wire_id}", metadata.id)
                    );
                    return None;
                }
            },
        };
        Some((metadata.id.clone(), skill, activation))
    });
    BundledSkill::from_definitions(definitions)
}

fn mcp_integration_from_wire_id(wire_id: &str) -> Option<McpIntegration> {
    match wire_id {
        "figma" => Some(McpIntegration::Figma),
        _ => None,
    }
}

fn remote_path(host_id: &HostId, path: &str) -> Option<LocalOrRemotePath> {
    StandardizedPath::try_new(path)
        .ok()
        .map(|path| LocalOrRemotePath::Remote(RemotePath::new(host_id.clone(), path)))
}

fn project_rule_within_home(
    host_id: &HostId,
    file: RemoteContextFileProto,
    home_dir: &LocalOrRemotePath,
) -> Option<ProjectRule> {
    let path = remote_path(host_id, &file.path)?;
    path.starts_with(home_dir).then_some(ProjectRule {
        path,
        content: file.content,
    })
}

impl Entity for RemoteAgentContext {
    type Event = ();
}

impl SingletonEntity for RemoteAgentContext {}

#[cfg(test)]
#[path = "remote_agent_context_tests.rs"]
mod tests;
