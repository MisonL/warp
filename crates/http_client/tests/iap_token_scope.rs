use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use http_client::Client;
use http_client::iap::{IAP_PROXY_AUTH_HEADER, IapTokenProvider};
use warp_core::AppId;
use warp_core::channel::{
    Channel, ChannelConfig, ChannelState, IapConfig, OzConfig, WarpServerConfig,
};

struct CountingTokenProvider {
    calls: Arc<AtomicUsize>,
}

impl IapTokenProvider for CountingTokenProvider {
    fn cached_token(&self) -> Option<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Some("staging-token".to_string())
    }
}

#[test]
fn iap_token_is_scoped_to_initial_iap_protected_origins() {
    ChannelState::set(ChannelState::new(
        Channel::Dev,
        ChannelConfig {
            app_id: AppId::new("dev", "warp", "WarpDev"),
            logfile_name: "warp_dev.log".into(),
            server_config: WarpServerConfig {
                server_root_url: "https://staging.warp.dev".into(),
                iap_protected_server_root_url: None,
                iap_protected_rtc_http_url: None,
                rtc_server_url: "wss://rtc.staging.warp.dev/graphql/v2".into(),
                session_sharing_server_url: None,
                firebase_auth_api_key: "".into(),
                iap_config: Some(IapConfig {
                    audiences: "test-audience".into(),
                    service_account_email: "test@example.com".into(),
                }),
            },
            oz_config: OzConfig {
                oz_root_url: "https://oz.warp.dev".into(),
                workload_audience_url: None,
            },
            telemetry_config: None,
            autoupdate_config: None,
            crash_reporting_config: None,
            mcp_static_config: None,
        },
    ));
    ChannelState::override_server_root_url("http://localhost:8080").unwrap();
    ChannelState::override_ws_server_url("ws://localhost:8081/graphql/v2").unwrap();

    let calls = Arc::new(AtomicUsize::new(0));
    let mut client = Client::new();
    client.set_iap_token_provider(Arc::new(CountingTokenProvider {
        calls: calls.clone(),
    }));

    assert_eq!(
        proxy_auth_header(&client, "http://localhost:8080/api/v1/graphql"),
        None
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    assert_eq!(
        proxy_auth_header(&client, "http://localhost:8081/api/v1/agent/events/stream"),
        None
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    assert_eq!(
        proxy_auth_header(&client, "https://staging.warp.dev/api/v1/graphql"),
        Some("Bearer staging-token".to_string())
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    assert_eq!(
        proxy_auth_header(
            &client,
            "https://rtc.staging.warp.dev/api/v1/agent/events/stream"
        ),
        Some("Bearer staging-token".to_string())
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

fn proxy_auth_header(client: &Client, url: &str) -> Option<String> {
    let request = client.get(url).build().unwrap();
    request
        .headers()
        .get(IAP_PROXY_AUTH_HEADER)
        .map(|value| value.to_str().unwrap().to_string())
}
