use freya::prelude::*;
use ruma::{
    OwnedDeviceId, OwnedUserId,
    api::client::uiaa::{AuthData, AuthType},
};
use tracing::trace;

pub enum HookAuthResult {
    NextStage,
    AuthFinished {
        access_token: Option<String>,
        refresh_token: Option<String>,
        device_id: Option<OwnedDeviceId>,
        user_id: OwnedUserId,
    },
}

pub enum AdditionalAuthType {
    Login,
    Register,
}

//TODO: Form requests based on additional auth type. Or maybe create a callback???
pub fn use_matrix_additional_auth<F>(
    callback: F,
    auth_type: AdditionalAuthType,
) -> (Signal<String>, impl FnMut(AuthData))
where
    //This callback should know if we're finished authenticating or if there is more to be done
    F: FnMut(HookAuthResult) + Clone + 'static,
{
    let mut error_string = use_signal(String::new);

    //Business logic here
    let run_auth = move |data: AuthData| {
        trace!("Additional auth function got data: {:#?}", data);
        spawn(async move {});
    };
    (error_string, run_auth)
}
