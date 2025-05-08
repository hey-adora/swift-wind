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

pub mod bevy_matrix_app {
    use std::thread::{self, JoinHandle};

    use bevy::app::App;

    use crate::bevy_matrix::{
        bevy_tokio::TokioPlugin,
        exit_plugin::ExitPlugin,
        reactive_runner_plugin::{ReactiveRunner, ReactiveRunnerPlugin},
    };

    pub fn run_matrix_app() -> (JoinHandle<()>, ReactiveRunner) {
        let (reactive_runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
        let handle = thread::spawn(move || {
            App::new()
                // .add_plugins(exit_channel_plugin)
                .add_plugins(reactive_runner_plugin)
                // .add_plugins(ExitPlugin::new())
                .add_plugins(TokioPlugin::new())
                // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1)))
                // .add_systems(Update, test_system.before(exit_plugin::shutdown))
                // .set_runner(runner::runner)
                .run();
        });
        (handle, reactive_runner)
    }
}

pub mod bevy_matrix {

    pub mod runner {
        use std::sync::mpsc;

        use bevy::app::{App, AppExit};
        use tracing::trace;

        pub fn runner(mut app: App) -> AppExit {
            app.finish();
            app.cleanup();

            let (tx, rx) = mpsc::channel::<()>();
            loop {
                trace!("In main loop");

                app.update();
                if let Some(exit) = app.should_exit() {
                    return exit;
                }
                //break AppExit::Success;
            }
        }
    }

    // pub mod async_bevy_channel {
    //     use bevy::prelude::*;
    //     use crossbeam_channel;
    //     use std::{
    //         fmt::Debug,
    //         ops::{Deref, DerefMut},
    //         sync::mpsc,
    //     };

    //     #[derive(Resource, Debug, Clone)]
    //     pub struct AsyncBevyChannel<T> {
    //         pub rx: crossbeam_channel::Receiver<T>,
    //     }

    //     impl<T> Deref for AsyncBevyChannel<T> {
    //         type Target = crossbeam_channel::Receiver<T>;
    //         fn deref(&self) -> &Self::Target {
    //             &self.rx
    //         }
    //     }

    //     impl<T> DerefMut for AsyncBevyChannel<T> {
    //         fn deref_mut(&mut self) -> &mut Self::Target {
    //             &mut self.rx
    //         }
    //     }

    //     impl<T> AsyncBevyChannel<T> {
    //         pub fn new() -> (Self, crossbeam_channel::Sender<T>) {
    //             let (tx, rx) = crossbeam_channel::unbounded();
    //             (Self { rx }, tx)
    //         }
    //     }

    //     impl<T: Send + Sync + Clone + Debug + Component + 'static> Plugin for AsyncBevyChannel<T> {
    //         fn build(&self, app: &mut App) {
    //             // let (tx, rx) = mpsc::channel::<()>();
    //             let res = self.clone();
    //             app.insert_resource(res);
    //             app.add_systems(Update, handle_req::<T>);
    //         }
    //     }

    //     pub fn handle_req<T: Send + Sync + Debug + Component + 'static>(
    //         mut commands: Commands,
    //         listener: Res<AsyncBevyChannel<T>>,
    //     ) {
    //         trace!("hello from bevy async channel");
    //         // commands.spawn_batch(listener.try_iter());
    //         for req in listener.try_iter() {
    //             trace!("spawning {req:#?}");
    //             commands.spawn(req);
    //         }
    //     }
    // }

    // pub mod exit_plugin {

    //     use bevy::prelude::*;

    //     #[derive(Default, Debug, Clone)]
    //     pub struct ExitPlugin {}

    //     #[derive(Component, Debug, Clone, Copy)]
    //     pub enum ExitToken {
    //         SigTerm,
    //     }

    //     impl ExitPlugin {
    //         pub fn new() -> ExitPlugin {
    //             Self::default()
    //         }
    //     }

    //     impl Plugin for ExitPlugin {
    //         fn build(&self, app: &mut App) {
    //             app.add_systems(Last, shutdown);
    //         }
    //     }

    //     pub fn req_exit() -> impl FnMut(&mut bevy::app::App) {
    //         |app: &mut bevy::app::App| {
    //             // code
    //             trace!("hello from callback");
    //             let world = app.world_mut();
    //             world.spawn(ExitToken::SigTerm);
    //         }
    //     }

