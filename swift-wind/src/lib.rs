pub mod matrix_service {

    pub mod bevy_matrix_app {
        use std::thread::{self, JoinHandle};

        use crate::matrix_service::{
            bevy_tokio::TokioPlugin,
            reactive_runner_plugin::{ReactiveRunner, ReactiveRunnerPlugin},
        };
        use bevy::app::App;

        pub fn run_matrix_app() -> (JoinHandle<()>, ReactiveRunner) {
            let (reactive_runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
            let handle = thread::spawn(move || {
                App::new()
                    .add_plugins(reactive_runner_plugin)
                    .add_plugins(TokioPlugin::new())
                    .run();
            });
            (handle, reactive_runner)
        }
    }

    pub mod reactive_runner_plugin {
        use async_channel::TryRecvError;
        use bevy::app::{App, AppExit};
        use bevy::prelude::*;
        use tracing::trace;

        pub enum Req {
            Data(Box<dyn FnOnce(&mut App) + Sync + Send>),
            Tick,
        }

        #[derive(Debug, Clone)]
        pub struct ReactiveRunnerPlugin {
            pub tx: async_channel::Sender<Req>,
            pub rx: async_channel::Receiver<Req>,
        }

        impl ReactiveRunnerPlugin {
            pub fn new() -> (Self, ReactiveRunner) {
                let (tx, rx) = async_channel::unbounded::<Req>();
                let plugin = ReactiveRunnerPlugin { tx: tx.clone(), rx };
                let reactive_runner = ReactiveRunner { tx };
                (plugin, reactive_runner)
            }
        }

        #[derive(Resource, Clone)]
        pub struct ReactiveRunner {
            pub tx: async_channel::Sender<Req>,
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
            // pub async fn req<>() {

            // }
        }

        impl Plugin for ReactiveRunnerPlugin {
            fn build(&self, app: &mut App) {
                let rx: async_channel::Receiver<Req> = self.rx.clone();
                app.insert_resource(ReactiveRunner {
                    tx: self.tx.clone(),
                });
                app.set_runner(move |mut app| -> AppExit {
                    app.finish();
                    app.cleanup();

                    loop {
                        if let Some(exit) = tick_blocking(&mut app, &rx) {
                            return exit;
                        }
                    }
                });
            }
        }

        pub fn tick_blocking(app: &mut App, rx: &async_channel::Receiver<Req>) -> Option<AppExit> {
            let mut i = 0_usize;
            loop {
                let req = if i == 0 {
                    trace!("waiting for req");
                    match rx.recv_blocking() {
                        Ok(v) => v,
                        Err(err) => {
                            return Some(AppExit::Error(1.try_into().unwrap()));
                        }
                    }
                } else {
                    match rx.try_recv() {
                        Ok(v) => v,
                        Err(TryRecvError::Empty) => {
                            break;
                        }
                        Err(TryRecvError::Closed) => {
                            return Some(AppExit::Error(1.try_into().unwrap()));
                        }
                    }
                };

                trace!("req received");
                match req {
                    Req::Data(data) => {
                        (data)(app);
                    }
                    Req::Tick => {}
                }

                i += 1;
            }

            trace!("tick");
            app.update();
            app.should_exit()
        }

        #[cfg(test)]
        mod tests {
            use crate::matrix_service::bevy_tick_counter::{TickCount, TickPlugin};

            use super::ReactiveRunnerPlugin;
            use bevy::prelude::*;
            use std::thread;
            use test_log::test;

            #[test]
            fn recv_tick() {
                let (reactive_runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
                let (tick_plugin, tick_count) = TickPlugin::new();

                let handle = thread::spawn(move || {
                    let mut app = App::new();
                    app.add_plugins(tick_plugin);
                    app.add_plugins(reactive_runner_plugin);
                    app.run();

                    tick_count.get()
                });

                reactive_runner.send_fn_blocking(move |app| {
                    let world = app.world_mut();
                    world.send_event(AppExit::Success);
                });

                let tick_count = handle.join().unwrap();
                assert_eq!(tick_count, 1);
            }

            #[test]
            fn recv_many_ticks() {
                let (reactive_runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
                let (tick_plugin, tick_count) = TickPlugin::new();

                let handle = thread::spawn(move || {
                    let mut app = App::new();
                    app.add_plugins(tick_plugin);
                    app.add_plugins(reactive_runner_plugin);
                    app.run();

                    tick_count.get()
                });

                for _ in 0..100 {
                    reactive_runner.send_tick_blocking();
                }

                reactive_runner.send_fn_blocking(move |app| {
                    let world = app.world_mut();
                    world.send_event(AppExit::Success);
                });

                let tick_count = handle.join().unwrap();
                assert_eq!(tick_count, 1);
            }
        }
    }

    pub mod bevy_tick_counter {
        use std::sync::{Arc, RwLock};

        use bevy::prelude::*;

        #[derive(Debug, Default, Clone)]
        pub struct TickPlugin {
            pub tick_count: TickCount,
        }

        impl TickPlugin {
            pub fn new() -> (Self, TickCount) {
                let tick_count = TickCount::new();
                let plugin = TickPlugin {
                    tick_count: tick_count.clone(),
                };
                (plugin, tick_count)
            }
        }

        impl Plugin for TickPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(Update, tick_count);
                app.insert_resource(self.tick_count.clone());
            }
        }

        #[derive(Resource, Debug, Default, Clone)]
        pub struct TickCount {
            pub count: Arc<RwLock<usize>>,
        }

        impl TickCount {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn inc(&self) {
                let count = &mut *self.count.write().unwrap();
                *count += 1;
            }

            pub fn get(&self) -> usize {
                *self.count.read().unwrap()
            }
        }

        pub fn tick_count(mut tick_count: ResMut<TickCount>) {
            trace!("IM COUNTING;");
            tick_count.inc();
        }
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
            pub fn spawn<F>(&self, mut f: F)
            where
                F: Future<Output = ()> + Send + Sync + 'static,
            {
                self.tokio_rt.spawn(async move {
                    f.await;
                });
            }

            // pub fn spawn_result<F>(&self, mut f: F)
            // where
            //     F: Future<Output = Result<(), anyhow::Error>> + Send + Sync + 'static,
            // {
            //     self.tokio_rt.spawn(async move {
            //         let err f.await;
            //     });
            // }

            pub fn spawn_with_output<T, F, F2, R>(&self, mut f: F, mut callback: F2)
            where
                T: Send + Sync + 'static,
                F: FnOnce() -> R + Send + Sync + 'static,
                R: Future<Output = T> + Send + Sync + 'static,
                F2: FnOnce(&mut App, T) + Send + Sync + 'static,
            {
                let reactive_runtime = self.reactive_runner.clone();
                self.tokio_rt.spawn(async move {
                    let result = f().await;
                    reactive_runtime
                        .send_fn(move |app| {
                            callback(app, result);
                        })
                        .await;
                });
            }
        }
    }

    pub mod bevy_matrix_api {

        use std::{
            hash::{DefaultHasher, Hash, Hasher},
            ops::Deref,
        };

        use bevy::prelude::*;
        use url::Url;

        use crate::matrix_service::reactive_runner_plugin::ReactiveRunner;

        #[derive(Debug, Clone, Component)]
        pub struct APITx<ResponseType> {
            pub tx: async_channel::Sender<ResponseType>,
        }

        #[derive(Debug, Clone, Component, PartialEq, Eq)]
        pub struct APIReqHash {
            pub hash: u64,
        }

        impl APIReqHash {
            pub fn new<T: Hash>(value: &T) -> Self {
                let mut hasher = DefaultHasher::new();
                value.hash(&mut hasher);
                let hash = hasher.finish();
                Self { hash }
            }
        }

        #[derive(Debug, Clone, Component)]
        pub struct APIProccesingLabel;

        // #[derive(Debug, Clone, Component)]
        // pub struct Request<ReqType> {
        //     pub req: async_channel::Sender<ReqType>,
        // }

        impl<ResType> Deref for APITx<ResType> {
            type Target = async_channel::Sender<ResType>;
            fn deref(&self) -> &Self::Target {
                &self.tx
            }
        }

        impl<ResType> APITx<ResType> {
            pub fn new(tx: async_channel::Sender<ResType>) -> Self {
                Self { tx }
            }
        }

        pub trait MakeReq {
            async fn make_req<R: Send + Sync + 'static>(
                &self,
                req: impl Bundle + Hash,
            ) -> async_channel::Receiver<R>;
            fn make_req_blocking<R: Send + Sync + 'static>(
                &self,
                req: impl Bundle + Hash,
            ) -> async_channel::Receiver<R>;
        }

        impl MakeReq for ReactiveRunner {
            async fn make_req<R: Send + Sync + 'static>(
                &self,
                req: impl Bundle + Hash,
            ) -> async_channel::Receiver<R> {
                let (tx, rx) = async_channel::unbounded::<R>();
                self.send_fn(move |app| {
                    let world = app.world_mut();
                    let mut commands = world.commands();
                    trace!("SPAWNING");
                    let req_hash = APIReqHash::new(&req);
                    commands.spawn((APITx::<R>::new(tx), req, req_hash));
                })
                .await;
                rx
            }

            fn make_req_blocking<R: Send + Sync + 'static>(
                &self,
                req: impl Bundle + Hash,
            ) -> async_channel::Receiver<R> {
                let (tx, rx) = async_channel::unbounded::<R>();
                self.send_fn_blocking(move |app| {
                    let world = app.world_mut();
                    let mut commands = world.commands();
                    trace!("SPAWNING");
                    let req_hash = APIReqHash::new(&req);
                    commands.spawn((APITx::<R>::new(tx), req, req_hash));
                });
                rx
            }
        }

        #[cfg(test)]
        mod tests {
            use bevy::{app::App, ecs::component::Component};
            use log::trace;
            use test_log::test;
            use tokio::runtime::Runtime;

            use crate::matrix_service::{
                bevy_matrix_api::{APITx, MakeReq},
                bevy_matrix_reqwest::MatrixReqwestPlugin,
                bevy_tokio::TokioPlugin,
                reactive_runner_plugin::{ReactiveRunnerPlugin, tick_blocking},
            };

            #[derive(Debug, Clone, Copy, Default, Component, Hash)]
            struct Foo;

            #[test]
            fn make_req() {
                let (runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
                let runner_rx = runner_plugin.rx.clone();

                let mut app = App::new();

                app.add_plugins((
                    runner_plugin,
                    TokioPlugin::new(),
                    MatrixReqwestPlugin::new(),
                ));

                let request_rx = reactive_runner.make_req_blocking::<()>(Foo);
                tick_blocking(&mut app, &runner_rx);
                trace!("hello");
                let responses_and_requests = {
                    let mut world = app.world_mut();
                    let mut q = world.query::<(&APITx<()>, &Foo)>();
                    let v = q
                        .iter(&mut world)
                        .map(|v| (v.0.clone(), v.1.clone()))
                        .collect::<Vec<(APITx<()>, Foo)>>();
                    v
                };
                let responses_and_requests_len = responses_and_requests.len();
                assert_eq!(responses_and_requests_len, 1);
                let (response, request) = responses_and_requests.first().unwrap();
                response.tx.send_blocking(()).unwrap();

                let _response = request_rx.recv_blocking().unwrap();
                trace!("complete");
            }
        }
    }

    pub mod bevy_matrix_reqwest {

        use bevy::prelude::*;
        use reqwest::StatusCode;
        use thiserror::Error;
        use tokio_util::bytes::Bytes;

        use crate::matrix_service::bevy_tokio::AsyncQueue;

        #[derive(Debug, Clone, Copy, Default)]
        pub struct MatrixReqwestPlugin {}

        impl MatrixReqwestPlugin {
            pub fn new() -> Self {
                Self::default()
            }
        }

        impl Plugin for MatrixReqwestPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(Update, request);
            }
        }

        #[derive(Component, Debug, Clone, Copy, Default)]
        pub struct HttpReq;

        #[derive(Component, Debug, Clone)]
        pub struct HttpRes(pub Result<Bytes, ReqError>);

        #[derive(Component, Debug, Clone, Copy, Default)]
        pub struct HTTPProccesingLabel;

        pub fn request(
            mut commands: Commands,
            reqs: Query<(Entity, &HttpReq), (Without<HTTPProccesingLabel>, Without<HttpRes>)>,
            async_queue: AsyncQueue,
        ) {
            for (e, http_req) in reqs.iter() {
                commands.entity(e).insert(HTTPProccesingLabel);
                async_queue.spawn_with_output(
                    async move || -> Result<Bytes, reqwest::Error> {
                        let res = reqwest::get("http://localhost:8008/_matrix/client/versions")
                            .await?
                            .bytes()
                            .await?;

                        Ok(res)
                    },
                    move |app, res| {
                        let world = app.world_mut();
                        let mut commands = world.commands();
                        commands
                            .entity(e)
                            .insert(HttpRes(res.map_err(|err| {
                                ReqError::ConnectionFailed(err.status().unwrap_or_default())
                            })))
                            .remove::<HTTPProccesingLabel>();
                        //
                    },
                );
            }
        }

        #[derive(Error, Debug, Clone, PartialEq, Eq)]
        pub enum ReqError {
            #[error("failed with error code {0}")]
            ConnectionFailed(StatusCode),
        }

        #[cfg(test)]
        mod tests {
            use bevy::app::App;
            use test_log::test;
            use tracing::trace;

            use crate::matrix_service::{
                bevy_matrix_reqwest::{HttpReq, HttpRes},
                bevy_tokio::TokioPlugin,
                reactive_runner_plugin::{self, ReactiveRunnerPlugin, tick_blocking},
            };

            use super::MatrixReqwestPlugin;

            #[test]
            pub fn make_req() {
                let (runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
                let runner_rx = runner_plugin.rx.clone();

                let mut app = App::new();

                {
                    let world = app.world_mut();
                    let mut commands = world.commands();
                    commands.spawn(HttpReq);
                    reactive_runner.send_tick_blocking();
                }

                app.add_plugins((
                    runner_plugin,
                    TokioPlugin::new(),
                    MatrixReqwestPlugin::new(),
                ));

                tick_blocking(&mut app, &runner_rx);
                tick_blocking(&mut app, &runner_rx);

                let res = {
                    let world = app.world_mut();
                    let mut q = world.query::<&HttpRes>();
                    let mut res_vec = Vec::<HttpRes>::new();
                    for res in q.iter(world) {
                        res_vec.push(res.clone());
                    }
                    res_vec
                };

                assert_eq!(res.len(), 1);
                assert!(res[0].0.is_ok());
                trace!("{res:#?}");
            }
        }
    }
    pub mod bevy_matrix_login_password {
        use std::{collections::HashMap, ops::Deref};

        use bevy::prelude::*;
        use reqwest::StatusCode;
        use serde::{Deserialize, Serialize};
        use thiserror::Error;
        use tokio::sync::mpsc;

        use crate::matrix_service::{
            bevy_matrix_api::{APIProccesingLabel, APITx},
            bevy_matrix_reqwest::{HttpReq, HttpRes, ReqError, request},
            bevy_tokio::AsyncQueue,
            reactive_runner_plugin::ReactiveRunner,
        };

        #[derive(Debug, Component, Clone)]
        pub struct MatrixLoginPasswordParams;

        #[derive(Debug, Component, Clone, PartialEq, Eq)]
        pub enum MatrixLoginPasswordProgess {
            WaitingForExecution,
            Executed,
            Completed(Result<MatrixVersions, CompletedErr>),
        }

        #[derive(Debug, Component, Clone, Error, PartialEq, Eq)]
        pub enum CompletedErr {
            #[error("{0}")]
            ConnectionFailed(#[from] ReqError),

            #[error("deserialize error")]
            DeserializeError,
        }

        impl Default for MatrixLoginPasswordProgess {
            fn default() -> Self {
                Self::WaitingForExecution
            }
        }

        #[derive(Debug, Component, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
        pub struct MatrixVersions {
            pub versions: Vec<String>,
            pub unstable_features: HashMap<String, bool>,
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub struct MatrixVersionsPlugin {}

        impl Plugin for MatrixVersionsPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(
                    Update,
                    (
                        matrix_versions_executor.before(request),
                        matrix_versions_finish.after(request),
                    ),
                );
            }
        }

        impl MatrixVersionsPlugin {
            pub fn new() -> Self {
                Self::default()
            }
        }

        pub fn matrix_versions_executor(
            requests: Query<
                (
                    Entity,
                    &MatrixLoginPasswordParams,
                    &APITx<MatrixLoginPasswordProgess>,
                ),
                (Without<APIProccesingLabel>),
            >,
            mut commands: Commands,
        ) {
            trace!("matrix versions init");
            for (entity, _params, api_tx) in requests {
                trace!("versions req made");
                if let Err(err) = api_tx.send_blocking(MatrixLoginPasswordProgess::Executed) {
                    warn!("matrix versions channel disconnected");
                    commands.entity(entity).despawn();
                    continue;
                }
                commands
                    .entity(entity)
                    .insert((APIProccesingLabel, HttpReq));
            }
        }

        pub fn matrix_versions_finish(
            responses: Query<(Entity, &HttpRes, &APITx<MatrixLoginPasswordProgess>)>,
            mut commands: Commands,
        ) {
            for (entity, res, tx) in responses {
                commands.entity(entity).despawn();
                trace!("matrix versions raw res: {res:#?}");
                let res = match res.0.as_ref() {
                    Ok(v) => v.as_ref(),
                    Err(err) => {
                        warn!("matrix versions res err: {err}");
                        tx.send_blocking(MatrixLoginPasswordProgess::Completed(Err(
                            CompletedErr::from(err.clone()),
                        )));
                        continue;
                    }
                };
                let res: MatrixVersions = match serde_json::from_slice(res) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("matrix versions serde err: {err}");
                        tx.send_blocking(MatrixLoginPasswordProgess::Completed(Err(
                            CompletedErr::DeserializeError,
                        )));
                        continue;
                    }
                };
                trace!("matrix versions serde res: {res:#?}");

                tx.send_blocking(MatrixLoginPasswordProgess::Completed(Ok(res)))
                    .unwrap();
            }
        }

        #[cfg(test)]
        mod tests {
            use bevy::app::App;
            use test_log::test;

            use crate::matrix_service::{
                bevy_matrix_api::MakeReq,
                bevy_matrix_reqwest::MatrixReqwestPlugin,
                bevy_matrix_versions::{
                    MatrixVersionsParams, MatrixVersionsPlugin, MatrixVersionsProgess,
                },
                bevy_tokio::TokioPlugin,
                reactive_runner_plugin::{ReactiveRunnerPlugin, tick_blocking},
            };

            #[test]
            fn req_login_password() {
                let (runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
                let runner_rx = runner_plugin.rx.clone();

                let mut app = App::new();

                app.add_plugins((
                    runner_plugin,
                    TokioPlugin::new(),
                    MatrixReqwestPlugin::new(),
                    MatrixVersionsPlugin::new(),
                ));

                let request_rx = reactive_runner
                    .make_req_blocking::<MatrixVersionsProgess>(MatrixVersionsParams);
                tick_blocking(&mut app, &runner_rx);
                tick_blocking(&mut app, &runner_rx);
                let r = request_rx.recv_blocking().unwrap();
                assert_eq!(r, MatrixVersionsProgess::Executed);
                let r = request_rx.recv_blocking().unwrap();
                matches!(r, MatrixVersionsProgess::Completed(_));
                let r = request_rx.recv_blocking();
                assert!(r.is_err());
            }
        }
    }

    pub mod bevy_matrix_versions {
        use std::{collections::HashMap, ops::Deref};

        use bevy::prelude::*;
        use reqwest::StatusCode;
        use serde::{Deserialize, Serialize};
        use thiserror::Error;
        use tokio::sync::mpsc;

        use crate::matrix_service::{
            bevy_matrix_api::{APIProccesingLabel, APITx},
            bevy_matrix_reqwest::{HttpReq, HttpRes, ReqError, request},
            bevy_tokio::AsyncQueue,
            reactive_runner_plugin::ReactiveRunner,
        };

        use super::bevy_matrix_api::APIReqHash;

        #[derive(Debug, Component, Clone, Hash)]
        pub struct MatrixVersionsParams;

        #[derive(Debug, Component, Clone, PartialEq, Eq)]
        pub enum MatrixVersionsProgess {
            WaitingForExecution,
            Executed,
            Completed(Result<MatrixVersions, CompletedErr>),
        }

        #[derive(Debug, Component, Clone, Error, PartialEq, Eq)]
        pub enum CompletedErr {
            #[error("{0}")]
            ConnectionFailed(#[from] ReqError),

            #[error("deserialize error")]
            DeserializeError,
        }

        impl Default for MatrixVersionsProgess {
            fn default() -> Self {
                Self::WaitingForExecution
            }
        }

        #[derive(Debug, Component, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
        pub struct MatrixVersions {
            pub versions: Vec<String>,
            pub unstable_features: HashMap<String, bool>,
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub struct MatrixVersionsPlugin {}

        impl Plugin for MatrixVersionsPlugin {
            fn build(&self, app: &mut App) {
                app.add_systems(
                    Update,
                    (
                        matrix_versions_executor.before(request),
                        matrix_versions_finish.after(request),
                    ),
                );
            }
        }

        impl MatrixVersionsPlugin {
            pub fn new() -> Self {
                Self::default()
            }
        }

        pub fn matrix_versions_executor(
            requests: Query<
                (
                    Entity,
                    &MatrixVersionsParams,
                    &APIReqHash,
                    &APITx<MatrixVersionsProgess>,
                ),
                (Without<APIProccesingLabel>),
            >,
            mut commands: Commands,
        ) {
            trace!("matrix versions init");
            for (entity, _params, api_req_hash, api_tx) in requests {
                trace!("versions req made");
                if let Err(err) = api_tx.send_blocking(MatrixVersionsProgess::Executed) {
                    warn!("matrix versions channel disconnected");
                    commands.entity(entity).despawn();
                    continue;
                }
                commands
                    .entity(entity)
                    .insert((APIProccesingLabel, HttpReq));
                // commands.spawn((api_req_hash.clone(), HttpReq));
            }
        }

        pub fn matrix_versions_finish(
            responses: Query<(Entity, &HttpRes, &APIReqHash, &APITx<MatrixVersionsProgess>)>,
            mut commands: Commands,
        ) {
            for (entity, res, api_req_hash, tx) in responses {
                commands.entity(entity).despawn();
                trace!("matrix versions raw res: {res:#?}");
                let res = match res.0.as_ref() {
                    Ok(v) => v.as_ref(),
                    Err(err) => {
                        warn!("matrix versions res err: {err}");
                        let _ = tx.send_blocking(MatrixVersionsProgess::Completed(Err(
                            CompletedErr::from(err.clone()),
                        )));
                        continue;
                    }
                };
                let res: MatrixVersions = match serde_json::from_slice(res) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!("matrix versions serde err: {err}");
                        let _ = tx.send_blocking(MatrixVersionsProgess::Completed(Err(
                            CompletedErr::DeserializeError,
                        )));
                        continue;
                    }
                };
                trace!("matrix versions serde res: {res:#?}");

                tx.send_blocking(MatrixVersionsProgess::Completed(Ok(res)))
                    .unwrap();
            }
        }

        #[cfg(test)]
        mod tests {
            use bevy::app::App;
            use test_log::test;
            use tracing::trace;

            use crate::matrix_service::{
                bevy_matrix_api::MakeReq,
                bevy_matrix_reqwest::MatrixReqwestPlugin,
                bevy_matrix_versions::{
                    MatrixVersionsParams, MatrixVersionsPlugin, MatrixVersionsProgess,
                },
                bevy_tokio::TokioPlugin,
                reactive_runner_plugin::{ReactiveRunnerPlugin, tick_blocking},
            };

            #[test]
            fn req_versions() {
                let (runner_plugin, reactive_runner) = ReactiveRunnerPlugin::new();
                let runner_rx = runner_plugin.rx.clone();

                let mut app = App::new();

                app.add_plugins((
                    runner_plugin,
                    TokioPlugin::new(),
                    MatrixReqwestPlugin::new(),
                    MatrixVersionsPlugin::new(),
                ));

                let request_rx = reactive_runner
                    .make_req_blocking::<MatrixVersionsProgess>(MatrixVersionsParams);
                tick_blocking(&mut app, &runner_rx);
                tick_blocking(&mut app, &runner_rx);
                let r = request_rx.recv_blocking().unwrap();
                assert_eq!(r, MatrixVersionsProgess::Executed);
                let r = request_rx.recv_blocking().unwrap();
                assert!(matches!(r, MatrixVersionsProgess::Completed(Ok(_))));
                let r = request_rx.recv_blocking();
                assert!(r.is_err());
            }
        }
    }
}
