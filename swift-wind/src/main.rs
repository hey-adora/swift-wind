mod components;
mod hook;
mod page;

use std::sync::Arc;

// use dioxus::prelude::*;
use crate::page::{connect::Connect, login::Login, register::Register};
use dioxus_router::prelude::{Outlet, Routable, Router, RouterConfig};
use freya::prelude::*;
use matrix_sdk::Client;
use tracing::{error, info, trace};

#[derive(Debug, Routable, Clone, PartialEq)]
#[rustfmt::skip]
#[allow(clippy::empty_line_after_outer_attr)]
pub enum Route {
    #[route("/")]
    Connect,

    #[route("/login")]
    Login,

    #[route("/register")]
    Register,
}

pub static CLIENT: GlobalSignal<MatrixClientState> = Global::new(MatrixClientState::default);

#[derive(Debug, Default, Clone)]
pub enum MatrixClientState {
    #[default]
    Disconnected,
    Connecting,
    Connected(Client),
    Error(String),
}

fn main() {
    tracing_subscriber::fmt()
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true),
        )
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .unwrap();

    info!("started!");

    launch(app);
}

fn app() -> Element {
    // use_init_theme((|| DARK_THEME)());
    // use_context_provider(|| Signal::new(MainClient::default()));

    //TODO: Create broad application state, should be able to switch between login/auth mode and chat mode
    rsx!(
        rect{
            // min_width: "fill",
            // min_height: "fill",
            // content: "flex",
            // main_align: "center",

            Router::<Route>{}
        }
    )
}
