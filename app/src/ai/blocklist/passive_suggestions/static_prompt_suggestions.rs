use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;
use warp_localization::LocaleId;

use crate::localization;
use crate::terminal::view::PromptSuggestion;

pub struct StaticPromptSuggestion {
    pub name: &'static str,
    pub pattern: &'static str,
    pub label_key: Option<&'static str>,
    pub query_key: &'static str,
}

/// Attempts to match a terminal command against predefined static prompt suggestions.
///
/// If the command matches a static rule, this returns a [`SuggestedQuery`] with details from the
/// command substituted into the rule's query template.
pub fn static_suggested_query(command: &str, locale: LocaleId) -> Option<PromptSuggestion> {
    // Try each rule in turn and apply the first match.
    for pattern in &*RULE_PATTERNS {
        if let Some(captures) = pattern.regex.captures(command) {
            // If there's a match, apply placeholders to the query.
            let label = pattern.rule.label_key.map(|key| {
                let template = localization::text_for_locale(locale, key);
                apply_captures(template.as_str(), &captures)
            });
            let query_template =
                localization::text_for_locale(LocaleId::EnUs, pattern.rule.query_key);
            let query = apply_captures(query_template.as_str(), &captures);

            return Some(PromptSuggestion {
                id: Uuid::new_v4().to_string(),
                label,
                prompt: query,
                coding_query_context: None,
                static_prompt_suggestion_name: Some(pattern.rule.name.to_string()),
                should_start_new_conversation: false,
            });
        }
    }

    None
}

/// A static prompt suggestion with its pattern precompiled to a [`Regex`].
struct StaticPromptRule {
    rule: &'static StaticPromptSuggestion,
    regex: Regex,
}

static RULE_PATTERNS: LazyLock<Vec<StaticPromptRule>> = LazyLock::new(|| {
    STATIC_RULES
        .iter()
        .map(|rule| match Regex::new(rule.pattern) {
            Ok(regex) => StaticPromptRule { rule, regex },
            Err(e) => {
                panic!(
                    "Invalid pattern for static prompt rule `{}`: {}",
                    rule.name, e
                );
            }
        })
        .collect()
});

