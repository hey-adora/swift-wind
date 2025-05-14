mod components;
mod hook;
mod page;

use crate::page::{
    connect::Connect, login::Login, main_interface::MainInterface, register::Register,
    settings::Settings,
};
use dioxus_router::prelude::{Routable, Router};
use freya::prelude::*;
use matrix_sdk::Client;
use tracing::info;

#[derive(Debug, Routable, Clone, PartialEq)]
#[rustfmt::skip]
#[allow(clippy::empty_line_after_outer_attr)]
pub enum Route {
    #[layout(Connect)] 

        #[route("/")]
        Login,

        #[route("/register")]
        Register,

    #[end_layout]

    #[route("/main_interface")]
    MainInterface,

    #[route("/settings")]
    Settings,
}

pub static CLIENT: GlobalSignal<MatrixClientState> = Global::new(MatrixClientState::default);

//These two are mainly used for the navigation and router
pub static CURRENT_SPACE: GlobalSignal<Option<String>> = Global::new(Option::default);
pub static CURRENT_ROOM: GlobalSignal<Option<String>> = Global::new(Option::default);

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

    launch_with_props(app, "Swift Wind", (1280.0, 720.0));
}

fn app() -> Element {
    rsx!(
        rect{
            Router::<Route>{}
        }
    )
}
