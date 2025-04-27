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

pub mod driver {
    use futures::future::BoxFuture;
    use matrix_sdk::reqwest::Url;
    use thiserror::Error;

    pub struct NewAccount {
        pub username: String,
        pub password: String,
    }

    pub trait Driver {
        fn connect(&mut self, server: Url) -> BoxFuture<Result<(), ConnectError>>;
        fn login_with_username_password(
            &mut self,
            username: String,
            password: String,
        ) -> BoxFuture<Result<(), LoginWithUsernameAndPasswordError>>;
        fn logout(&mut self) -> BoxFuture<Result<(), LogoutError>>;
    }

    #[derive(Error, Debug)]
    pub enum ConnectError {
        #[error("failed to connect")]
        ConnectionFailed,

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }

    #[derive(Error, Debug)]
    pub enum LoginWithUsernameAndPasswordError {
        #[error("invalid credentials")]
        InvalidCredentials,

        #[error("failed to connect")]
        ConnectionFailed,

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }

    #[derive(Error, Debug)]
    pub enum LogoutError {
        #[error("failed to connect")]
        ConnectionFailed,

        #[error(transparent)]
        Other(#[from] anyhow::Error),
    }
}

pub mod matrix_driver {
    use futures::{FutureExt, future::BoxFuture};
    use matrix_sdk::{Client, reqwest::Url};

    use crate::driver::{ConnectError, Driver, LoginWithUsernameAndPasswordError, LogoutError};

    #[derive(Debug, Clone, Default)]
    pub struct MatrixDriver {
        client: Option<Client>,
        username: String,
    }

    impl Driver for MatrixDriver {
        fn connect(&mut self, server: Url) -> BoxFuture<Result<(), ConnectError>> {
            async move {
                let client = Client::builder()
                    .homeserver_url(server)
                    .handle_refresh_tokens()
                    .build()
                    .await;

                let client = match client {
                    Ok(client) => client,
                    Err(err) => {
                        return Err(ConnectError::Other(err.into()));
                    }
                };

                self.client = Some(client);

                Ok(())
            }
            .boxed()
        }

        fn login_with_username_password(
            &mut self,
            username: String,
            password: String,
        ) -> BoxFuture<Result<(), LoginWithUsernameAndPasswordError>> {
            async move {
                let client = self.client.as_ref().unwrap();
                let res = client
                    .matrix_auth()
                    .login_username(username.clone(), &password)
                    .request_refresh_token()
                    .await;

                let res = match res {
                    Ok(res) => res,
                    Err(err) => {
                        return Err(LoginWithUsernameAndPasswordError::Other(err.into()));
                    }
                };

                self.username = username;

                Ok(())
            }
            .boxed()
        }

        fn logout(&mut self) -> BoxFuture<Result<(), LogoutError>> {
            async move {
                let client = self.client.as_ref().unwrap();
                // client.lo
                Ok(())
            }
            .boxed()
        }
    }
}

pub mod matrix_engine {
    use std::collections::HashMap;

    use matrix_sdk::{Client, reqwest::Url};
    use thiserror::Error;
    use tokio::{
        select,
        sync::{mpsc, oneshot},
    };
    use tokio_util::{sync::CancellationToken, task::TaskTracker};
    use tracing::{debug, error, trace, trace_span, warn};

    pub struct ReqAccount {
        pub server: String,
        pub username: String,
        pub password: String,
    }

    pub struct MatrixUsernamePasswordLogin {
        pub username: String,
        pub password: String,
    }

    #[derive(Clone, Debug)]
    pub struct EngineTx {
        //pub add_account_tx: mpsc::UnboundedSender<(oneshot::Sender<bool>, ReqAccount)>,
        pub connect_tx: mpsc::UnboundedSender<(
            oneshot::Sender<Result<login_username::Res, login_username::HandleErr>>,
            login_username::Req,
        )>,
        pub shutdown: CancellationToken,
        //pub login_tx: mpsc::UnboundedSender<bool>,
    }

    #[derive(Debug)]
    pub struct EngineRx {
        //pub add_account_rx: mpsc::UnboundedReceiver<(oneshot::Sender<bool>, ReqAccount)>,
        pub connect_rx: mpsc::UnboundedReceiver<(
            oneshot::Sender<Result<login_username::Res, login_username::HandleErr>>,
            login_username::Req,
        )>,
        pub shutdown: CancellationToken,
        //pub login_rx: mpsc::UnboundedReceiver<bool>,
    }