static STATIC_RULES: &[StaticPromptSuggestion] = &[
    // git checkout -b <branch>: Checks out a new branch named <branch>.
    StaticPromptSuggestion {
        name: "GIT_CHECKOUT_NEW_BRANCH",
        pattern: r"^git\s+checkout\s+-b\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.git_checkout_new_branch.label"),
        query_key: "terminal.passive_suggestion.static.git_checkout_new_branch.query",
    },
    // git clone <repo>: Clones a repository named <repo>.
    StaticPromptSuggestion {
        name: "GIT_CLONE",
        pattern: r"^git\s+clone\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.git_clone.label"),
        query_key: "terminal.passive_suggestion.static.git_clone.query",
    },
    // git switch -c <branch>: Creates and switches to a new branch named <branch>.
    StaticPromptSuggestion {
        name: "GIT_SWITCH_NEW_BRANCH",
        pattern: r"^git\s+switch\s+-c\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.git_switch_new_branch.label"),
        query_key: "terminal.passive_suggestion.static.git_switch_new_branch.query",
    },
    // git push: Pushes changes to a remote repository.
    StaticPromptSuggestion {
        name: "GIT_PUSH",
        pattern: r"^git\s+push\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.git_push.query",
    },
    // git init: Initializes a new, empty Git repository.
    StaticPromptSuggestion {
        name: "GIT_INIT",
        pattern: r"^git\s+init\s*$",
        label_key: Some("terminal.passive_suggestion.static.git_init.label"),
        query_key: "terminal.passive_suggestion.static.git_init.query",
    },
    // npm init / yarn init / pnpm init: Initializes a Node.js project.
    StaticPromptSuggestion {
        name: "NODE_PACKAGE_INIT",
        pattern: r"^(npm|yarn|pnpm)\s+init\s*$",
        label_key: Some("terminal.passive_suggestion.static.node_package_init.label"),
        query_key: "terminal.passive_suggestion.static.node_package_init.query",
    },
    // npx create-react-app <project>: Creates a new React app called <project>.
    StaticPromptSuggestion {
        name: "NPX_CREATE_REACT_APP",
        pattern: r"^npx\s+create-react-app\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.npx_create_react_app.label"),
        query_key: "terminal.passive_suggestion.static.npx_create_react_app.query",
    },
    // npx create-next-app <project>: Creates a new Next.js app called <project>.
    StaticPromptSuggestion {
        name: "NPX_CREATE_NEXT_APP",
        pattern: r"^npx\s+create-next-app\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.npx_create_next_app.label"),
        query_key: "terminal.passive_suggestion.static.npx_create_next_app.query",
    },
    // cargo new <project>: Creates a new Rust package named <project>.
    StaticPromptSuggestion {
        name: "CARGO_NEW_PROJECT",
        pattern: r"^cargo\s+new\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.cargo_new_project.label"),
        query_key: "terminal.passive_suggestion.static.cargo_new_project.query",
    },
    // poetry new <project>: Creates a new Poetry-based Python project named <project>.
    StaticPromptSuggestion {
        name: "POETRY_NEW_PROJECT",
        pattern: r"^poetry\s+new\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.poetry_new_project.label"),
        query_key: "terminal.passive_suggestion.static.poetry_new_project.query",
    },
    // django-admin startproject <project>: Creates a new Django project named <project>.
    StaticPromptSuggestion {
        name: "DJANGO_START_PROJECT",
        pattern: r"^django-admin\s+startproject\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.django_start_project.label"),
        query_key: "terminal.passive_suggestion.static.django_start_project.query",
    },
    // rails new <app>: Creates a new Rails app named <app>.
    StaticPromptSuggestion {
        name: "RAILS_NEW_APP",
        pattern: r"^rails\s+new\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.rails_new_app.label"),
        query_key: "terminal.passive_suggestion.static.rails_new_app.query",
    },
    // gradle init / mvn archetype:generate: Initializes a Gradle or Maven project.
    StaticPromptSuggestion {
        name: "JAVA_PROJECT_INIT",
        pattern: r"^(gradle\s+init|mvn\s+archetype:generate)\s*$",
        label_key: Some("terminal.passive_suggestion.static.java_project_init.label"),
        query_key: "terminal.passive_suggestion.static.java_project_init.query",
    },
    // go mod init <module>: Initializes a new Go module named <module>.
    StaticPromptSuggestion {
        name: "GO_MOD_INIT",
        pattern: r"^go\s+mod\s+init\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.go_mod_init.label"),
        query_key: "terminal.passive_suggestion.static.go_mod_init.query",
    },
    // swift package init: Initializes a new Swift package.
    StaticPromptSuggestion {
        name: "SWIFT_PACKAGE_INIT",
        pattern: r"^swift\s+package\s+init\s*$",
        label_key: Some("terminal.passive_suggestion.static.swift_package_init.label"),
        query_key: "terminal.passive_suggestion.static.swift_package_init.query",
    },
    // terraform init: Initializes Terraform in the current directory.
    StaticPromptSuggestion {
        name: "TERRAFORM_INIT",
        pattern: r"^terraform\s+init\s*$",
        label_key: Some("terminal.passive_suggestion.static.terraform_init.label"),
        query_key: "terminal.passive_suggestion.static.terraform_init.query",
    },
    // prisma init: Initializes Prisma in the current project.
    StaticPromptSuggestion {
        name: "PRISMA_INIT",
        pattern: r"^prisma\s+init\s*$",
        label_key: Some("terminal.passive_suggestion.static.prisma_init.label"),
        query_key: "terminal.passive_suggestion.static.prisma_init.query",
    },
    // python -m venv <env_name>: Creates a new Python virtual environment named <env_name>.
    StaticPromptSuggestion {
        name: "PYTHON_CREATE_VENV",
        pattern: r"^python\s+-m\s+venv\s+(\S+)\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.python_create_venv.query",
    },
    // bundle init: Creates a new Gemfile (Ruby Bundler).
    StaticPromptSuggestion {
        name: "BUNDLE_INIT",
        pattern: r"^bundle\s+init\s*$",
        label_key: Some("terminal.passive_suggestion.static.bundle_init.label"),
        query_key: "terminal.passive_suggestion.static.bundle_init.query",
    },
    // ollama pull <model>: Pulls an Ollama model named <model>.
    StaticPromptSuggestion {
        name: "OLLAMA_PULL_MODEL",
        pattern: r"^ollama\s+pull\s+(\S+)\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.ollama_pull_model.query",
    },
    // kubectl top nodes: Shows node resource usage in Kubernetes.
    StaticPromptSuggestion {
        name: "KUBECTL_TOP_NODES",
        pattern: r"^kubectl\s+top\s+(nodes|node|no)\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.kubectl_top_nodes.query",
    },
    // kubectl top pods: Shows pod resource usage in Kubernetes.
    StaticPromptSuggestion {
        name: "KUBECTL_TOP_PODS",
        pattern: r"^kubectl\s+top\s+(pods|po|pod)\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.kubectl_top_pods.query",
    },
    // kubectl get...: Gets Kubernetes resources (any).
    StaticPromptSuggestion {
        name: "KUBECTL_GET_RESOURCES",
        pattern: r"^kubectl\s+get.*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.kubectl_get_resources.query",
    },
    // docker ps: Lists Docker containers.
    StaticPromptSuggestion {
        name: "DOCKER_LIST_CONTAINERS",
        pattern: r"^docker\s+ps\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.docker_list_containers.query",
    },
    // docker image ls: Lists Docker images.
    StaticPromptSuggestion {
        name: "DOCKER_LIST_IMAGES",
        pattern: r"^docker\s+image\s+ls\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.docker_list_images.query",
    },
    // docker-compose up -d <service>: Spins up a service <service> in Docker Compose.
    StaticPromptSuggestion {
        name: "DOCKER_COMPOSE_UP_SERVICE",
        pattern: r"^docker-compose\s+up\s+-d\s+(\S+)\s*$",
        label_key: Some("terminal.passive_suggestion.static.docker_compose_up_service.label"),
        query_key: "terminal.passive_suggestion.static.docker_compose_up_service.query",
    },
    // docker network create <network>: Creates a Docker network named <network>.
    StaticPromptSuggestion {
        name: "DOCKER_NETWORK_CREATE",
        pattern: r"^docker\s+network\s+create\s+(\S+)\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.docker_network_create.query",
    },
    // vagrant init <box>: Initializes a Vagrant box named <box>.
    StaticPromptSuggestion {
        name: "VAGRANT_INIT_BOX",
        pattern: r"^vagrant\s+init\s+(\S+)\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.vagrant_init_box.query",
    },
    // vagrant up: Brings up a Vagrant environment.
    StaticPromptSuggestion {
        name: "VAGRANT_UP",
        pattern: r"^vagrant\s+up\s*$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.vagrant_up.query",
    },
    // grep -r <pattern>: Searches recursively for <pattern> in files.
    StaticPromptSuggestion {
        // Capture everything after `grep -r ` into capture group 1.
        name: "GREP_RECURSIVE_SEARCH",
        pattern: r"^grep\s+-r\s+(.*)$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.grep_recursive_search.query",
    },
    // find <args>: Searches for files/directories using `find`.
    StaticPromptSuggestion {
        // Capture everything after `find ` into capture group 1.
        // E.g. `find . -name "*.rs"`.
        name: "FIND_FILES",
        pattern: r"^find\s+(.*)$",
        label_key: None,
        query_key: "terminal.passive_suggestion.static.find_files.query",
    },
    // ssh-keygen (no args): Generates an SSH key with default options.
    StaticPromptSuggestion {
        // This pattern matches "ssh-keygen" by itself or anything after it (e.g. "-t rsa -b 4096").
        name: "SSH_KEYGEN",
        pattern: r"^ssh-keygen(?:\s+(.*))?$",
        // We'll keep the label/query generic so it applies whether or not the user passed extra flags.
        // Not using the capture group here, but it's there if we need it for the future.
        label_key: None,
        query_key: "terminal.passive_suggestion.static.ssh_keygen.query",
    },
];

pub fn apply_captures(template: &str, captures: &regex::Captures) -> String {
    // We'll look for placeholders of the form `{1}`, `{2}`, etc. and replace them with the
    // corresponding capture group.
    let mut result = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        rest = &rest[start + 1..];

        let Some(end) = rest.find('}') else {
            result.push('{');
            result.push_str(rest);
            return result;
        };

        let index = &rest[..end];
        if let Ok(index) = index.parse::<usize>() {
            if index > 0 {
                if let Some(capture) = captures.get(index) {
                    result.push_str(capture.as_str());
                    rest = &rest[end + 1..];
                    continue;
                }
            }
        }

        result.push('{');
        result.push_str(index);
        result.push('}');
        rest = &rest[end + 1..];
    }

    result.push_str(rest);
    result
}
