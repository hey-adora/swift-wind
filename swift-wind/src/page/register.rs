use dioxus_router::prelude::navigator;
use freya::dioxus_core;
use freya::prelude::*;
use tracing::debug;
use tracing::trace;

use crate::CLIENT;
use crate::MatrixClientState;
use crate::components::additional_authorization::AdditonalAuthHandler;
use crate::components::additional_authorization::AuthenticationState;
use crate::hook::CommonUserAuthData;
use crate::hook::register::use_matrix_register;
use crate::hook::submit_additional_auth::AdditionalAuthType;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Register() -> Element {
    let mut form = use_signal(LoginForm::default);
    let navigator = navigator();

    use_effect(move || {
        let MatrixClientState::Connected(client) = CLIENT() else {
            return;
        };
        spawn(async move {
            let types = client.matrix_auth().get_login_types().await;
            debug!("TYPES: {:#?}", types);
        });
    });

    let (error_string, mut run_matrix_register, state_machine) = use_matrix_register(move || {
        trace!("wwowza yowza mowza");
    });

    let on_register = move |_| {
        let auth_data = CommonUserAuthData {
            username: form().username,
            password: form().password,
            session_id: None,
        };
        run_matrix_register(auth_data);
    };

    rsx! {
        rect {
            height: "100%",
            width: "100%",
            main_align: "center",
            cross_align: "center",

            if let Some(auth_state) = state_machine.read().as_ref() {

                match auth_state{
                    AuthenticationState::Authorized { .. } => {
                        // navigator.push("main_interface");
                        rsx!{label { "Account created, loading main interface" }}
                    },
                    AuthenticationState::AdditionalAuthRequired { chosen_flow:_, common_user_data } => {
                        rsx!{AdditonalAuthHandler { state: state_machine, additional_auth_type: AdditionalAuthType::Register(common_user_data.clone()) }}
                    },
                }
            }
            else{
                rect {
                    content: "flex",
                    direction: "vertical",
                    spacing: "10",

                    label {
                        font_size: "24",

                        "Register"
                    }

                    Input {
                        value: form().username,
                        placeholder: "username",
                        onchange: move |txt| {
                            form.write().username = txt;
                        }
                    }
                    Input {
                        value: form().password,
                        placeholder: "password",
                        mode: InputMode::Hidden('*'),
                        onchange: move |txt| {
                            form.write().password = txt;
                        }
                    }
                    rect {
                        width: "100%",
                        content: "flex",
                        direction: "horizontal",
                        main_align: "space-between",

                        Button {
                            onclick: on_register,
                            label {
                                "Register"
                            }
                        }

                        Link {
                            to: crate::Route::Login,

                            label {
                                "Login"
                            }
                        }
                    }
                }
                label {
                    color: "red",
                    "{error_string}"
                }
            }
        }
    }
}
