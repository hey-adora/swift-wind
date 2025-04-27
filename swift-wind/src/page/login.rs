use chrono::prelude::*;
use dioxus_router::prelude::navigator;
use freya::dioxus_core;
use freya::prelude::*;
use tracing::trace;

use crate::CLIENT;
use crate::MatrixClientState;
use crate::components::additional_authorization::AdditonalAuthHandler;
use crate::components::additional_authorization::AuthenticationState;
use crate::components::form::FormField;
use crate::hook::CommonUserAuthData;
use crate::hook::connect::use_matrix_connect;
use crate::hook::login::use_matrix_login;
use crate::hook::submit_additional_auth::AdditionalAuthType;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[component]
pub fn Login() -> Element {
    let mut form_url = use_signal(|| String::from("http://127.0.0.1:8008"));
    let mut form_username = use_signal(|| String::new());
    let mut form_password = use_signal(|| String::new());
    let navigator = navigator();

    let (mut get_connect_err, run_matrix_connect) = use_matrix_connect(move || {
        //navigator.push("/login");
    });

    let on_connect = {
        let mut run_matrix_connect = run_matrix_connect.clone();
        move |_| {
            let url = form_url();
            run_matrix_connect(url);
        }
    };

    use_effect({
        let mut run_matrix_connect = run_matrix_connect.clone();
        move || {
            return;
            let url = &*form_url.read_unchecked();
            trace!("noooooooooo {}", url);
            run_matrix_connect(url.clone());
            // let url = &*form_url.read_unchecked();
            // run_matrix_connect(url.clone());
        }
    });

    let errors = use_memo(move || {
        let a = get_connect_err();
        if !a.is_empty() {
            return a;
        }

        let b = CLIENT();
        if let MatrixClientState::Error(err) = b {
            return err;
        }

        String::new()
    });

    let mut form = use_signal(LoginForm::default);

    let (error_string, mut run_matrix_login, state_machine) = use_matrix_login(move || {});

    // let on_login = move |_| {
    //     let auth_data = CommonUserAuthData {
    //         username: form().username,
    //         password: form().password,
    //         session_id: None,
    //     };
    //     run_matrix_login(auth_data);
    // };

    rsx! {

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

                FormField {
                    name: "Server",
                    value: form_url,
                    onchange: move |txt| {
                        *form_url.write() = txt;
                        //get_connect_err.write().clear();
                     },
                }

                match CLIENT() {
                    MatrixClientState::Connecting => {
                        rsx!(
                            Loader {}
                        )
                    }
                    MatrixClientState::Connected(_) => {
                        rsx!(
                            FormField {
                                name: "Username",
                                value: form_username,
                                onchange: move |txt| {
                                    *form_username.write() = txt;
                                 },
                            }

                            FormField {
                                name: "Password",
                                value: form_password,
                                hidden: true,
                                onchange: move |txt| {
                                    *form_password.write() = txt;
                                 },
                            }

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
                                    to: crate::Route::Register,


                                    rect {
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
                        )
                    }
                    MatrixClientState::Error(err) => {
                        let time = Utc::now();
                        rsx!(
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
                                    "Try Again"
                                }
                            }

                            label {
                                color: "red",

                                "{time} error connecting: {err}"
                            }
                        )
                    }
                    MatrixClientState::Disconnected => {
                        rsx!(
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
                                    "Connect"
                                }
                            }
                        )
                    }
                }
            }
        }


    }
}

// #[cfg(test)]
// mod login_tests {
//     use matrix_driver::kernel;
//     //use log::LevelFilter;
//     use matrix_sdk::{Client, config::SyncSettings, reqwest::Url};
//     use ruma::events::room::message::RoomMessageEventContent;
//     use test_log::test;
//     use tokio::sync::mpsc;
//     use tokio_util::task::TaskTracker;
//     use tracing::{debug, error, level_filters::LevelFilter};

//     mod matrix_driver {
//         use std::{collections::HashMap, future};

//         use futures::{FutureExt, future::BoxFuture};
//         use matrix_sdk::{Client, reqwest::Url};
//         use ruma::api::client::session::login::v3::Response;
//         use tokio::{
//             select,
//             sync::{mpsc, oneshot},
//         };
//         use tokio_util::task::TaskTracker;