    //     pub fn shutdown(tokens: Query<&ExitToken>, mut event_writer: EventWriter<AppExit>) {
    //         trace!("hello from exit");
    //         for token in tokens.iter() {
    //             trace!("queried {token:#?}");
    //             match token {
    //                 ExitToken::SigTerm => {
    //                     event_writer.write(AppExit::Success);
    //                 }
    //             };
    //         }
    //     }
    // }

    pub mod reactive_runner_plugin {
        use bevy::app::{App, AppExit};
        use bevy::ecs::component::ComponentMutability;
        use bevy::ecs::system::SystemParam;
        use bevy::prelude::*;
        use bevy::tasks::block_on;
        use std::cell::RefCell;
        use std::marker::PhantomData;
        use std::ops::Deref;
        use std::sync::mpsc;
        use tokio::sync::{broadcast, watch};
        use tokio::task::spawn_blocking;
        use tracing::trace;

        // pub trait ReqWrap {}

        // #[derive(Component)]
        // pub trait ReqData<T: Component> {}

        pub enum Req {
            Data(Box<dyn FnOnce(&mut App) + Sync + Send>),
            Tick,
        }

        #[derive(Debug, Clone)]
        pub struct ReactiveRunnerPlugin {
            tx: async_channel::Sender<Req>,
            rx: async_channel::Receiver<Req>,
        }

        impl ReactiveRunnerPlugin {
            pub fn new() -> (Self, ReactiveRunner) {
                let (tx, rx) = async_channel::unbounded::<Req>();
                let plugin = ReactiveRunnerPlugin { tx: tx.clone(), rx };
                let reactive_runner = ReactiveRunner { tx };
                (plugin, reactive_runner)
            }
        }

        // pub struct Req {
        //     pub data
        // }

        // #[derive(SystemParam, Debug, Clone)]
        // pub struct Tick<'w, Marker: 'static> {
        //     tx: Res<'w, TickTx>,
        //     phantom: PhantomData<Marker>,
        // }

        #[derive(Resource, Clone)]
        pub struct ReactiveRunner {
            tx: async_channel::Sender<Req>,
        }

        impl ReactiveRunner {
            pub async fn send_tick(&self) {
                self.tx.send(Req::Tick).await.unwrap();
            }
            pub fn send_tick_blocking(&self) {
                self.tx.send_blocking(Req::Tick).unwrap();
            }
            pub async fn send_fn(&self, f: impl FnOnce(&mut App) + Send + Sync + 'static) {
                self.tx.send(Req::Data(Box::new(f))).await.unwrap();
            }
            pub fn send_fn_blocking(&self, f: impl FnOnce(&mut App) + Send + Sync + 'static) {
                self.tx.send_blocking(Req::Data(Box::new(f))).unwrap();
            }
        }

        // impl Deref for TickTx {
        //     type Target = watch::Sender<()>;

        //     fn deref(&self) -> &Self::Target {
        //         &self.tx
        //     }
        // }

        impl Plugin for ReactiveRunnerPlugin {
            fn build(&self, app: &mut App) {
                let rx = self.rx.clone();
                app.insert_resource(ReactiveRunner {
                    tx: self.tx.clone(),
                });
                app.set_runner(move |mut app| -> AppExit {
                    app.finish();
                    app.cleanup();

                    // let (tx, rx) = mpsc::channel::<()>();
                    loop {
                        //let req = rx.;
                        //req;

                        trace!("tick");

                        app.update();
                        if let Some(exit) = app.should_exit() {
                            return exit;
                        }
                        trace!("waiting for req");
                        let req = match rx.recv_blocking() {
                            Ok(req) => req,
                            Err(err) => {
                                error!("err: {}", err);
                                return AppExit::Error(1.try_into().unwrap());
                            }
                        };
                        trace!("req received");
                        match req {
                            Req::Data(mut data) => {
                                (data)(&mut app);
                            }
                            Req::Tick => {
                                continue;
                            }
                        }
                        //break AppExit::Success;
                    }
                });
            }
        }

        // pub fn runner(mut app: App) -> AppExit {
        //     app.finish();
        //     app.cleanup();

        //     let (tx, rx) = mpsc::channel::<()>();
        //     loop {
        //         match rx.recv() {
        //             Ok(_) => {}
        //             Err(err) => {
        //                 error!("err: {}", err);
        //                 return AppExit::Error(1.try_into().unwrap());
        //             }
        //         };

