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

/// A very broad overarching state of the current page
/// Meant for switching two other components
#[derive(Default)]
enum GeneralLoginState {
    #[default]
    Register,
    Login,
}

#[derive(Default, Clone)]
enum RegisterState {
    #[default]
    EnteringData,
    ExtraAuthRequired {
        recaptcha: bool,
        shared_token: bool,
        email: bool,
        terms: bool,
        session_token: Option<String>,
    },
    RegisterComplete,
}

#[derive(Default)]
enum LoginState {
    #[default]
    EnteringData,
    ExtraAuthRequired {
        recaptcha: bool,
    },
    LoginComplete,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let mut login_state = use_signal(|| GeneralLoginState::default());
    let mut form = use_signal(|| LoginForm::default());

    rsx! {
        rect {
            height: "100%",
            width: "100%",
            main_align: "center",
            cross_align: "center",

            rect {
                content: "flex",
                direction: "vertical",
                spacing: "10",

                label {
                    font_size: "24",

                    "Login"
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
                        label {
                            "Login"
                        }
                    }

                    Link {
                        to: crate::Route::Register,

                        label {
                            "Register"
                        }
                    }
                }
            }
        }
    }
}