    // #[derive(Clone, Debug)]
    // pub struct DriverTx {
    //     pub connect_tx: mpsc::UnboundedSender<(oneshot::Sender<bool>, MatrixConnect)>,
    //     pub login_tx: mpsc::UnboundedSender<(oneshot::Sender<bool>, MatrixUsernamePasswordLogin)>,
    // }

    // #[derive(Debug)]
    // pub struct DriverRx {
    //     pub connect_rx: mpsc::UnboundedReceiver<(oneshot::Sender<bool>, MatrixConnect)>,
    //     pub login_rx: mpsc::UnboundedReceiver<(oneshot::Sender<bool>, MatrixUsernamePasswordLogin)>,
    // }

    pub fn create_engine_channel(cancelation_token: CancellationToken) -> (EngineTx, EngineRx) {
        //let (add_account_res_tx, add_account_res_rx) = mpsc::unbounded_channel();
        let (connect_res_tx, connect_res_rx) = mpsc::unbounded_channel();
        //let (login_res_tx, login_res_rx) = mpsc::unbounded_channel();

        let tx = EngineTx {
            //add_account_tx: add_account_res_tx,
            //login_tx: login_res_tx,
            connect_tx: connect_res_tx,
            shutdown: cancelation_token.clone(),
        };
        let rx = EngineRx {
            //add_account_rx: add_account_res_rx,
            //login_rx: login_res_rx,
            connect_rx: connect_res_rx,
            shutdown: cancelation_token,
        };
        (tx, rx)
    }

    // pub fn create_driver_channel() -> (DriverTx, DriverRx) {
    //     let (connect_tx, connect_rx) = mpsc::unbounded_channel();
    //     let (login_tx, login_rx) = mpsc::unbounded_channel();

    //     let tx = DriverTx {
    //         connect_tx,
    //         login_tx,
    //     };

    //     let rx = DriverRx {
    //         connect_rx,
    //         login_rx,
    //     };

    //     (tx, rx)
    // }

    pub async fn run(engine_tx: EngineTx, mut engine_rx: EngineRx) {
        let tracker = TaskTracker::new();
        // let (add_account_res_tx, add_account_res_rx) = mpsc::unbounded_channel();
        // let (connect_res_tx, connect_res_rx) = mpsc::unbounded_channel();
        // let (login_res_tx, login_res_rx) = mpsc::unbounded_channel();
        //let (kernel_tx, mut kernel_rx) = create_kernel_channel();
        // let client = Client::builder()
        //     .homeserver_url(url)
        //     .handle_refresh_tokens()
        //     .build()
        //     .await
        //     .unwrap();
        let mut client: Option<Client> = None;

        //let mut accounts = HashMap::<String, Box<dyn Driver>>::new();
        //let (driver_tx, driver_rx) = create_driver_channel();
        //let driver_taks = run_driver(kernel_tx, driver_rx, MatrixDriver::default());

        // let mut add_account_fn = async move |(res, url, username, password): (
        //     oneshot::Sender<bool>,
        //     String,
        //     String,
        //     String,
        // )| {
        //     accounts.insert("k".to_string(), "k5".to_string());
        //     tracing::debug!("adding account!");
        // };

        // let connect_fn = async move |res| {
        //     accounts.insert("k".to_string(), "k5".to_string());
        //     tracing::debug!("connected!");
        // };

        // let login_fn = async |res| {
        //     tracing::debug!("logged in!");
        // };

        loop {
            select! {
                // req = kernel_rx.add_account_rx.recv() => {
                //     let Some((res, req)) = req else {
                //         return;
                //     };
                //     add_account(tracker);
                // }
                req = engine_rx.connect_rx.recv() => {
                    let Some((res, req)) = req else {
                        return;
                    };
                    tracker.spawn(login_username::handle(client.clone(), req, res));
                    //let err = res.send(result);
                    // if res.send(result).is_err() {
                    //     error!("failed to send connection result, channel closed");
                    // }
                    //tracker.spawn(connect_fn(connected_res));
                },
                _ = tracker.wait() => {
                    break;
                }
                // login_res = kernel_rx.login_res_rx.recv() => {
                //     tracker.spawn(login_fn(login_res));
                // }
            }
        }

        //tracker.close();
        //tracker.wait().await;
        //let (login_res_tx, login_res_rx) = mpsc::unbounded_channel::<Response>();
        //login_res_rx.recv()
    }