        //         trace!("tick");

        //         app.update();
        //         if let Some(exit) = app.should_exit() {
        //             return exit;
        //         }
        //         //break AppExit::Success;
        //     }
        // }
    }

    pub mod bevy_tokio {
        use std::{marker::PhantomData, ops::Deref, sync::Arc};

        use bevy::{
            app::{App, Plugin},
            ecs::{
                component::Component,
                resource::Resource,
                system::{Res, SystemParam},
            },
        };
        use tokio::runtime::Runtime;

        use super::reactive_runner_plugin::ReactiveRunner;

        #[derive(Debug, Clone, Default)]
        pub struct TokioPlugin {
            rt: TokioRt,
        }

        impl TokioPlugin {
            pub fn new() -> Self {
                Self::default()
            }
        }

        impl Plugin for TokioPlugin {
            fn build(&self, app: &mut bevy::app::App) {
                app.insert_resource(self.rt.clone());
            }
        }

        #[derive(Resource, Debug, Clone)]
        pub struct TokioRt {
            rt: Arc<tokio::runtime::Runtime>,
        }

        impl Deref for TokioRt {
            type Target = tokio::runtime::Runtime;

            fn deref(&self) -> &Self::Target {
                &self.rt
            }
        }

        impl Default for TokioRt {
            fn default() -> Self {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                let rt = Arc::new(rt);
                Self { rt }
            }
        }

        #[derive(SystemParam)]
        pub struct AsyncQueue<'w> {
            reactive_runner: Res<'w, ReactiveRunner>,
            tokio_rt: Res<'w, TokioRt>,
            // phantom: PhantomData<Marker>,
        }

