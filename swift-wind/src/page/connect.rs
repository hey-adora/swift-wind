use crate::{
    CLIENT, MatrixClientState, Route, components::ICON, hook::connect::use_matrix_connect,
};
use dioxus_router::prelude::{Outlet, navigator};
use freya::prelude::*;

// "http://127.0.0.1:8448"

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct ConnectFormError {
    url: String,
}

#[component]
pub fn Connect() -> Element {
    rsx! {
        rect {
            background: "linear-gradient(325deg, #fff97d 20%, #ffd978 0%, #ffd978 40%, #ff74d1 0%, #ff74d1 60%, #6ddbff 0%, #6ddbff 80%, #b276ff 0%, #b276ff 100%)",
            height: "100%",
            width: "100%",


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


                Outlet::<Route>{}


                rect {
                    label {
                        "Powered By Matrix"
                    }
                }


            }


        }
    }
}