//         pub trait Driver {
//             fn connect(&mut self, server: Url) -> BoxFuture<bool>;
//             fn login_with_username_password(
//                 &mut self,
//                 username: String,
//                 password: String,
//             ) -> BoxFuture<bool>;
//             fn logout(&mut self) -> BoxFuture<bool>;
//         }

//         pub struct MatrixDriver {
//             client: Option<Client>,
//             username: String,
//         }

//         impl Default for MatrixDriver {
//             fn default() -> Self {
//                 MatrixDriver {
//                     client: None,
//                     username: String::new(),
//                 }
//             }
//         }

//         impl Driver for MatrixDriver {
//             fn connect(&mut self, server: Url) -> BoxFuture<bool> {
//                 async move {
//                     let client = Client::builder()
//                         .homeserver_url(server)
//                         .handle_refresh_tokens()
//                         .build()
//                         .await;

//                     let client = match client {
//                         Ok(client) => client,
//                         Err(err) => {
//                             return false;
//                         }
//                     };

//                     self.client = Some(client);

//                     true
//                 }
//                 .boxed()
//             }

//             fn login_with_username_password(
//                 &mut self,
//                 username: String,
//                 password: String,
//             ) -> BoxFuture<bool> {
//                 async move {
//                     let client = self.client.as_ref().unwrap();
//                     let res = client
//                         .matrix_auth()
//                         .login_username(username.clone(), &password)
//                         .request_refresh_token()
//                         .await;

//                     let res = match res {
//                         Ok(res) => res,
//                         Err(err) => {
//                             return false;
//                         }
//                     };

//                     self.username = username;

//                     true
//                 }
//                 .boxed()
//             }

//             fn logout(&mut self) -> BoxFuture<bool> {
//                 async move {
//                     let client = self.client.as_ref().unwrap();
//                     // client.lo
//                     true
//                 }
//                 .boxed()
//             }
//         }

//         struct Con {
//             login_tx: mpsc::Sender<(String, String)>,
//         }

//         pub struct ReqAccount {
//             pub server: String,
//             pub username: String,
//             pub password: String,
//         }

//         pub struct MatrixConnect {
//             pub server_url: String,
//         }

//         pub struct MatrixUsernamePasswordLogin {
//             pub username: String,
//             pub password: String,
//         }

//         #[derive(Clone, Debug)]
//         pub struct KernelTx {
//             pub req_account_tx: mpsc::UnboundedSender<(oneshot::Sender<bool>, ReqAccount)>,
//             pub req_connect_tx: mpsc::UnboundedSender<bool>,
//             pub req_login_tx: mpsc::UnboundedSender<bool>,
//         }

//         #[derive(Debug)]
//         pub struct KernelRx {
//             pub add_account_res_rx: mpsc::UnboundedReceiver<(oneshot::Sender<bool>, ReqAccount)>,
//             pub connect_res_rx: mpsc::UnboundedReceiver<bool>,
//             pub login_res_rx: mpsc::UnboundedReceiver<bool>,
//         }

//         #[derive(Clone, Debug)]
//         pub struct DriverTx {
//             pub connect_tx: mpsc::UnboundedSender<(oneshot::Sender<bool>, MatrixConnect)>,
//             pub login_tx:
//                 mpsc::UnboundedSender<(oneshot::Sender<bool>, MatrixUsernamePasswordLogin)>,
//         }

//         #[derive(Debug)]
//         pub struct DriverRx {
//             pub connect_rx: mpsc::UnboundedReceiver<(oneshot::Sender<bool>, MatrixConnect)>,
//             pub login_rx:
//                 mpsc::UnboundedReceiver<(oneshot::Sender<bool>, MatrixUsernamePasswordLogin)>,
//         }

//         pub fn create_kernel_channel() -> (KernelTx, KernelRx) {
//             let (add_account_res_tx, add_account_res_rx) = mpsc::unbounded_channel();
//             let (connect_res_tx, connect_res_rx) = mpsc::unbounded_channel();
//             let (login_res_tx, login_res_rx) = mpsc::unbounded_channel();

