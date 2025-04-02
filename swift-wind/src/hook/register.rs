use freya::prelude::*;
use matrix_sdk::HttpError;
use matrix_sdk::RumaApiError;

use ruma::api::client::account::register::RegistrationKind;
use ruma::api::client::uiaa::AuthType;
use ruma::api::error::FromHttpResponseError;
use std::collections::VecDeque;
use tracing::debug;
use tracing::error;
use tracing::trace;
use tracing::warn;

use crate::CLIENT;
use crate::MatrixClientState;
use crate::components::additional_authorization::AuthenticationState;

use super::CommonUserAuthData;

pub fn use_matrix_register<F>(
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
            warn!("trying to register before connected");
            *error_string.write() =
                "Client has not connected to server, how are you here?".to_string();
            return;
        };
        let mut callback = callback.clone();
        spawn(async move {
            let mut register_request = ruma::api::client::account::register::v3::Request::new();
            register_request.initial_device_display_name =
                Some("app.element.io: Firefox on Linux".to_string());
            // register_request.password = Some(auth_data.password.clone());
            // register_request.username = Some(auth_data.username.clone());
            // register_request.refresh_token = true;
            // register_request.kind = RegistrationKind::User;

            trace!("Sending inital register request");
            let resp = client.matrix_auth().register(register_request).await;

            debug!("what the flow: {:#?}", resp);
            return;
            //Holy error! This is what we should expect, effectively means that the user needs to do another step of auth,
            //like a recaptcha, shared token, or read terms and conditions
            if let Err(matrix_sdk::Error::Http(HttpError::Api(FromHttpResponseError::Server(
                RumaApiError::Uiaa(info),
            )))) = resp
            {
                // todo!("complex reg");
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

                trace!("Register chose auth flow: {:#?}", chosen_flow);
                *returned_state_machine.write() =
                    Some(AuthenticationState::AdditionalAuthRequired {
                        chosen_flow,
                        common_user_data: auth_data,
                    });
                callback();
            } else if resp.is_ok() {
                trace!("Inital register auth got accepted");
                *returned_state_machine.write() = Some(AuthenticationState::Authorized);
                callback();
            } else if let Err(err) = resp {
                error!("Inital register got unexpected api error: {err}");
                *error_string.write() = err.to_string();
            }
        });
    };

    (error_string, register, returned_state_machine)
}
