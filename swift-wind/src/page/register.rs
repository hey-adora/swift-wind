use dioxus_router::prelude::navigator;
use freya::dioxus_core;
use freya::prelude::*;
use tracing::debug;
use tracing::trace;

use crate::CLIENT;
use crate::MatrixClientState;
use crate::components::additional_authorization::AdditonalAuthHandler;
use crate::components::additional_authorization::AuthenticationState;
use crate::components::form::FormField;
use crate::hook::CommonUserAuthData;
use crate::hook::connect::use_matrix_connect;
use crate::hook::register::use_matrix_register;
use crate::hook::submit_additional_auth::AdditionalAuthType;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Register() -> Element {
    let mut form_url = use_signal(|| String::from("http://127.0.0.1:8008"));
    let mut form_username = use_signal(|| String::new());
    let mut form_password = use_signal(|| String::new());
    let navigator = navigator();

    let (mut get_matrix_connect, mut run_matrix_connect) = use_matrix_connect(move || {
        navigator.push("/login");
    });

    // use_effect(move || {
    //     let MatrixClientState::Connected(client) = CLIENT() else {
    //         return;
    //     };
    //     spawn(async move {
    //         let types = client.matrix_auth().get_login_types().await;
    //         debug!("TYPES: {:#?}", types);
    //     });
    // });

    let (error_string, mut run_matrix_register, state_machine) = use_matrix_register(move || {
        trace!("wwowza yowza mowza");
    });

    let on_register = move |_| {
        let auth_data = CommonUserAuthData {
            username: form_username(),
            password: form_password(),
            session_id: None,
        };
        run_matrix_register(auth_data);
    };

    rsx! {


        rect {
            content: "flex",
            direction: "horizontal",
            width: "100%",
            height: "100%",

            main_align: "center",

            rect {
                content: "flex",
                direction: "vertical",
                spacing: "5",
                max_width: "225",
                cross_align: "center",

                label {
                    color: "#454545",
                    font_size: "36",
                    font_weight: "bold",
                    text_align: "center",
                    margin: "0 0 20 0",

                    "Register"
                }

                FormField {
                    name: "Server",
                    value: form_url,
                    onchange: move |txt| {
                        *form_url.write() = txt;
                        get_matrix_connect.write().clear();
                     },
                }

                FormField {
                    name: "Username",
                    value: form_username,
                    onchange: move |txt| {
                        *form_username.write() = txt;
                     },
                }

                FormField {
                    name: "Password",
                    value: form_password,
                    hidden: true,
                    onchange: move |txt| {
                        *form_password.write() = txt;
                     },
                }

                rect {
                    margin: "10 0 0 0",
                    content: "flex",
                    direction: "horizontal",
                    width: "100%",
                    main_align: "space-between",
                    cross_align: "center",

                    Button {
                        theme: ButtonThemeWith {
                            background: Some(Cow::Borrowed("rgb(255, 116, 209)")),
                            hover_background: Some(Cow::Borrowed("rgb(206, 91, 167)")),
                            border_fill: Some(Cow::Borrowed("rgb(109, 219, 255, 0)")),
                            padding: Some(Cow::Borrowed("5 20")),
                            ..Default::default()
                        },
                        onclick: on_register,
                        label {
                            font_size: "24",

                            color: "white",
                            font_weight: "bold",
                            "Sign Up"
                        }
                    }

                    Link {
                        to: crate::Route::Login,


                        rect {
                            content: "flex",
                            direction: "verticle",


                            label {
                                color: "#454545",
                                font_size: "16",

                                "Or login"
                            }

                            rect {
                                width: "65",
                                height: "2",
                                background: "#454545",
                            }
                        }
                    }
                }
            }
        }



        // rect {
        //     height: "100%",
        //     width: "100%",
        //     main_align: "center",
        //     cross_align: "center",

        //     if let Some(auth_state) = state_machine.read().as_ref() {

        //         match auth_state{
        //             AuthenticationState::Authorized { .. } => {
        //                 // navigator.push("main_interface");
        //                 rsx!{label { "Account created, loading main interface" }}
        //             },
        //             AuthenticationState::AdditionalAuthRequired { chosen_flow:_, common_user_data } => {
        //                 rsx!{AdditonalAuthHandler { state: state_machine, additional_auth_type: AdditionalAuthType::Register(common_user_data.clone()) }}
        //             },
        //         }
        //     }
        //     else{
        //         rect {
        //             content: "flex",
        //             direction: "vertical",
        //             spacing: "10",

        //             label {
        //                 font_size: "24",

        //                 "Register"
        //             }

        //             Input {
        //                 value: form().username,
        //                 placeholder: "username",
        //                 onchange: move |txt| {
        //                     form.write().username = txt;
        //                 }
        //             }
        //             Input {
        //                 value: form().password,
        //                 placeholder: "password",
        //                 mode: InputMode::Hidden('*'),
        //                 onchange: move |txt| {
        //                     form.write().password = txt;
        //                 }
        //             }
        //             rect {
        //                 width: "100%",
        //                 content: "flex",
        //                 direction: "horizontal",
        //                 main_align: "space-between",

        //                 Button {
        //                     onclick: on_register,
        //                     label {
        //                         "Register"
        //                     }
        //                 }

        //                 Link {
        //                     to: crate::Route::Login,

        //                     label {
        //                         "Login"
        //                     }
        //                 }
        //             }
        //         }
        //         label {
        //             color: "red",
        //             "{error_string}"
        //         }
        //     }
        // }
    }
}
