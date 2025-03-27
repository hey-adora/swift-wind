use std::error::Error;

use freya::prelude::*;
use matrix_sdk::{HttpError, RumaApiError};
use ruma::api::{
    client::{account::register::RegistrationKind, uiaa::AuthData},
    error::FromHttpResponseError,
};
use tracing::{error, trace, warn};

use crate::{CLIENT, MatrixClientState};

use super::CommonUserAuthData;

pub enum HookAuthResult {
    NextStage,
    AuthFinished,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AdditionalAuthType {
    Login(CommonUserAuthData),
    Register(CommonUserAuthData),
}

//TODO: Form requests based on additional auth type. Or maybe create a callback???
pub fn use_matrix_additional_auth<F>(
    callback: F,
) -> (Signal<String>, impl FnMut(AuthData, AdditionalAuthType))
where
    //This callback should know if we're finished authenticating or if there is more to be done
    F: FnMut(HookAuthResult) + Clone + 'static,
{
    let mut error_string = use_signal(String::new);

    //Business logic here
    let run_auth = move |data: AuthData, auth_type: AdditionalAuthType| {
        trace!(
            "Got additional auth data: {:#?} for auth type :{:#?}",
            data, auth_type
        );
        let mut callback = callback.clone();
        spawn(async move {
            let res = match auth_type {
                AdditionalAuthType::Login(common_user_auth_data) => {
                    auth_login(common_user_auth_data).await
                }
                AdditionalAuthType::Register(common_user_auth_data) => {
                    auth_register(&data, common_user_auth_data).await
                }
            };
            match res {
                Ok(auth_res) => callback(auth_res),
                Err(e) => {
                    *error_string.write() = e.to_string();
                }
            }
        });
    };
    (error_string, run_auth)
}

async fn auth_register(
    finished_auth_data: &AuthData,
    common_user_data: CommonUserAuthData,
) -> Result<HookAuthResult, Box<dyn Error>> {
    let MatrixClientState::Connected(client) = CLIENT() else {
        warn!("trying to authenticate before connected");
        panic!("Use thiserror to return a state mismatch error");
    };

    let mut register_request = ruma::api::client::account::register::v3::Request::new();
    register_request.password = Some(common_user_data.password.clone());
    register_request.username = Some(common_user_data.username.clone());
    register_request.refresh_token = true;
    register_request.kind = RegistrationKind::User;
    register_request.auth = Some(finished_auth_data.clone());

    trace!("Sending additional auth register request");
    let resp = client.matrix_auth().register(register_request).await;
    if let Err(matrix_sdk::Error::Http(HttpError::Api(FromHttpResponseError::Server(
        RumaApiError::Uiaa(info),
    )))) = resp
    {
        if let Some(auth_error) = info.auth_error {
            error!("Failed to authenticate: {:#?}", auth_error);
            panic!(
                "Use thiserror to return authentication error: {:#?}",
                auth_error
            );
        }
        return Ok(HookAuthResult::NextStage);
    } else if resp.is_ok() {
        trace!("Additional authentication registration completed");
        return Ok(HookAuthResult::AuthFinished);
    } else if let Err(err) = resp {
        error!("Additional Authentication got unexpected api error: {err}");
        return Err(Box::new(err));
    }
    unreachable!()
}

async fn auth_login(
    common_user_data: CommonUserAuthData,
) -> Result<HookAuthResult, Box<dyn Error>> {
    let MatrixClientState::Connected(client) = CLIENT() else {
        warn!("trying to authenticate before connected");
        panic!("Use thiserror to return a state mismatch error");
    };

    trace!("Sending additional auth login request");
    let resp = client
        .matrix_auth()
        .login_username(common_user_data.username, &common_user_data.password)
        .request_refresh_token()
        .await;

    if let Err(matrix_sdk::Error::Http(HttpError::Api(FromHttpResponseError::Server(
        RumaApiError::Uiaa(info),
    )))) = resp
    {
        if let Some(auth_error) = info.auth_error {
            error!("Failed to authenticate: {:#?}", auth_error);
            panic!(
                "Use thiserror to return authentication error: {:#?}",
                auth_error
            );
        }
        return Ok(HookAuthResult::NextStage);
    } else if resp.is_ok() {
        trace!("Additional authentication login completed");
        return Ok(HookAuthResult::AuthFinished);
    } else if let Err(err) = resp {
        error!("Additional Authentication got unexpected api error: {err}");
        return Err(Box::new(err));
    }

    unreachable!()
}
