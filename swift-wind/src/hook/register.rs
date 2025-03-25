use freya::dioxus_core;
use freya::prelude::*;
use matrix_sdk::Client;
use matrix_sdk::HttpError;
use matrix_sdk::RumaApiError;
use matrix_sdk::reqwest::Url;
use ruma::api::client::account::register::RegistrationKind;
use ruma::api::client::uiaa::AuthType;
use ruma::api::client::uiaa::Dummy;
use ruma::api::error::FromHttpResponseError;
use tracing::warn;

use super::connect::CLIENT;
use super::connect::MatrixClientState;

// pub static CLIENT: GlobalSignal<Option<Client>> = Global::new(|| None);

#[derive(Clone, Default)]
pub enum MainClient {
    #[default]
    Disconnected,
    Connected(Client),
}

#[derive(Default)]
enum GeneralLoginState {
    #[default]
    Register,
    Login,
}

#[derive(Default, Clone)]
enum RegisterState {
    #[default]
    EnteringData,
    ExtraAuthRequired {
        recaptcha: bool,
        shared_token: bool,
        email: bool,
        terms: bool,
        session_token: Option<String>,
    },
    RegisterComplete,
}

// TODO make register hook
pub fn use_matrix_register() {
    // use_rou
    let register = |username: String, password: String| {
        let MatrixClientState::Connected(client) = CLIENT() else {
            warn!("trying to register before connected");
            return;
        };
        spawn(async move {
            let mut register_request = ruma::api::client::account::register::v3::Request::new();
            register_request.password = Some(password.to_string());
            register_request.username = Some(username.to_string());
            register_request.refresh_token = true;
            register_request.kind = RegistrationKind::User;

            let resp = client
                .matrix_auth()
                .register(register_request.clone())
                .await;

            //Holy error! This is what we should expect, effectively means that the user needs to do another step of auth,
            //like a recaptcha, shared token, or read terms and conditions
            if let Err(matrix_sdk::Error::Http(HttpError::Api(FromHttpResponseError::Server(
                RumaApiError::Uiaa(info),
            )))) = resp
            {
                let mut authentication_steps = (false, false, false, false);
                for flow in &info.flows {
                    //Very crude, checks to see what auth steps we need to do next. Idealy we should handle all flows and figure out which one to use
                    for stage in &flow.stages {
                        match stage {
                            AuthType::ReCaptcha => authentication_steps.0 = true,
                            AuthType::EmailIdentity => authentication_steps.1 = true,
                            AuthType::RegistrationToken => authentication_steps.2 = true,
                            AuthType::Terms => authentication_steps.3 = true,
                            _ => {}
                        }
                    }

                    let new_state = RegisterState::ExtraAuthRequired {
                        recaptcha: authentication_steps.0,
                        shared_token: authentication_steps.1,
                        email: authentication_steps.2,
                        terms: authentication_steps.3,
                        session_token: info.session.clone(),
                    };
                    // register_state.set(new_state);
                }
            } else {
                //TODO
                //MOST will require extra auth, however here we'll need to send back the session token, device_id, user_id, and access_token
            }
        });
    };
}

