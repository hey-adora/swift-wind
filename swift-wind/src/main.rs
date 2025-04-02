mod components;
mod hook;
mod page;

// use dioxus::prelude::*;
use crate::page::{
    connect::Connect, login::Login, main_interface::MainInterface, register::Register,
    settings::Settings,
};
use dioxus_router::prelude::{Routable, Router};
use freya::prelude::*;
use matrix_sdk::Client;
use ruma::RoomId;
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

    // #[route("/login")]
    // Login,

    

    // Maybe have a parameter for which space to display?  
    // like /main_interface/SPACEID/ROOMID
    // Probably want to "collapse" all sub-spaces and rooms within a space into one flat structure to make this work
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
