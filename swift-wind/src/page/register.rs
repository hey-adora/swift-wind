use dioxus_router::prelude::navigator;
use freya::dioxus_core;
use freya::prelude::*;
use tracing::trace;

use crate::components::additional_authorization::AuthenticationState;
use crate::components::additional_authorization::additional_auth_handler;
use crate::hook::register::use_matrix_register;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Register() -> Element {
    let mut form = use_signal(LoginForm::default);
    let navigator = navigator();

    let (error_string, mut run_matrix_register, state_machine) = use_matrix_register(move || {});

    let on_register = move |_| {
        let username = form().username;
        let password = form().password;
        run_matrix_register(username, password);
    };

    rsx! {
        rect {
            height: "100%",
            width: "100%",
            main_align: "center",
            cross_align: "center",

            if let Some(auth_state) = state_machine.read().as_ref() {
                additional_auth_handler { state: state_machine }
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
                    "{error_string}"
                }
            }
        }
    }
}