#[component]
pub fn Register(homeserver: String) -> Element {
    let mut register_state = use_signal(|| RegisterState::default());
    let mut client = use_context::<Signal<MainClient>>();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);

    //When user registry data is submitted create a request and send it, this should fail initally
    let submit_on_click = move |e| {
        let homeserver = homeserver.clone();
        register_state.set(RegisterState::RegisterComplete);
        username.set("wowza".to_string());
        spawn(async move {
            let lock_client = client.write().clone();

            if let MainClient::Disconnected = lock_client {
                let tmp_cl = Client::new(Url::parse(&homeserver).unwrap()).await.unwrap();

                let mut register_request = ruma::api::client::account::register::v3::Request::new();
                register_request.password = Some(password.to_string());
                register_request.username = Some(username.to_string());
                register_request.refresh_token = true;
                register_request.kind = RegistrationKind::User;

                let resp = tmp_cl
                    .matrix_auth()
                    .register(register_request.clone())
                    .await;

                //Holy error! This is what we should expect, effectively means that the user needs to do another step of auth,
                //like a recaptcha, shared token, or read terms and conditions
                if let Err(matrix_sdk::Error::Http(HttpError::Api(
                    FromHttpResponseError::Server(RumaApiError::Uiaa(info)),
                ))) = resp
                {
                    let mut authentication_steps = (false, false, false, false);
                    for flow in &info.flows {
                        //Very crude, checks to see what auth steps we need to do next. Idealy we should handle all flows and figure out which one to use
                        for stage in &flow.stages {
                            match stage {
                                AuthType::ReCaptcha => authentication_steps.0 = true,
                                AuthType::EmailIdentity => authentication_steps.1 = true,
                                AuthType::RegistrationToken => authentication_steps.2 = true,
                                AuthType::Terms => authentication_steps.3 = true,
                                _ => {}
                            }
                        }

                        let new_state = RegisterState::ExtraAuthRequired {
                            recaptcha: authentication_steps.0,
                            shared_token: authentication_steps.1,
                            email: authentication_steps.2,
                            terms: authentication_steps.3,
                            session_token: info.session.clone(),
                        };
                        register_state.set(new_state);
                    }
                } else {
                    //TODO
                    //MOST will require extra auth, however here we'll need to send back the session token, device_id, user_id, and access_token
                }
                client.set(MainClient::Connected(tmp_cl));
            };
        });
    };

    match register_state.read().clone() {
        RegisterState::ExtraAuthRequired {
            recaptcha,
            shared_token,
            email,
            terms,
            session_token,
        } => {
            //More auth required but simply need to return the dummy auth.
            if !(recaptcha || shared_token || email || terms) {
                spawn(async move {
                    let mut dummy = Dummy::new();
                    dummy.session = session_token;

                    let mut register_request =
                        ruma::api::client::account::register::v3::Request::new();
                    register_request.password = Some(password.to_string());
                    register_request.username = Some(username.to_string());
                    register_request.refresh_token = true;
                    register_request.kind = RegistrationKind::User;
                    register_request.auth = Some(ruma::api::client::uiaa::AuthData::Dummy(dummy));

                    if let MainClient::Connected(client) = client.read().clone() {
                        let resp = client
                            .matrix_auth()
                            .register(register_request.clone())
                            .await;
                        println!("{:#?}", resp);
                    } else {
                        panic!("Client should be connected!")
                    }
                    register_state.set(RegisterState::RegisterComplete);
                });
            }
        }
        RegisterState::RegisterComplete => {
            println!("Register complete")
        }
        _ => {}
    }

    rsx! {
        rect {
            min_width: "30%",
            min_height: "30%",

            match register_state.read().clone(){
                RegisterState::EnteringData => {
                    rsx!{
                        label {"Enter username and password for registry"}
                        Input {
                            value: username.read().clone(),
                            onchange: move |e| {
                                username.set(e)
                            },
                        }
                        Input {
                            value: password.read().clone(),
                            onchange: move |e| {
                                password.set(e)
                            },
                            mode: InputMode::Hidden('*')
                        }
                        Button {
                            onclick: submit_on_click,
                            label {"Register"}
                        }
                    }

                },
                RegisterState::ExtraAuthRequired { recaptcha, shared_token, email, terms, session_token } => {
                    //TODO: Handle all auth types
                    if ! (recaptcha || shared_token || email || terms ){
                        rsx!{
                            Loader{}
                            label {"Attempting Registration"}
                        }
                    }
                    else {
                        rsx!{

                            label {"wowza"}
                        }
                    }
                },
                RegisterState::RegisterComplete => {
                    rsx!{label {"Registry complete"}}
                }
            }
        }
    }
}
