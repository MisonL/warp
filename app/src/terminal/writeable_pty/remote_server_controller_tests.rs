use super::{
    AppContextLike, connection_label_from_session_hosts, connection_label_from_ssh_host,
    connection_label_from_user_and_host,
};

struct TestAppContext {
    remote_host_unknown: &'static str,
}

impl AppContextLike for TestAppContext {
    fn remote_host_unknown_text(&self) -> String {
        self.remote_host_unknown.to_string()
    }
}

const EN_CONTEXT: TestAppContext = TestAppContext {
    remote_host_unknown: "Remote host",
};

const ZH_CONTEXT: TestAppContext = TestAppContext {
    remote_host_unknown: "远程主机",
};

#[test]
fn connection_label_prefers_ssh_host_over_reported_hostname() {
    assert_eq!(
        connection_label_from_session_hosts(
            "moira",
            "remote-reported-hostname",
            Some("ssh-user@devbox.namespace"),
            &EN_CONTEXT,
        ),
        "moira@devbox.namespace"
    );
    assert_eq!(
        connection_label_from_session_hosts("moira", "remote-reported-hostname", None, &EN_CONTEXT),
        "moira@remote-reported-hostname"
    );
}

#[test]
fn connection_label_from_ssh_host_strips_user_prefix() {
    assert_eq!(
        connection_label_from_ssh_host("moira@moira.devbox.namespace"),
        "moira.devbox.namespace"
    );
    assert_eq!(
        connection_label_from_ssh_host("moira.devbox.namespace"),
        "moira.devbox.namespace"
    );
}

#[test]
fn connection_label_from_user_and_host_matches_udi_format() {
    assert_eq!(
        connection_label_from_user_and_host("kevinyang", Some("ssh-testing"), &EN_CONTEXT),
        "kevinyang@ssh-testing"
    );
    assert_eq!(
        connection_label_from_user_and_host("kevinyang", None, &EN_CONTEXT),
        "kevinyang"
    );
    assert_eq!(
        connection_label_from_user_and_host("", Some("ssh-testing"), &EN_CONTEXT),
        "ssh-testing"
    );
    assert_eq!(
        connection_label_from_user_and_host("", None, &EN_CONTEXT),
        "Remote host"
    );
    assert_eq!(
        connection_label_from_user_and_host("", None, &ZH_CONTEXT),
        "远程主机"
    );
}