//             let tx = KernelTx {
//                 req_account_tx: add_account_res_tx,
//                 req_login_tx: login_res_tx,
//                 req_connect_tx: connect_res_tx,
//             };
//             let rx = KernelRx {
//                 add_account_res_rx,
//                 login_res_rx,
//                 connect_res_rx,
//             };
//             (tx, rx)
//         }

//         pub fn create_driver_channel() -> (DriverTx, DriverRx) {
//             let (connect_tx, connect_rx) = mpsc::unbounded_channel();
//             let (login_tx, login_rx) = mpsc::unbounded_channel();

//             let tx = DriverTx {
//                 connect_tx,
//                 login_tx,
//             };

//             let rx = DriverRx {
//                 connect_rx,
//                 login_rx,
//             };

//             (tx, rx)
//         }

//         // impl DriverRx {
//         //     pub async fn login_req_rx(&mut self) -> impl Future<Output = Option<(String, String)>> {
//         //         self.login_req_rx.recv()
//         //     }
//         // }

//         // pub fn create_driver_channel() -> (DriverTx, DriverRx) {
//         //     let (login_res_tx, login_res_rx) = mpsc::unbounded_channel();
//         //     let (login_req_tx, login_req_rx) = mpsc::unbounded_channel();

//         //     let driver_tx = DriverTx {
//         //         login_res_rx,
//         //         login_req_tx,
//         //     };

//         //     let driver_rx = DriverRx {
//         //         login_res_tx,
//         //         login_req_rx,
//         //     };

//         //     (driver_tx, driver_rx)
//         // }

//         pub async fn kernel() {
//             let tracker = TaskTracker::new();
//             // let (add_account_res_tx, add_account_res_rx) = mpsc::unbounded_channel();
//             // let (connect_res_tx, connect_res_rx) = mpsc::unbounded_channel();
//             // let (login_res_tx, login_res_rx) = mpsc::unbounded_channel();
//             let (kernel_tx, mut kernel_rx) = create_kernel_channel();

//             let mut accounts = HashMap::<String, Box<dyn Driver>>::new();
//             let (driver_tx, driver_rx) = create_driver_channel();
//             let driver_taks = run_driver(kernel_tx, driver_rx, MatrixDriver::default());

//             // let mut add_account_fn = async move |(res, url, username, password): (
//             //     oneshot::Sender<bool>,
//             //     String,
//             //     String,
//             //     String,
//             // )| {
//             //     accounts.insert("k".to_string(), "k5".to_string());
//             //     tracing::debug!("adding account!");
//             // };

//             // let connect_fn = async move |res| {
//             //     accounts.insert("k".to_string(), "k5".to_string());
//             //     tracing::debug!("connected!");
//             // };

//             // let login_fn = async |res| {
//             //     tracing::debug!("logged in!");
//             // };

//             select! {
//                 add_account_res = kernel_rx.add_account_res_rx.recv() => {
//                     let Some((res, url, username, password)) = add_account_res else {
//                         return;
//                     };
//                     add_account(tracker);
//                 }
//                 // connected_res = kernel_rx.connect_res_rx.recv() => {
//                 //     tracker.spawn(connect_fn(connected_res));
//                 // },
//                 // login_res = kernel_rx.login_res_rx.recv() => {
//                 //     tracker.spawn(login_fn(login_res));
//                 // }
//             }

//             tracker.close();
//             tracker.wait().await;
//             //let (login_res_tx, login_res_rx) = mpsc::unbounded_channel::<Response>();
//             //login_res_rx.recv()
//         }

//         pub fn add_account(task_tracker: TaskTracker, accounts: &mut HashMap<String, String>) {
//             accounts.insert(k, v);
//             task_tracker.spawn(task);
//         }

//         pub async fn run_driver<D: Driver>(
//             response: KernelTx,
//             mut requests: DriverRx,
//             mut driver: D,
//         ) {
//             //let (login_req_tx, login_req_rx) = mpsc::unbounded_channel::<(String, String)>();

