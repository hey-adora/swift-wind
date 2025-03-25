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

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Register() -> Element {
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
        }
    }
}
