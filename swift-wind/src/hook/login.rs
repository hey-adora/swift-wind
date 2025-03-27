use freya::prelude::*;
use matrix_sdk::HttpError;
use matrix_sdk::RumaApiError;

use ruma::api::client::uiaa::AuthType;
use ruma::api::error::FromHttpResponseError;
use std::collections::VecDeque;
use tracing::error;
use tracing::trace;
use tracing::warn;

use crate::CLIENT;
use crate::MatrixClientState;
use crate::components::additional_authorization::AuthenticationState;

use super::CommonUserAuthData;

pub fn use_matrix_login<F>(
    callback: F,
) -> (
    Signal<String>,
    impl FnMut(CommonUserAuthData),
    Signal<Option<AuthenticationState>>,
)
where
    F: FnMut() + Clone + 'static,
{
    let mut error_string = use_signal(String::new);
    let mut returned_state_machine: Signal<Option<AuthenticationState>> =
        use_signal(|| Option::None);

    let register = move |mut auth_data: CommonUserAuthData| {
        let MatrixClientState::Connected(client) = CLIENT() else {
            warn!("trying to login before connected");
            *error_string.write() =
                "Client has not connected to server, how are you here?".to_string();
            return;
        };
        let mut callback = callback.clone();
        spawn(async move {
            trace!("Sending inital login request");
            let resp = client
                .matrix_auth()
                .login_username(&auth_data.username, &auth_data.password)
                .request_refresh_token()
                .await;

            if let Err(matrix_sdk::Error::Http(HttpError::Api(FromHttpResponseError::Server(
                RumaApiError::Uiaa(info),
            )))) = resp
            {
                let chosen_flow: VecDeque<AuthType> = {
                    if !info.flows.is_empty() {
                        let shortest_flow = info
                            .flows
                            .iter()
                            .min_by_key(|v| v.stages.len())
                            .unwrap()
                            .clone();
                        VecDeque::from(shortest_flow.stages)
                    } else {
                        warn!("Server asked for additional auth but provided no flow");
                        *error_string.write() =
                                "Server requires no authentication flow yet requested User Interactive Authentication. This should not happen".to_string();
                        return;
                    }
                };

                auth_data.session_id = info.session;

                trace!("Login chose auth flow: {:#?}", chosen_flow);
                *returned_state_machine.write() =
                    Some(AuthenticationState::AdditionalAuthRequired {
                        chosen_flow,
                        common_user_data: auth_data,
                    });
            } else if resp.is_ok() {
                trace!("Inital login got accepted");
                *returned_state_machine.write() = Some(AuthenticationState::Authorized);
            } else if let Err(err) = resp {
                error!("Inital login got unexpected api error: {err}");
                *error_string.write() = err.to_string();
            }

            callback();
        });
    };

    (error_string, register, returned_state_machine)
}
