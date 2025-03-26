use std::collections::VecDeque;

use freya::prelude::*;
use ruma::{
    OwnedDeviceId, OwnedUserId,
    api::client::uiaa::{AuthData, AuthType, Dummy},
};
use tracing::error;

use crate::hook::submit_additional_auth::*;

#[derive(Debug)]
pub enum AuthenticationState {
    Authorized {
        access_token: Option<String>,
        refresh_token: Option<String>,
        device_id: Option<OwnedDeviceId>,
        user_id: OwnedUserId,
    },
    AdditionalAuthRequired {
        chosen_flow: VecDeque<AuthType>,
        session_id: Option<String>,
    },
}
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
struct RegisterTokenForm {
    token: String,
}

// This component should collect data for authorization and display status,
// the hook should send the data to the server and wait for result
#[component]
pub fn additional_auth_handler(mut state: Signal<Option<AuthenticationState>>) -> Element {
    //Gonna need a lot more forms
    let mut token_form = use_signal(RegisterTokenForm::default);

    let (error_string, mut run_auth) = use_matrix_additional_auth(
        move |auth_res| match auth_res {
            //Go to next stage of auth, force update by popping auth type from flow
            HookAuthResult::NextStage => {
                if let AuthenticationState::AdditionalAuthRequired {
                    chosen_flow,
                    session_id: _,
                } = state.write().as_mut().unwrap()
                {
                    if chosen_flow.pop_front().is_none() {
                        error!(
                            "No more authentication flow to complete but server still request additional auth"
                        );
                    }
                }
            }
            HookAuthResult::AuthFinished {
                access_token,
                refresh_token,
                device_id,
                user_id,
            } => {
                let new_state = AuthenticationState::Authorized {
                    access_token,
                    refresh_token,
                    device_id,
                    user_id,
                };
                state.write().replace(new_state);
            }
        },
        AdditionalAuthType::Register,
    );

    match state.read().as_ref().unwrap() {
        AuthenticationState::Authorized {
            access_token,
            refresh_token,
            device_id,
            user_id,
        } => {
            rsx!()
        }
        AuthenticationState::AdditionalAuthRequired {
            chosen_flow,
            session_id,
        } => {
            // Depending on the first element choose what should be displayed
            // Then, whether through automatic or form submission give the hook the authorization data
            let current_auth_step = chosen_flow.front().unwrap();

            match current_auth_step {
                AuthType::ReCaptcha => {
                    rsx! {
                        //Probably can just remove this
                        label { "Opening recaptcha in external browser" }
                        //TODO: Open recaptcha in external browser
                        Loader{}
                    }
                }
                AuthType::EmailIdentity => todo!(),
                AuthType::Msisdn => todo!(),
                AuthType::Sso => todo!(),
                AuthType::Dummy => {
                    let mut dummy = Dummy::new();
                    dummy.session = session_id.clone();
                    run_auth(AuthData::Dummy(dummy));
                    rsx! {
                        //Probably can just remove this
                        label { "Attempting authentication" }
                        Loader{}
                    }
                }
                AuthType::RegistrationToken => {
                    rsx! {
                        label { "Enter your preshared authentication token" }
                        Input {
                            placeholder: "Token",
                            value: token_form().token,
                            onchange: move|txt|{
                                token_form.write().token = txt
                            }
                        }
                    }
                }
                AuthType::Terms => todo!(),
                _ => todo!(),
            }
        }
    }
}
