use freya::dioxus_core;
use freya::prelude::*;
use matrix_sdk::Client;
use matrix_sdk::HttpError;
use matrix_sdk::RumaApiError;
use matrix_sdk::reqwest::Url;
use ruma::api::client::account::register::RegistrationKind;
use ruma::api::client::uiaa::AuthType;
use ruma::api::client::uiaa::Dummy;
use ruma::api::error::FromHttpResponseError;
use tracing::trace;
use tracing::warn;

pub static CLIENT: GlobalSignal<MatrixClientState> = Global::new(|| MatrixClientState::default());

#[derive(Debug, Default, Clone)]
pub enum MatrixClientState {
    #[default]
    Disconnected,
    Connecting,
    Connected(Client),
    Error(String),
}

pub fn use_matrix_connect<F>(callback: F) -> (Signal<String>, impl FnMut(String))
where
    F: FnMut() + Clone + 'static,
{
    let mut get_connect = use_signal(|| String::new());

    let run_connect = move |url: String| {
        if let MatrixClientState::Connecting = CLIENT() {
            warn!("already trying to connect to matrix server");
            return;
        }
        *CLIENT.write() = MatrixClientState::Connecting;
        let mut callback = callback.clone();
        spawn(async move {
            let url = match Url::parse(&url) {
                Ok(url) => url,
                Err(err) => {
                    *get_connect.write() = err.to_string();
                    *CLIENT.write() = MatrixClientState::Disconnected;
                    return;
                }
            };
            let client = Client::new(url).await.unwrap();
            let version = client.server_versions().await;
            let version = match version {
                Ok(v) => v,
                Err(err) => {
                    *CLIENT.write() = MatrixClientState::Error(err.to_string());
                    return;
                }
            };
            trace!("connected to {:#?}", version);
            *CLIENT.write() = MatrixClientState::Connected(client);
            callback();
        });
    };

    (get_connect, run_connect)
}
