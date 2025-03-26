use freya::dioxus_core;
use freya::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let mut form = use_signal(LoginForm::default);

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
