use dioxus_router::{hooks::use_route, prelude::navigator};
use freya::prelude::*;
use matrix_sdk::{Client, reqwest::Url};
use tracing::trace;

use crate::{
    Route,
    hook::connect::{CLIENT, MatrixClientState, use_matrix_connect},
};

// "http://127.0.0.1:8448"

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ConnectForm {
    url: String,
}

impl Default for ConnectForm {
    fn default() -> Self {
        Self {
            url: String::from("http://127.0.0.1:8008"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct ConnectFormError {
    url: String,
}

// #[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
// pub enum ConnectionState {
//     #[default]
//     Disconnected,
//     Connected,
//     Error(String),
// }

#[component]
pub fn Connect() -> Element {
    let mut form = use_signal(|| ConnectForm::default());
    let router: Route = use_route();
    let navigator = navigator();

    // let mut form_errors = use_signal(|| ConnectFormError::default());
    // let mut connection_state = use_signal(|| ConnectionState::default());
    let (mut get_matrix_connect, mut run_matrix_connect) = use_matrix_connect(move || {
        navigator.push("/login");
    });

    let on_connect = move |_| {
        let url = form().url;
        trace!("connecting to: {url}");
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

                    "Home Server"
                }

                rect {
                    Input {
                        width: "80%",
                        value: form().url,
                        placeholder: "http://127.0.0.1:8008",
                        onchange: move |txt| {
                            form.write().url = txt;
                            get_matrix_connect.write().clear();
                        }
                    }
                    label {
                        "{errors}"
                     }
                }

                Button {
                    onclick: on_connect,
                    label {
                        "Connect"
                    }
                }
            }
        }
    }
}