        impl AsyncQueue<'_> {
            pub fn spawn<T: Send + Sync + 'static>(
                &self,
                mut f: impl Future<Output = T> + Send + Sync + 'static,
                mut callback: impl FnOnce(&mut App, T) + Send + Sync + 'static,
            )
            // pub fn spawn<Function, FunctionFuture, FunctionReturn>(&self, mut f: Function)
            // where
            //     Function: FnMut() -> FunctionFuture + Send + Sync + 'static,
            //     // FunctionReturn: FnMut(),
            //     // FunctionReturn: Component + Send + Sync + 'static,
            //     FunctionReturn: FnMut(&mut App) + Send + Sync + 'static,
            //     FunctionFuture: Future<Output = FunctionReturn> + Send + Sync + 'static,
            {
                // let rt = Runtime::new().unwrap();
                // rt.s

                // rt.sp;
                let reactive_runtime = self.reactive_runner.clone();
                self.tokio_rt.spawn(async move {
                    let result = f.await;
                    reactive_runtime
                        .send_fn(move |app| {
                            callback(app, result);
                        })
                        .await;
                    //code  :w
                });
                // self.reactive_runner.send_fn(f);
            }
        }
    }

    // pub mod bevy_matrix_request {

    //     use bevy::prelude::*;

    //     pub fn req_exit() -> impl FnMut(&mut bevy::app::App) {
    //         |app: &mut bevy::app::App| {
    //             // code
    //             trace!("hello from callback");
    //             let world = app.world_mut();
    //             world.spawn(ExitToken::SigTerm);
    //         }
    //     }
    // }

    pub mod bevy_matrix_remote {

        use std::ops::Deref;

        use bevy::prelude::*;
        use url::Url;

        #[derive(Component, Debug, Clone)]
        pub struct MatrixEndpoint(pub Url);

        impl Deref for MatrixEndpoint {
            type Target = Url;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub struct MatrixRemotePlugin {}

        impl Plugin for MatrixRemotePlugin {
            fn build(&self, app: &mut App) {

                // app.add_systems(Update, systems);
            }
        }
    }

    pub mod bevy_matrix_reqwest {

        use bevy::prelude::*;

        #[derive(Debug, Clone, Copy, Default)]
        pub struct MatrixRemotePlugin {}

        impl Plugin for MatrixRemotePlugin {
            fn build(&self, app: &mut App) {

                // app.add_systems(Update, systems);
            }
        }
    }

    pub mod bevy_matrix_versions {
        use std::collections::HashMap;

        use bevy::prelude::*;

        pub struct Request(pub String);

        pub struct Response {
            pub versions: Vec<String>,
            pub features: HashMap<String, bool>,
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub struct Matrix1_0VersionsPlugin {}

        impl Plugin for Matrix1_0VersionsPlugin {
            fn build(&self, app: &mut App) {
                // app.add_systems(Update, systems);
            }
        }

        pub fn create_matrix_1_0_request_body(mut commands: Commands) {
            // commands.spawn(bundle);
        }
    }

    // pub mod plugin {
    //     pub mod exit {
    //         pub
    //     }
    // }

    #[cfg(test)]
    mod tests {
        use std::{sync::Arc, thread, time::Duration};

        use bevy::prelude::*;
        // use bevy::{
        //     DefaultPlugins,
        //     app::{App, AppExit, ScheduleRunnerPlugin, Update},
        //     ecs::schedule::IntoScheduleConfigs,
        //     tasks::AsyncComputeTaskPool,
        // };
        // use freya::dioxus_core::SpawnIfAsync;
        use test_log::test;
        use tracing::trace;

        use crate::bevy_matrix::{
            bevy_tokio::TokioPlugin, reactive_runner_plugin::ReactiveRunnerPlugin, runner,
        };

        use super::bevy_tokio::AsyncQueue;

        fn test_system(queue: AsyncQueue) {
            queue.spawn(async move { 0 }, move |app, result| {
                trace!("got {result}");
            });
            trace!("wow");
        }

        #[test]
        fn queen() {
            // // App::empty().set_runner(runner::runner).run();
            // // let (exit_channel_plugin, exit_req) =
            // //     async_bevy_channel::AsyncBevyChannel::<ExitToken>::new();

            // let (reactive_runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();

            // // let rt = tokio::runtime::Builder::new_multi_thread()
            // //     .enable_all()
            // //     .build()
            // //     .unwrap();
            // // let rt = Arc::new(rt);
            // // // thread::scope(|s| {
            // // //     s.spawn();
            // // let _guard = rt.enter();

            // // let pool = AsyncComputeTaskPool::get_or_init();
            // // });
            // // App::new()
            // //     // .add_plugins(exit_channel_plugin)
            // //     // .add_plugins(plugin::exit::Config {})
            // //     .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(1)))
            // //     .add_systems(Update, test_system)
            // //     // .set_runner(runner::runner)
            // //     .run();
            // // DefaultPlugins

            // // let m = ruma::api::client::discovery::get_supported_versions::METADATA;
            // let handle = thread::spawn(move || {
            //     App::new()
            //         // .add_plugins(exit_channel_plugin)
            //         .add_plugins(reactive_runner_plugin)
            //         // .add_plugins(ExitPlugin::new())
            //         .add_plugins(TokioPlugin::new())
            //         // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(1)))
            //         // .add_systems(Update, test_system.before(exit_plugin::shutdown))
            //         // .set_runner(runner::runner)
            //         .run();
            // });
            // // std::thread::sleep(Duration::from_secs(5));
            // //reactive_runner.exit_req.send(ExitToken::SigTerm).unwrap();
            // let exit = || {
            //     |app: &mut bevy::app::App| {
            //         // code
            //         trace!("hello from callback");
            //         let world = app.world_mut();
            //         world.spawn(ExitToken::SigTerm);
            //     }
            // };
            // reactive_runner.send_fn_blocking(exit());
            // handle.join().unwrap();
        }
    }
}

// pub mod driver {
//     use futures::future::BoxFuture;
//     use matrix_sdk::reqwest::Url;
//     use thiserror::Error;

//     pub struct NewAccount {
//         pub username: String,
//         pub password: String,
//     }

//     pub trait Driver {
//         fn connect(&mut self, server: Url) -> BoxFuture<Result<(), ConnectError>>;
//         fn login_with_username_password(
//             &mut self,
//             username: String,
//             password: String,
//         ) -> BoxFuture<Result<(), LoginWithUsernameAndPasswordError>>;
//         fn logout(&mut self) -> BoxFuture<Result<(), LogoutError>>;
//     }

//     #[derive(Error, Debug)]
//     pub enum ConnectError {
//         #[error("failed to connect")]
//         ConnectionFailed,

//         #[error(transparent)]
//         Other(#[from] anyhow::Error),
//     }

//     #[derive(Error, Debug)]
//     pub enum LoginWithUsernameAndPasswordError {
//         #[error("invalid credentials")]
//         InvalidCredentials,

//         #[error("failed to connect")]
//         ConnectionFailed,

//         #[error(transparent)]
//         Other(#[from] anyhow::Error),
//     }

//     #[derive(Error, Debug)]
//     pub enum LogoutError {
//         #[error("failed to connect")]
//         ConnectionFailed,

//         #[error(transparent)]
//         Other(#[from] anyhow::Error),
//     }
// }

// pub mod matrix_driver {
//     use futures::{FutureExt, future::BoxFuture};
//     use matrix_sdk::{Client, reqwest::Url};

//     use crate::driver::{ConnectError, Driver, LoginWithUsernameAndPasswordError, LogoutError};

//     #[derive(Debug, Clone, Default)]
//     pub struct MatrixDriver {
//         client: Option<Client>,
//         username: String,
//     }

//     impl Driver for MatrixDriver {
//         fn connect(&mut self, server: Url) -> BoxFuture<Result<(), ConnectError>> {
//             async move {
//                 let client = Client::builder()
//                     .homeserver_url(server)
//                     .handle_refresh_tokens()
//                     .build()
//                     .await;

//                 let client = match client {
//                     Ok(client) => client,
//                     Err(err) => {
//                         return Err(ConnectError::Other(err.into()));
//                     }
//                 };

//                 self.client = Some(client);

//                 Ok(())
//             }
//             .boxed()
//         }

//         fn login_with_username_password(
//             &mut self,
//             username: String,
//             password: String,
//         ) -> BoxFuture<Result<(), LoginWithUsernameAndPasswordError>> {
//             async move {
//                 let client = self.client.as_ref().unwrap();
//                 let res = client
//                     .matrix_auth()
//                     .login_username(username.clone(), &password)
//                     .request_refresh_token()
//                     .await;

//                 let res = match res {
//                     Ok(res) => res,
//                     Err(err) => {
//                         return Err(LoginWithUsernameAndPasswordError::Other(err.into()));
//                     }
//                 };

//                 self.username = username;

//                 Ok(())
//             }
//             .boxed()
//         }

//         fn logout(&mut self) -> BoxFuture<Result<(), LogoutError>> {
//             async move {
//                 let client = self.client.as_ref().unwrap();
//                 // client.lo
//                 Ok(())
//             }
//             .boxed()
//         }
//     }
// }

// pub mod matrix_engine {
//     use std::collections::HashMap;

//     use matrix_sdk::{Client, reqwest::Url};
//     use thiserror::Error;
//     use tokio::{
//         select, spawn,
//         sync::{mpsc, oneshot},
//         task::JoinHandle,
//     };
//     use tokio_util::{sync::CancellationToken, task::TaskTracker};
//     use tracing::{debug, error, trace, trace_span, warn};

//     pub struct ReqAccount {
//         pub server: String,
//         pub username: String,
//         pub password: String,
//     }

//     pub struct MatrixUsernamePasswordLogin {
//         pub username: String,
//         pub password: String,
//     }

//     #[derive(Clone, Debug)]
//     pub struct EngineExternalTx {
//         pub connect_tx: mpsc::UnboundedSender<(
//             oneshot::Sender<Result<connect::Res, connect::HandleErr>>,
//             connect::Req,
//         )>,
//         pub shutdown_tx: mpsc::UnboundedSender<oneshot::Sender<()>>,
//     }

//     impl EngineExternalTx {
//         pub async fn connect(&self, url: String) -> Result<connect::Res, connect::HandleErr> {
//             let (tx, rx) = oneshot::channel::<Result<connect::Res, connect::HandleErr>>();
//             self.connect_tx
//                 .send((tx, connect::Req { server_url: url }))
//                 .map_err(|_| connect::HandleErr::MatrixEngineIsNotRunning)?;
//             rx.await
//                 .map_err(|_| connect::HandleErr::MatrixEngineIsNotRunning)?
//         }
//         pub async fn shutdown(&self) {
//             let (tx, rx) = oneshot::channel::<()>();
//             let _ = self.shutdown_tx.send(tx);
//             let _ = rx.await;
//         }
//     }

//     #[derive(Debug)]
//     pub struct EngineExternalRx {
//         pub connect_rx: mpsc::UnboundedReceiver<(
//             oneshot::Sender<Result<connect::Res, connect::HandleErr>>,
//             connect::Req,
//         )>,
//         pub shutdown_rx: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
//     }

//     #[derive(Clone, Debug)]
//     pub struct EngineInternalTx {
//         pub connect_tx: mpsc::UnboundedSender<(
//             oneshot::Sender<Result<connect::Res, connect::HandleErr>>,
//             connect::Res,
//         )>,
//         //pub shutdown_tx: mpsc::UnboundedSender<oneshot::Sender<()>>,
//     }

//     #[derive(Debug)]
//     pub struct EngineInternalRx {
//         pub connect_rx: mpsc::UnboundedReceiver<(
//             oneshot::Sender<Result<login_username::Res, login_username::HandleErr>>,
//             login_username::Res,
//         )>,
//         //pub shutdown_rx: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
//     }

//     pub fn create_engine_external_channel() -> (EngineExternalTx, EngineExternalRx) {
//         let (connect_tx, connect_rx) = mpsc::unbounded_channel();
//         let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel();

//         let tx = EngineExternalTx {
//             connect_tx,
//             shutdown_tx,
//         };
//         let rx = EngineExternalRx {
//             connect_rx,
//             shutdown_rx,
//         };
//         (tx, rx)
//     }

//     pub fn create_engine_internal_channel() -> (EngineInternalTx, EngineInternalRx) {
//         let (connect_tx, connect_rx) = mpsc::unbounded_channel();
//         //let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

//         let tx = EngineInternalTx {
//             connect_tx,
//             //    shutdown_tx,
//         };
//         let rx = EngineInternalRx {
//             connect_rx,
//             //    shutdown_rx,
//         };
//         (tx, rx)
//     }

//     //pub async fn run() -> (EngineExternalTx, JoinHandle<()>) {
//     pub async fn run() -> EngineExternalTx {
//         let (engine_external_tx, mut engine_external_rx) = create_engine_external_channel();
//         //let (engine_internal_tx, mut engine_internal_rx) = create_engine_internal_channel();
//         //let tracker = TaskTracker::new();

//         //let mut shutdown_awaiters: Vec<oneshot::Sender<()>> = Vec::new();

//         let event_loop = spawn({
//             async move {
//                 let mut client: Option<Client> = None;
//                 let tracker = TaskTracker::new();
//                 let mut shutdown_awaiters: Vec<oneshot::Sender<()>> = Vec::new();

//                 loop {
//                     select! {
//                         // connect handlers
//                         external_req = engine_external_rx.connect_rx.recv() => {
//                             let (external_res, req) = external_req.unwrap();
//                             if tracker.is_closed() {
//                                 continue;
//                             }
//                             //let client = client.clone();
//                             tracker.spawn(async move {
//                                 let result = connect::handle(&mut client, req).await;
//                                 let _ = external_res.send(result);
//                             });

//                         },

//                         // shutdown handlers
//                         external_req = engine_external_rx.shutdown_rx.recv() => {
//                             let external_res = external_req.unwrap();
//                             shutdown_awaiters.push(external_res);
//                             tracker.close();
//                             //engine_internal_tx.shutdown_tx.send(res).await.unwrap();
//                             //break;
//                         }

//                         _ = tracker.wait() => {
//                             break;
//                         }

//                         // //
//                         // _ = tracker.wait() => {
//                         //     break;
//                         // }

//                     }
//                 }

//                 for awaiter in shutdown_awaiters {
//                     let _ = awaiter.send(());
//                 }
//             }
//         });

//         //let a = external_handler_event_loop.;

//         // let internal_handler_event_loop = spawn(async move {
//         //     let mut client: Option<Client> = None;
//         //     let mut shutdown_signal: Option<oneshot::Sender<()>> = None;

//         //     loop {
//         //         select! {
//         //             res = engine_internal_rx.connect_rx.recv() => {

//         //             }
//         //             res = engine_internal_rx.shutdown_rx.recv() => {
//         //                 let res = res.unwrap();
//         //                 shutdown_signal = Some(res);
//         //                 tracker.close();
//         //             }
//         //             //
//         //             _ = tracker.wait() => {
//         //                 break;
//         //             }

//         //         }
//         //     }

//         //     if let Some(signal) = shutdown_signal {
//         //         let _ = signal.send(());
//         //     }
//         // });

//         // let join_fn = async move {
//         //     external_handler_event_loop.await.unwrap();
//         //     internal_handler_event_loop.await.unwrap();
//         // };

//         // (engine_external_tx, event_loop)
//         engine_external_tx

//         // for awaiter in shutdown_awaiters {
//         //     let _ = awaiter.send(());
//         // }
//     }

//     #[cfg(test)]
//     pub mod tests {
//         use matrix_sdk::Client;
//         use rand::distr::{Alphanumeric, SampleString};
//         use test_log::test;
//         use tokio_util::sync::CancellationToken;

//         use crate::matrix_engine::{self, create_engine_external_channel};

//         #[test(tokio::test)]
//         async fn connect_succ() {
//             //let cancelation_token = CancellationToken::new();
//             //let (engine_tx, engine_rx) = create_engine_external_channel();
//             let tx = matrix_engine::run().await;
//             tx.connect("http://localhost:8008");

//             tx.shutdown().await;

//             // let mut client: Option<Client> = None;

//             // let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
//             // let req = super::Req {
//             //     server_url: String::from("http://localhost:8008"),
//             // };

//             // handle(&mut client, req).await.unwrap();
//         }
//     }

//     pub mod connect {
//         use matrix_sdk::{Client, ClientBuildError, config::SyncSettings};
//         use ruma::api::{MatrixVersion, client::session::get_login_types::v3::LoginType};
//         use thiserror::Error;
//         use tracing::{error, trace, trace_span};
//         use url::{ParseError, Url};

//         #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//         pub struct Req {
//             pub server_url: String,
//         }

//         #[derive(Clone, Debug)]
//         pub struct Res {
//             pub server_version: MatrixVersion,
//             pub login_flow: Vec<LoginType>,
//         }

//         pub async fn handle(client: &mut Option<Client>, req: Req) -> Result<Res, HandleErr> {
//             let new_client = Client::builder()
//                 .homeserver_url(req.server_url)
//                 .handle_refresh_tokens()
//                 .build()
//                 .await
//                 .map_err(|err| match err {
//                     ClientBuildError::Url(err) => HandleErr::InvalidUrl(err),
//                     err => HandleErr::from(err),
//                 })?;

//             let version_res = new_client.server_versions().await?;
//             // let login_res = new_client.matrix_auth().get_login_types().await?;

//             *client = Some(new_client);

//             let res = Res {
//                 server_version: *version_res.first().unwrap(),
//                 login_flow: login_res.flows,
//             };

//             trace!("{res:#?}");

//             Ok(res)
//         }

//         #[derive(Error, Debug)]
//         pub enum HandleErr {
//             #[error("matrix engine is not running")]
//             MatrixEngineIsNotRunning,

//             #[error("failed to connect {0}")]
//             ConnectionFailed(#[from] matrix_sdk::HttpError),

//             #[error("invalid url {0}")]
//             InvalidUrl(#[from] ParseError),

//             #[error("client build error")]
//             ClientBuildError(#[from] ClientBuildError),
//         }

//         // #[cfg(test)]
//         // mod tests {
//         //     use matrix_sdk::Client;
//         //     use rand::distr::{Alphanumeric, SampleString};
//         //     use test_log::test;

//         //     use super::handle;

//         //     #[test(tokio::test)]
//         //     async fn connect_succ() {
//         //         let mut client: Option<Client> = None;

//         //         let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
//         //         let req = super::Req {
//         //             server_url: String::from("http://localhost:8008"),
//         //         };

//         //         handle(&mut client, req).await.unwrap();
//         //     }
//         // }
//     }

//     pub mod discovery {
//         use matrix_sdk::{Client, ClientBuildError, config::SyncSettings};
//         use ruma::api::{MatrixVersion, client::session::get_login_types::v3::LoginType};
//         use thiserror::Error;
//         use tracing::{error, trace, trace_span};
//         use url::{ParseError, Url};

//         #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//         pub struct Req {
//             pub server_url: String,
//         }

//         #[derive(Clone, Debug)]
//         pub struct Res {
//             pub server_version: MatrixVersion,
//             pub login_flow: Vec<LoginType>,
//         }

//         pub async fn handle(req: Req) -> Result<Res, HandleErr> {
//             let new_client = Client::builder()
//                 .homeserver_url(&req.server_url)
//                 .build()
//                 .await
//                 .map_err(|err| match err {
//                     ClientBuildError::Url(err) => HandleErr::InvalidUrl(err),
//                     err => HandleErr::from(err),
//                 })?;

//             let version_res = new_client.server_versions().await?;
//             let login_res = new_client.matrix_auth().get_login_types().await?;

//             let res = Res {
//                 server_version: *version_res.first().unwrap(),
//                 login_flow: login_res.flows,
//             };

//             trace!("{res:#?}");

//             Ok(res)
//         }

//         #[derive(Error, Debug)]
//         pub enum HandleErr {
//             #[error("failed to connect {0}")]
//             ConnectionFailed(#[from] matrix_sdk::HttpError),

//             #[error("invalid url {0}")]
//             InvalidUrl(#[from] ParseError),

//             #[error("client build error")]
//             ClientBuildError(#[from] ClientBuildError),
//         }

//         // #[cfg(test)]
//         // mod tests {
//         //     use matrix_sdk::Client;
//         //     use rand::distr::{Alphanumeric, SampleString};
//         //     use test_log::test;

//         //     use super::handle;

//         //     #[test(tokio::test)]
//         //     async fn discover_succ() {
//         //         let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
//         //         let req = super::Req {
//         //             server_url: String::from("http://localhost:8008"),
//         //         };

//         //         handle(req).await.unwrap();
//         //     }
//         // }
//     }

//     pub mod login_username {
//         use matrix_sdk::{Client, ClientBuildError, config::SyncSettings};
//         use thiserror::Error;
//         use tokio::sync::oneshot;
//         use url::{ParseError, Url};

//         #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//         pub struct Req {
//             pub server_url: String,
//             pub username: String,
//             pub password: String,
//         }

//         pub type Res = ();

//         pub async fn handle(
//             client: Option<Client>,
//             req: Req,
//             // res: oneshot::Sender<Result<Res, HandleErr>>,
//         ) -> Result<Res, HandleErr> {
//             let client = client.ok_or(HandleErr::ClientIsNotConnected)?;
//             // let new_client = Client::builder()
//             //     .homeserver_url(&req.server_url)
//             //     .handle_refresh_tokens()
//             //     .build()
//             //     .await
//             //     .map_err(|err| match err {
//             //         ClientBuildError::Url(err) => HandleErr::InvalidUrl(err),
//             //         err => HandleErr::from(err),
//             //     })?;

//             let login_res = client
//                 .matrix_auth()
//                 .login_username(req.username, &req.password)
//                 .initial_device_display_name("login test")
//                 .await?;

//             let sync_settings = SyncSettings::new();
//             let result = client.sync_once(sync_settings).await;
//             let sync_response = match result {
//                 Ok(r) => r,
//                 Err(matrix_sdk::Error::Url(parse_err)) => {
//                     return Err(HandleErr::InvalidUrl(parse_err));
//                 }
//                 Err(err) => {
//                     return Err(err.into());
//                 }
//             };
//             //*client = Some(new_client);

//             Ok(())
//         }

//         #[derive(Error, Debug)]
//         pub enum HandleErr {
//             #[error("client is not connected to any matrix server, login cannot be performed")]
//             ClientIsNotConnected,

//             #[error("failed to connect")]
//             ConnectionFailed(#[from] matrix_sdk::Error),

//             #[error("invalid url {0}")]
//             InvalidUrl(#[from] ParseError),

//             #[error("client build error")]
//             ClientBuildError(#[from] ClientBuildError),
//             // #[error(transparent)]
//             // Other(#[from] anyhow::Error),
//         }

//         // #[cfg(test)]
//         // mod tests {
//         //     use matrix_sdk::Client;
//         //     use rand::distr::{Alphanumeric, SampleString};
//         //     use test_log::test;

//         //     use super::handle;

//         //     #[test(tokio::test)]
//         //     async fn login_succ() {
//         //         let mut client: Option<Client> = None;

//         //         let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
//         //         let req = super::Req {
//         //             server_url: String::from("http://localhost:8008"),
//         //             username: rand_str.clone(),
//         //             password: rand_str.clone(),
//         //         };

//         //         //handle(&mut client, req).await.unwrap();
//         //     }

//         //     #[test(tokio::test)]
//         //     #[should_panic(expected = "wrong server url")]
//         //     async fn login_fail() {
//         //         let mut client: Option<Client> = None;

//         //         let rand_str = Alphanumeric.sample_string(&mut rand::rng(), 16);
//         //         let req = super::Req {
//         //             server_url: String::from("localhost:6969"),
//         //             username: rand_str.clone(),
//         //             password: rand_str.clone(),
//         //         };

//         //         //handle(&mut client, req).await.unwrap();
//         //     }
//         // }
//     }
// }

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
