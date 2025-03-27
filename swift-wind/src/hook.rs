pub mod connect;
pub mod login;
pub mod register;
pub mod submit_additional_auth;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommonUserAuthData {
    pub username: String,
    pub password: String,
    pub session_id: Option<String>,
}
