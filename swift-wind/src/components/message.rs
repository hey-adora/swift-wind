use std::{rc::Rc, sync::Arc};

use freya::prelude::*;
use matrix_sdk::reqwest::Url;
use ruma::{
    UserId,
    events::room::message::{LimitType, OriginalSyncRoomMessageEvent, ServerNoticeType},
};
use tracing::{info, warn};

use crate::{CLIENT, MatrixClientState};

#[component]
pub fn room_message(evt: OriginalSyncRoomMessageEvent) -> Element {
    let user_id = evt.sender.clone();

    //TODO: Replace this with a caching system so we dont have to hammer the server, can also act as a way to
    //provide fallback PFPs if we cant get em. Can also act as a way to transparently modify display names if two are unique
    let fetch_user_data = use_resource(move || {
        let value = user_id.clone();
        async move {
            let MatrixClientState::Connected(client) = CLIENT() else {
                warn!("trying to read messages before connected");
                panic!("Tried to read message before connected");
            };

            client
                .account()
                .fetch_user_profile_of(&value)
                .await
                .unwrap()
        }
    });

    //Massive match statement that selects what component to use based on the message,
    //makes it easy to compartmentalize every message type into their own sub-component
    let message_contents = match evt.content.msgtype {
        ruma::events::room::message::MessageType::Audio(audio_message_event_content) => {
            info!(
                "Got audio message content: {:#?}",
                audio_message_event_content
            );
            rsx! {label { "Missing audio implementation" }}
        }
        ruma::events::room::message::MessageType::Emote(emote_message_event_content) => {
            if let Some(format) = emote_message_event_content.formatted {
                rsx!(emote_component {
                    body: format.body,
                    formatted: true
                })
            } else {
                rsx!(emote_component {
                    body: emote_message_event_content.body,
                    formatted: false
                })
            }
        }
        ruma::events::room::message::MessageType::File(file_message_event_content) => {
            info!(
                "Got file message content: {:#?}",
                file_message_event_content
            );
            rsx! {label { "Missing file message implementation" }}
        }
        ruma::events::room::message::MessageType::Image(image_message_event_content) => {
            info!(
                "Got image message content: {:#?}",
                image_message_event_content
            );
            rsx! {label { "Missing image message implementation" }}
        }
        ruma::events::room::message::MessageType::Location(location_message_event_content) => {
            info!(
                "Got location message content: {:#?}",
                location_message_event_content
            );
            rsx! {label { "Missing location message implementation" }}
        }
        ruma::events::room::message::MessageType::Notice(notice_message_event_content) => {
            if let Some(format) = notice_message_event_content.formatted {
                rsx!(notice_component {
                    body: format.body,
                    formatted: true
                })
            } else {
                rsx!(notice_component {
                    body: notice_message_event_content.body,
                    formatted: false
                })
            }
        }
        ruma::events::room::message::MessageType::ServerNotice(
            server_notice_message_event_content,
        ) => {
            rsx!(server_notice_component {
                body: server_notice_message_event_content.body,
                s_notice_type: server_notice_message_event_content.server_notice_type,
                admin_contact: server_notice_message_event_content.admin_contact,
                limit_type: server_notice_message_event_content.limit_type
            })
        }
        ruma::events::room::message::MessageType::Text(text_message_event_content) => {
            if let Some(format) = text_message_event_content.formatted {
                rsx!(text_component {
                    body: format.body,
                    formatted: true
                })
            } else {
                rsx!(text_component {
                    body: text_message_event_content.body,
                    formatted: false
                })
            }
        }
        ruma::events::room::message::MessageType::Video(video_message_event_content) => {
            info!(
                "Got video message content: {:#?}",
                video_message_event_content
            );
            rsx! {label { "Missing video message implementation" }}
        }
        ruma::events::room::message::MessageType::VerificationRequest(
            key_verification_request_event_content,
        ) => {
            info!(
                "Got video message content: {:#?}",
                key_verification_request_event_content
            );
            rsx! {label { "Missing key verification message implementation" }}
        }
        _ => todo!(),
    };

    let user_data = fetch_user_data.cloned().unwrap_or_default();
    let name = user_data
        .displayname
        .unwrap_or_else(|| format!("User ID: {}", evt.sender));
    //TODO: Add "controls" like reply, delete, react, copy text, etc
    rsx! {
        rect {
            direction: "vertical" ,
            //Profile pic next to display name
            rect {
                direction: "horizontal",
                if let Some(avatar_url) = user_data.avatar_url{
                    NetworkImage{
                        url: avatar_url.as_str().parse::<Url>().unwrap(),
                    }
                }
                label { "UserID: {name}" }
            }
            //Message contents
            {message_contents}
        }
    }
}
#[component]
fn emote_component(body: String, formatted: bool) -> Element {
    //TODO: Process formatted message differently
    if formatted {
        warn!("Got formatted emote message, cant parse it currently");
    }
    rsx! {
        rect {
            height: "fill",
            width: "fill",
            direction: "vertical",
            label {
                color: "grey",
                font_style:"italic",
                "{body}"
            }
        }
    }
}

#[component]
fn notice_component(body: String, formatted: bool) -> Element {
    if formatted {
        warn!("Got formatted notice, cant parse it currently");
    }
    rsx! {
        rect {
            height: "fill",
            width: "fill",
            direction: "vertical",
            label {
                color: "red",
                font_size: "10",
                "Notice:"
            }
            label { "{body}" }
        }
    }
}

#[component]
fn server_notice_component(
    body: String,
    s_notice_type: ServerNoticeType,
    admin_contact: Option<String>,
    limit_type: Option<LimitType>,
) -> Element {
    //TODO: Use limit_type, or maybe not?
    rsx! {
        rect {
            height: "fill",
            width: "fill",
            direction: "vertical",
            label {
                color: "red",
                font_size: "10",
                "Server notice: {s_notice_type}"
            }
            label { "{body}" }
            if let Some(admin_contact) = admin_contact {
                label {
                    font_size: "10",
                    "Contact {admin_contact} for more information"
                }
            }

        }
    }
}

#[component]
fn text_component(body: String, formatted: bool) -> Element {
    //TODO: Process formatted message differently
    if formatted {
        warn!("Got formatted text message, cant parse it currently");
    }
    rsx! {
        rect {
            height: "fill",
            width: "fill",
            direction: "vertical",
            label { "{body}" }
        }
    }
}
