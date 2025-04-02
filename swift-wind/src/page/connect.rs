use crate::{CLIENT, MatrixClientState, components::ICON, hook::connect::use_matrix_connect};
use dioxus_router::prelude::navigator;
use freya::prelude::*;

// "http://127.0.0.1:8448"

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct ConnectFormError {
    url: String,
}

#[component]
pub fn Connect() -> Element {
    let mut form_url = use_signal(|| String::from("http://127.0.0.1:8008"));
    let mut form_username = use_signal(|| String::new());
    let mut form_password = use_signal(|| String::new());

    let navigator = navigator();
    // let mut theme = use_theme();

    // LIGHT

    let login_btn_theme = Theme {
        button: ButtonTheme {
            background: Cow::Borrowed("#6ddbff"),
            ..LIGHT_THEME.button
        },
        ..LIGHT_THEME
    };

    // let mut form_errors = use_signal(|| ConnectFormError::default());
    // let mut connection_state = use_signal(|| ConnectionState::default());
    let (mut get_matrix_connect, mut run_matrix_connect) = use_matrix_connect(move || {
        navigator.push("/login");
    });

    let on_connect = move |_| {
        let url = form_url();
        run_matrix_connect(url);
    };

    let errors = use_memo(move || {
        let a = get_matrix_connect();
        if !a.is_empty() {
            return a;
        }

        let b = CLIENT();
        if let MatrixClientState::Error(err) = b {
            return err;
        }

        String::new()
    });

    // let t = ButtonThemeWith {
    //     background: "#6ddbff",
    //     ..Default::default()
    // };

    let PlatformInformation { viewport_size, .. } = *use_platform_information().read();

    rsx! {
        rect {
            background: "linear-gradient(325deg, #fff97d 20%, #ffd978 0%, #ffd978 40%, #ff74d1 0%, #ff74d1 60%, #6ddbff 0%, #6ddbff 80%, #b276ff 0%, #b276ff 100%)",
            height: "100%",
            width: "100%",
            // main_align: "center",
            // cross_align: "center",

            content: "flex",
            direction: "horizontal",
            main_align: "space-between",


            rect {
                width: "60%",
                height: "100%",
            }

            rect {
                background: "white",
                content: "flex",
                direction: "vertical",
                spacing: "10",
                width: "40%",
                height: "100%",
                main_align: "space-between",
                padding: "20",
                corner_radius: "10",


                rect {
                    content: "flex",
                    direction: "horizontal",
                    spacing: "20",
                    height: "200",


                    svg {
                        svg_data: static_bytes(ICON),
                        width: "64",
                        height: "64",
                    }

                    rect {
                        content: "flex",
                        direction: "vertical",

                        label {
                            color: "#454545",
                            font_size: "16",

                            "Welcome to Swift-Wind!"
                        }

                        label {
                            color: "#454545",
                            font_size: "16",

                            "The open soure matrix client!"
                        }

                     }
                }

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

                        rect {
                            content: "flex",
                            direction: "vertical",
                            spacing: "5",

                            label {
                                font_size: "16",
                                color: "#454545",

                                "Server"
                            }

                            rect {

                                Input {
                                    width: "100%",
                                    value: form_url(),
                                    placeholder: "http://127.0.0.1:8008",
                                    onchange: move |txt| {
                                        *form_url.write() = txt;
                                        get_matrix_connect.write().clear();
                                    }
                                }
                                label {
                                    "{errors}"
                                 }
                            }
                        }

                        rect {
                            content: "flex",
                            direction: "vertical",
                            spacing: "5",


                            label {
                                font_size: "16",
                                color: "#454545",

                                "Username"
                            }

                            rect {
                                Input {
                                    width: "100%",
                                    value: form_username(),
                                    placeholder: "Adora",
                                    onchange: move |txt| {
                                        *form_username.write() = txt;
                                    }
                                }
                                label {
                                    ""
                                 }
                            }
                        }

                        rect {
                            content: "flex",
                            direction: "vertical",
                            spacing: "5",


                            label {
                                font_size: "16",
                                color: "#454545",

                                "Password"
                            }

                            rect {
                                Input {
                                    width: "100%",

                                    value: form_password(),
                                    mode: InputMode::Hidden('*'),
                                    onchange: move |txt| {
                                        *form_password.write() = txt;
                                    }
                                }
                                label {
                                    ""
                                 }
                            }
                        }



                        // ThemeProvider {
                        //     theme: login_btn_theme,
                        //     Button {

                        //         onclick: on_connect,
                        //         rect {
                        //             background: "#6ddbff",
                        //             label {
                        //                 "Sign In"
                        //             }
                        //         }
                        //     }
                        // }

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
                                to: crate::Route::Login,


                                rect {
                                    // border: "0 0 1 0 center #454545",
                                    // padding: "0 0 5 0",
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
                    }
                }



                rect {
                    label {
                        "Powered By Matrix"
                    }
                }


            }


        }
    }
}