    #[cfg(test)]
    pub mod tests {
        use matrix_sdk::Client;
        use rand::distr::{Alphanumeric, SampleString};
        use test_log::test;
        use tokio_util::sync::CancellationToken;

        use crate::matrix_engine::{self, create_engine_channel};

        #[test(tokio::test)]
        async fn connect_succ() {
            let cancelation_token = CancellationToken::new();
            let (engine_tx, engine_rx) = create_engine_channel(cancelation_token);
            matrix_engine::run(engine_tx, engine_rx).await;
            // let mut client: Option<Client> = None;

            // let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
            // let req = super::Req {
            //     server_url: String::from("http://localhost:8008"),
            // };

            // handle(&mut client, req).await.unwrap();
        }
    }

    pub mod connect {
        use matrix_sdk::{Client, ClientBuildError, config::SyncSettings};
        use ruma::api::{MatrixVersion, client::session::get_login_types::v3::LoginType};
        use thiserror::Error;
        use tracing::{error, trace, trace_span};
        use url::{ParseError, Url};

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct Req {
            pub server_url: String,
        }

        #[derive(Clone, Debug)]
        pub struct Res {
            pub server_version: MatrixVersion,
            pub login_flow: Vec<LoginType>,
        }

        pub async fn handle(client: &mut Option<Client>, req: Req) -> Result<Res, HandleErr> {
            let new_client = Client::builder()
                .homeserver_url(req.server_url)
                .handle_refresh_tokens()
                .build()
                .await
                .map_err(|err| match err {
                    ClientBuildError::Url(err) => HandleErr::InvalidUrl(err),
                    err => HandleErr::from(err),
                })?;

            let version_res = new_client.server_versions().await?;
            let login_res = new_client.matrix_auth().get_login_types().await?;

            *client = Some(new_client);

            let res = Res {
                server_version: *version_res.first().unwrap(),
                login_flow: login_res.flows,
            };

            trace!("{res:#?}");

            Ok(res)
        }

