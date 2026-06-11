use super::{
    connection_label_from_session_hosts, connection_label_from_ssh_host,
    connection_label_from_user_and_host,
};

const FALLBACK_HOST_LABEL: &str = "Remote host";

#[test]
fn connection_label_prefers_ssh_host_over_reported_hostname() {
    assert_eq!(
        connection_label_from_session_hosts(
            "moira",
            "remote-reported-hostname",
            Some("ssh-user@devbox.namespace"),
            FALLBACK_HOST_LABEL,
        ),
        "moira@devbox.namespace"
    );
    assert_eq!(
        connection_label_from_session_hosts(
            "moira",
            "remote-reported-hostname",
            None,
            FALLBACK_HOST_LABEL,
        ),
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
        connection_label_from_user_and_host("kevinyang", Some("ssh-testing"), FALLBACK_HOST_LABEL),
        "kevinyang@ssh-testing"
    );
    assert_eq!(
        connection_label_from_user_and_host("kevinyang", None, FALLBACK_HOST_LABEL),
        "kevinyang"
    );
    assert_eq!(
        connection_label_from_user_and_host("", Some("ssh-testing"), FALLBACK_HOST_LABEL),
        "ssh-testing"
    );
    assert_eq!(
        connection_label_from_user_and_host("", None, FALLBACK_HOST_LABEL),
        FALLBACK_HOST_LABEL
    );
}