//             let connect_fn =
//                 async |driver: &mut D, (res_tx, url): (oneshot::Sender<bool>, String)| {
//                     let url = Url::parse(&url).unwrap();
//                     driver.connect(url).await;
//                     res_tx.send(true);
//                 };

//             let login_fn = async |driver: &mut D, (res_tx, (username, password))| {
//                 let res = driver
//                     .login_with_username_password(username, password)
//                     .await;
//                 response.req_login_tx.send(res).unwrap();
//             };

//             select! {
//                 connect_req = requests.connect_rx.recv() => {
//                     let Some(req) = connect_req else {
//                         return;
//                     };
//                     connect_fn(&mut driver, req).await;
//                 },
//                 login_req = requests.login_rx.recv() => {
//                     let Some(req) = login_req else {
//                         return;
//                     };
//                     login_fn(&mut driver, req).await;

//                 }
//             }
//         }

//         pub async fn driver(
//             login_res_tx: mpsc::Sender<Response>,
//             mut login_req_rx: mpsc::Receiver<(String, String)>,
//         ) {
//             let url = Url::parse("http://127.0.0.1:8008").unwrap();
//             let client = Client::builder()
//                 .homeserver_url(url)
//                 .handle_refresh_tokens()
//                 .build()
//                 .await
//                 .unwrap();

//             // loop {
//             //     select! {
//             //         login_info = login_req_rx.recv() => {
//             //             let Some((username, password)) = login_info else {
//             //                 return;
//             //             };
//             //             let resp: Response = client
//             //             .matrix_auth()
//             //             .login_username(username, &password)
//             //             .request_refresh_token()
//             //             .await
//             //             .unwrap();

//             //             login_res_tx.send(resp).await.unwrap();
//             //         }
//             //     }
//             // }

//             // let Some((username, password)) = login_req_rx.recv().await else {
//             //     return;
//             // };

//             // let resp: Response = client
//             //     .matrix_auth()
//             //     .login_username(username, &password)
//             //     .request_refresh_token()
//             //     .await
//             //     .unwrap();

//             // login_res_tx.send(resp).await.unwrap();
//         }
//     }

//     #[test(tokio::test)]
//     async fn login() {
//         let tracker = TaskTracker::new();

//         let kernel_task = kernel();
//         tracker.spawn(kernel_task);

//         kernel_task.await;
//         // tracing_subscriber::fmt()
//         //     .event_format(
//         //         tracing_subscriber::fmt::format()
//         //             .with_file(true)
//         //             .with_line_number(true),
//         //     )
//         //     .with_max_level(LevelFilter::TRACE)
//         //     .init();

//         // simple_logger::SimpleLogger::new()
//         //     .with_level(log::LevelFilter::Trace)
//         //     .init()
//         //     .unwrap();

//         // let (login_res_tx, mut login_res_rx) = mpsc::channel(1);
//         // let (login_req_tx, login_req_rx) = mpsc::channel(1);
//         // login_req_tx
//         //     .send(("test1".to_string(), "test1".to_string()))
//         //     .await
//         //     .unwrap();
//         // matrix_driver::driver(login_res_tx, login_req_rx).await;

//         // let Some(res) = login_res_rx.recv().await else {
//         //     return;
//         // };
//         // tracing::trace!("{:?}", res);

//         // let url = Url::parse("http://127.0.0.1:8008").unwrap();
//         // let client = Client::builder()
//         //     .homeserver_url(url)
//         //     .handle_refresh_tokens()
//         //     .build()
//         //     .await
//         //     .unwrap();

//         // let resp = client
//         //     .matrix_auth()
//         //     .login_username("test1", "test1")
//         //     .request_refresh_token()
//         //     .await
//         //     .unwrap();

//         // debug!("login {resp:#?}");

//         // let sync_settings = SyncSettings::default();
//         // let r = client.sync_once(sync_settings).await.unwrap();

//         // let rooms = client.rooms();

//         // for room in rooms {
//         //     let room_name = room.name().unwrap();
//         //     let content = RoomMessageEventContent::text_plain(">:[");
//         //     room.send(content).await.unwrap();
//         // }

//         //debug!("rooms {rooms:#?}");
//     }
//}