        #[derive(Error, Debug)]
        pub enum HandleErr {
            #[error("failed to connect {0}")]
            ConnectionFailed(#[from] matrix_sdk::HttpError),

            #[error("invalid url {0}")]
            InvalidUrl(#[from] ParseError),

            #[error("client build error")]
            ClientBuildError(#[from] ClientBuildError),
        }

        #[cfg(test)]
        mod tests {
            use matrix_sdk::Client;
            use rand::distr::{Alphanumeric, SampleString};
            use test_log::test;

            use super::handle;

            #[test(tokio::test)]
            async fn connect_succ() {
                let mut client: Option<Client> = None;

                let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
                let req = super::Req {
                    server_url: String::from("http://localhost:8008"),
                };

                handle(&mut client, req).await.unwrap();
            }
        }
    }

    pub mod discovery {
        use matrix_sdk::{Client, ClientBuildError, config::SyncSettings};
        use ruma::api::{MatrixVersion, client::session::get_login_types::v3::LoginType};
        use thiserror::Error;
        use tracing::{error, trace, trace_span};
        use url::{ParseError, Url};

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct Req {
            pub server_url: String,
        }

        #[derive(Clone, Debug)]
        pub struct Res {
            pub server_version: MatrixVersion,
            pub login_flow: Vec<LoginType>,
        }

        pub async fn handle(req: Req) -> Result<Res, HandleErr> {
            let new_client = Client::builder()
                .homeserver_url(&req.server_url)
                .build()
                .await
                .map_err(|err| match err {
                    ClientBuildError::Url(err) => HandleErr::InvalidUrl(err),
                    err => HandleErr::from(err),
                })?;

            let version_res = new_client.server_versions().await?;
            let login_res = new_client.matrix_auth().get_login_types().await?;

            let res = Res {
                server_version: *version_res.first().unwrap(),
                login_flow: login_res.flows,
            };

            trace!("{res:#?}");

            Ok(res)
        }

        #[derive(Error, Debug)]
        pub enum HandleErr {
            #[error("failed to connect {0}")]
            ConnectionFailed(#[from] matrix_sdk::HttpError),

            #[error("invalid url {0}")]
            InvalidUrl(#[from] ParseError),

            #[error("client build error")]
            ClientBuildError(#[from] ClientBuildError),
        }

        #[cfg(test)]
        mod tests {
            use matrix_sdk::Client;
            use rand::distr::{Alphanumeric, SampleString};
            use test_log::test;

            use super::handle;

            #[test(tokio::test)]
            async fn discover_succ() {
                let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
                let req = super::Req {
                    server_url: String::from("http://localhost:8008"),
                };

                handle(req).await.unwrap();
            }
        }
    }

    pub mod login_username {
        use matrix_sdk::{Client, ClientBuildError, config::SyncSettings};
        use thiserror::Error;
        use tokio::sync::oneshot;
        use url::{ParseError, Url};

        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub struct Req {
            pub server_url: String,
            pub username: String,
            pub password: String,
        }

        pub type Res = ();

        pub async fn handle(
            client: Option<Client>,
            req: Req,
            res: oneshot::Sender<Result<Res, HandleErr>>,
        ) {
            let client = client.ok_or(HandleErr::ClientIsNotConnected)?;
            // let new_client = Client::builder()
            //     .homeserver_url(&req.server_url)
            //     .handle_refresh_tokens()
            //     .build()
            //     .await
            //     .map_err(|err| match err {
            //         ClientBuildError::Url(err) => HandleErr::InvalidUrl(err),
            //         err => HandleErr::from(err),
            //     })?;

            let login_res = client
                .matrix_auth()
                .login_username(req.username, &req.password)
                .initial_device_display_name("login test")
                .await?;

            let sync_settings = SyncSettings::new();
            let result = client.sync_once(sync_settings).await;
            let sync_response = match result {
                Ok(r) => r,
                Err(matrix_sdk::Error::Url(parse_err)) => {
                    return Err(HandleErr::InvalidUrl(parse_err));
                }
                Err(err) => {
                    return Err(err.into());
                }
            };
            //*client = Some(new_client);

            Ok(())
        }

        #[derive(Error, Debug)]
        pub enum HandleErr {
            #[error("client is not connected to any matrix server, login cannot be performed")]
            ClientIsNotConnected,

            #[error("failed to connect")]
            ConnectionFailed(#[from] matrix_sdk::Error),

            #[error("invalid url {0}")]
            InvalidUrl(#[from] ParseError),

            #[error("client build error")]
            ClientBuildError(#[from] ClientBuildError),
            // #[error(transparent)]
            // Other(#[from] anyhow::Error),
        }

        #[cfg(test)]
        mod tests {
            use matrix_sdk::Client;
            use rand::distr::{Alphanumeric, SampleString};
            use test_log::test;

            use super::handle;

            #[test(tokio::test)]
            async fn login_succ() {
                let mut client: Option<Client> = None;

                let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
                let req = super::Req {
                    server_url: String::from("http://localhost:8008"),
                    username: rand_str.clone(),
                    password: rand_str.clone(),
                };

                //handle(&mut client, req).await.unwrap();
            }

            #[test(tokio::test)]
            #[should_panic(expected = "wrong server url")]
            async fn login_fail() {
                let mut client: Option<Client> = None;

                let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
                let req = super::Req {
                    server_url: String::from("localhost:6969"),
                    username: rand_str.clone(),
                    password: rand_str.clone(),
                };

                //handle(&mut client, req).await.unwrap();
            }
        }
    }
}

#[cfg(test)]
mod kernel_tests {
    use test_log::test;
    use tracing::{debug, error, level_filters::LevelFilter};

    #[test(tokio::test)]
    async fn login() {}
}

pub mod tailwind {
    pub struct Parser {
        pub on_bg_fn: Box<dyn FnMut(&str)>,
    }

    impl Default for Parser {
        fn default() -> Self {
            Self {
                on_bg_fn: Box::new(move |input: &str| {}),
            }
        }
    }

    impl Parser {
        pub fn on_bg<F: FnMut(&str) + 'static>(mut self, f: F) -> Self {
            self.on_bg_fn = Box::new(f);
            self
        }
    }

    const SYNTAX_BG: &str = "bg";

    pub enum Token {
        Bg,
        Invalid,
    }

    impl From<&str> for Token {
        fn from(value: &str) -> Self {
            match value {
                "bg" => Token::Bg,
                _ => Token::Invalid,
            }
        }
    }

    pub struct Cords {
        pub start: usize,
        pub end: usize,
    }

    pub fn parse<S: AsRef<str>>(input: S) -> Vec<(Token, Cords)> {
        let input = input.as_ref();

        // let chars = input.chars();
        // let len = chars.cou();

        let output: Vec<(Token, Cords)> = input
            .split_ascii_whitespace()
            .map(|a| (Token::from(a), Cords { start: 0, end: 0 }))
            .collect();

        // let mut i: usize = 0;
        // loop {
        //     let c = { input.chars() };
        //     match char {
        //         'b' => {}
        //         _ => {}
        //     }
        // }

        output
    }

    #[cfg(test)]
    pub mod tailwind_tests {
        use test_log::test;
        use tracing::{debug, error, level_filters::LevelFilter, trace};

        use crate::tailwind::Parser;

        #[test(tokio::test)]
        async fn parse() -> anyhow::Result<()> {
            let parser = Parser::default().on_bg(|color: &str| {
                trace!("oh look, a color: {color}");
            });

            let input = "bg-red";
            // parser.parse(input);
            // parser.run();

            Ok(())
        }
    }
}

#[cfg(test)]
pub mod ui_tests {
    use freya::core::events::PlatformEvent;
    use freya::prelude::*;
    use freya_testing::prelude::*;
    use freya_testing::{config::TestingConfig, event::TestEvent, launch::launch_test_with_config};
    use test_log::test;
    use tracing::{debug, error, level_filters::LevelFilter};

    #[component]
    pub fn Div(#[props(default)] style: ReadOnlySignal<String>) -> Element {
        rsx!(rect {})
    }

    #[test(tokio::test)]
    async fn wasmer() -> anyhow::Result<()> {
        fn event_component() -> Element {
            let mut enabled = use_signal(|| false);

            //Button(props);
            rsx!(
                rect {
                    width: "100%",
                    height: "100%",
                    background: "red",
                    onclick: move |_| {
                        enabled.set(true);
                    },
                    label {
                        "Is enabled? {enabled}"
                    }
                }
            )
        }

        let config = TestingConfig::<()>::new();
        let mut utils = launch_test_with_config(event_component, config);

        utils.save_snapshot("./how_the_f_0.png");
        let rect = utils.root().get(0);
        let label = rect.get(0);

        utils.resize((500., 250.).into());

        utils.wait_for_update().await;

        utils.save_snapshot("./how_the_f_1.png");

        let text = label.get(0);
        assert_eq!(text.text(), Some("Is enabled? false"));

        // Push a click event to the events queue
        utils.push_event(TestEvent::Mouse {
            name: EventName::Click,
            cursor: (5.0, 5.0).into(),
            button: Some(MouseButton::Left),
        });

        // Poll the VirtualDOM with the new events
        utils.wait_for_update().await;

        // Because the click event was sent, and the state updated, the text was changed as well!
        let text = label.get(0);
        assert_eq!(text.text(), Some("Is enabled? true"));

        Ok(())
    }
}

// #[cfg(test)]
// mod plugin_test {
//     use test_log::test;
//     use tracing::{debug, error, level_filters::LevelFilter};
//     use wasmer::{Function, FunctionType, Instance, Module, Store, Type, Value, imports};

//     #[test(tokio::test)]
//     async fn wasmer() -> anyhow::Result<()> {
//         let module_wat = r#"
//         (module
//         (type $t0 (func (param i32) (result i32)))
//         (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
//             local.get $p0
//             i32.const 1
//             i32.add))
//         "#;

//         let mut store = Store::default();
//         //let module = Module::new(&store, &module_wat)?;
//         let module = Module::from_file(
//             &store,
//             "/home/hey/github/swift-wind/target/wasm32-unknown-unknown/debug/swift_wind_plugin_matrix.wasm",
//         )?;
//         // The module doesn't import anything, so we create an empty import object.
//         let print_type = FunctionType::new(vec![Type::ExternRef], vec![]);
//         let print = Function::new(&mut store, &print_type, |args| {
//             let ptr = args[0].unwrap_externref().unwrap().downcast(store);

//             Ok(vec![])
//         });
//         let import_object = imports! {
//             "print" =>
//         };
//         let instance = Instance::new(&mut store, &module, &import_object)?;

//         let add_one = instance.exports.get_function("add_one")?;
//         let result = add_one.call(&mut store, &[Value::I32(42)])?;
//         assert_eq!(result[0], Value::I32(43));

//         Ok(())
//     }
// }
