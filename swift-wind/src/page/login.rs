use freya::dioxus_core;
use freya::prelude::*;

use crate::hook::CommonUserAuthData;
use crate::hook::login::use_matrix_login;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let mut form = use_signal(LoginForm::default);
    let (error_string, mut run_matrix_login, state_machine) = use_matrix_login(move || {});

    let on_login = move |_| {
        let auth_data = CommonUserAuthData {
            username: form().username,
            password: form().password,
            session_id: None,
        };
        run_matrix_login(auth_data);
    };

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
                        onclick: on_login,
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
            label {
                color: "red",
                "{error_string}"
            }
        }
    }
}
