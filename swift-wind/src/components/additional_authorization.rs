use std::collections::VecDeque;

use freya::prelude::*;
use ruma::api::client::uiaa::{AuthData, AuthType, Dummy, RegistrationToken};
use tracing::error;

use crate::hook::{CommonUserAuthData, submit_additional_auth::*};

pub enum AuthenticationState {
    Authorized,
    AdditionalAuthRequired {
        chosen_flow: VecDeque<AuthType>,
        common_user_data: CommonUserAuthData,
    },
}
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
struct RegisterTokenForm {
    token: String,
}

// This component should collect data for authorization and display status,
// the hook should send the data to the server and wait for result
#[component]
pub fn AdditonalAuthHandler(
    mut state: Signal<Option<AuthenticationState>>,
    additional_auth_type: AdditionalAuthType,
) -> Element {
    //Gonna need a lot more forms
    let mut token_form = use_signal(RegisterTokenForm::default);

    let (error_string, mut run_auth) = use_matrix_additional_auth(move |auth_res| match auth_res {
        //Go to next stage of auth, force update by popping auth type from flow
        HookAuthResult::NextStage => {
            if let AuthenticationState::AdditionalAuthRequired {
                chosen_flow,
                common_user_data: _,
            } = state.write().as_mut().unwrap()
            {
                if chosen_flow.pop_front().is_none() {
                    error!(
                        "No more authentication flow to complete but server still request additional auth"
                    );
                }
            }
        }
        HookAuthResult::AuthFinished => {
            state.write().replace(AuthenticationState::Authorized);
        }
    });

    rsx! {
        match state.read().as_ref().unwrap() {
            AuthenticationState::Authorized => {
                rsx! {label { "Authentication completed, please wait..." }}
            }
            AuthenticationState::AdditionalAuthRequired {
                chosen_flow,
                common_user_data,
            } => {
                // Depending on the first element choose what should be displayed
                // Then, whether through automatic or form submission give the hook the authorization data
                let current_auth_step = chosen_flow.front().unwrap();
                let common_user_data = common_user_data.clone();

                match current_auth_step {
                    AuthType::ReCaptcha => todo!(),
                    AuthType::EmailIdentity => todo!(),
                    AuthType::Msisdn => todo!(),
                    AuthType::Sso => todo!(),
                    AuthType::Dummy => {
                        let mut dummy = Dummy::new();
                        dummy.session = common_user_data.session_id.clone();
                        run_auth(AuthData::Dummy(dummy), additional_auth_type);
                        rsx! {
                            //Probably can just remove this
                            label { "Attempting authentication" }
                            Loader{}
                        }
                    }
                    AuthType::RegistrationToken => {
                        let send_registration_token = move |_| {
                            let mut token = RegistrationToken::new(token_form().token);
                            token.session = common_user_data.session_id.clone();
                            run_auth(
                                AuthData::RegistrationToken(token),
                                additional_auth_type.clone(),
                            );
                        };

                        rsx! {
                            label { "Enter your Registration Token" }
                            Input {
                                placeholder: "Token",
                                value: token_form().token,
                                onchange: move|txt|{
                                    token_form.write().token = txt
                                }
                            }
                            Button {
                                onclick: send_registration_token,
                                label {
                                    "Submit"
                                }
                            }
                        }
                    }
                    AuthType::Terms => todo!(),
                    _ => todo!(),
                }
            }
        }
        label {
            color: "red",
            "{error_string}"
        }
    }
}
