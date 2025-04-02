use dioxus_router::prelude::navigator;
use freya::dioxus_core;
use freya::prelude::*;
use tracing::trace;

use crate::CLIENT;
use crate::MatrixClientState;
use crate::components::additional_authorization::AdditonalAuthHandler;
use crate::components::additional_authorization::AuthenticationState;
use crate::components::form::FormField;
use crate::hook::CommonUserAuthData;
use crate::hook::connect::use_matrix_connect;
use crate::hook::login::use_matrix_login;
use crate::hook::submit_additional_auth::AdditionalAuthType;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let mut form_url = use_signal(|| String::from("http://127.0.0.1:8008"));
    let mut form_username = use_signal(|| String::new());
    let mut form_password = use_signal(|| String::new());
    let navigator = navigator();

    let (mut get_connect_err, run_matrix_connect) = use_matrix_connect(move || {
        //navigator.push("/login");
    });

    let on_connect = {
        let mut run_matrix_connect = run_matrix_connect.clone();
        move |_| {
            let url = form_url();
            run_matrix_connect(url);
        }
    };

    use_effect({
        let mut run_matrix_connect = run_matrix_connect.clone();
        move || {
            let url = &*form_url.read_unchecked();
            trace!("noooooooooo {}", url);
            run_matrix_connect(url.clone());
            // let url = &*form_url.read_unchecked();
            // run_matrix_connect(url.clone());
        }
    });

    let errors = use_memo(move || {
        let a = get_connect_err();
        if !a.is_empty() {
            return a;
        }

        let b = CLIENT();
        if let MatrixClientState::Error(err) = b {
            return err;
        }

        String::new()
    });

    let mut form = use_signal(LoginForm::default);

    let (error_string, mut run_matrix_login, state_machine) = use_matrix_login(move || {});

    // let on_login = move |_| {
    //     let auth_data = CommonUserAuthData {
    //         username: form().username,
    //         password: form().password,
    //         session_id: None,
    //     };
    //     run_matrix_login(auth_data);
    // };

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

                    "Login"
                }

                FormField {
                    name: "Server",
                    value: form_url,
                    onchange: move |txt| {
                        *form_url.write() = txt;
                        get_connect_err.write().clear();
                     },
                }

                match CLIENT() {
                    MatrixClientState::Connecting => {
                        rsx!(
                            Loader {}
                            label {
                                "witf"
                            }

                        )
                    }
                    MatrixClientState::Connected(_) => {
                        rsx!(
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
                                        background: Some(Cow::Borrowed("#6ddbff")),
                                        hover_background: Some(Cow::Borrowed("rgb(88, 176, 206)")),
                                        border_fill: Some(Cow::Borrowed("rgb(109, 219, 255, 0)")),
                                        padding: Some(Cow::Borrowed("5 20")),
                                        ..Default::default()
                                    },
                                    onclick: on_connect,
                                    label {
                                        font_size: "24",

                                        color: "white",
                                        font_weight: "bold",
                                        "Sign In"
                                    }
                                }

                                Link {
                                    to: crate::Route::Register,


                                    rect {
                                        content: "flex",
                                        direction: "verticle",


                                        label {
                                            color: "#454545",
                                            font_size: "16",

                                            "Or register"
                                        }

                                        rect {
                                            width: "85",
                                            height: "2",
                                            background: "#454545",
                                        }
                                    }
                                }
                            }
                        )
                    }
                    MatrixClientState::Error(err) => {
                        rsx!(
                            Button {
                                theme: ButtonThemeWith {
                                    background: Some(Cow::Borrowed("#6ddbff")),
                                    hover_background: Some(Cow::Borrowed("rgb(88, 176, 206)")),
                                    border_fill: Some(Cow::Borrowed("rgb(109, 219, 255, 0)")),
                                    padding: Some(Cow::Borrowed("5 20")),
                                    ..Default::default()
                                },
                                onclick: on_connect,
                                label {
                                    font_size: "24",

                                    color: "white",
                                    font_weight: "bold",
                                    "Try Again"
                                }
                            }

                            label {
                                color: "red",

                                "error connecting: {err}"
                            }
                        )
                    }
                    MatrixClientState::Disconnected => {
                        rsx!(
                            Button {
                                theme: ButtonThemeWith {
                                    background: Some(Cow::Borrowed("#6ddbff")),
                                    hover_background: Some(Cow::Borrowed("rgb(88, 176, 206)")),
                                    border_fill: Some(Cow::Borrowed("rgb(109, 219, 255, 0)")),
                                    padding: Some(Cow::Borrowed("5 20")),
                                    ..Default::default()
                                },
                                onclick: on_connect,
                                label {
                                    font_size: "24",

                                    color: "white",
                                    font_weight: "bold",
                                    "Connect"
                                }
                            }
                        )
                    }
                }
            }
        }


    }
}
