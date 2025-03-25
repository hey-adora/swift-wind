mod components;
mod pages;

use std::sync::Arc;

use freya::prelude::*;
use matrix_sdk::Client;

use pages::login_page;

#[derive(Clone, Default)]
pub enum MainClient {
    #[default]
    Disconnected,
    Connected(Client),
}

fn main() {
    launch(app);
}

fn app() -> Element {
    use_init_theme((|| DARK_THEME)());
    use_context_provider(|| Signal::new(MainClient::default()));

    //TODO: Create broad application state, should be able to switch between login/auth mode and chat mode
    rsx!(
        rect{
            min_width: "fill",
            min_height: "fill",
            content: "flex",
            main_align: "center",

            login_page{}
        }
    )
}
