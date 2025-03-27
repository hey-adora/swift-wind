use freya::prelude::*;
use matrix_sdk::Client;
use matrix_sdk::reqwest::Url;
use tracing::error;
use tracing::trace;
use tracing::warn;

use crate::CLIENT;
use crate::MatrixClientState;

pub fn use_matrix_connect<F>(callback: F) -> (Signal<String>, impl FnMut(String))
where
    F: FnMut() + Clone + 'static,
{
    let mut get_connect = use_signal(String::new);

    let run_connect = move |url: String| {
        if let MatrixClientState::Connecting = CLIENT() {
            warn!("already trying to connect to matrix server");
            return;
        }
        trace!("connecting to: {url}");
        *CLIENT.write() = MatrixClientState::Connecting;
        let mut callback = callback.clone();
        spawn(async move {
            let url = match Url::parse(&url) {
                Ok(url) => url,
                Err(err) => {
                    *get_connect.write() = err.to_string();
                    *CLIENT.write() = MatrixClientState::Disconnected;
                    error!("URL parse error in connect {:?}", err);
                    return;
                }
            };
            let client = Client::builder()
                .homeserver_url(url)
                .handle_refresh_tokens()
                .build()
                .await
                .unwrap();
            let version = client.server_versions().await;
            let version = match version {
                Ok(v) => v,
                Err(err) => {
                    *CLIENT.write() = MatrixClientState::Error(err.to_string());
                    return;
                }
            };
            trace!(
                "Homeserver reports it can support protocol version(s): {:#?}",
                version
            );
            *CLIENT.write() = MatrixClientState::Connected(client);
            callback();
        });
    };

    (get_connect, run_connect)
}
