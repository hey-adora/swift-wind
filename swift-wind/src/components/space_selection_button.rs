use dioxus_router::prelude::navigator;
use freya::prelude::*;
use matrix_sdk::reqwest::Url;
use ruma::RoomId;
use tracing::warn;

use crate::{CLIENT, MatrixClientState};

#[derive(Clone)]
struct RoomData {
    avatar_url: Option<Url>,
    name: String,
    id: String,
}

/// Allows identification and selection of a parent space/room, this will instruct the router to load the selected room when clicked
#[component]
pub fn SpaceSelectionButton(space: String, selected: bool) -> Element {
    //TODO: Use selected prop to place a visible marker that denotes that the space is already selected

    let space_id = RoomId::parse(&space).unwrap();
    //Like the message component, try to gracefully return instead of just panicing, again with a caching layer
    let fetch_space_data = use_resource(move || {
        let value = space_id.clone();
        async move {
            let MatrixClientState::Connected(client) = CLIENT() else {
                warn!("trying to read space data before connected");
                panic!("Tried to read space before connected");
            };

            let room = client.get_room(&value).unwrap();
            let avatar_url = room
                .avatar_url()
                .map(|url| url.as_str().parse::<Url>().unwrap());

            let name = {
                if let Ok(display_name) = room.display_name().await {
                    match display_name {
                        matrix_sdk::RoomDisplayName::Named(str) => str,
                        matrix_sdk::RoomDisplayName::Aliased(str) => str,
                        matrix_sdk::RoomDisplayName::Calculated(str) => str,
                        matrix_sdk::RoomDisplayName::EmptyWas(str) => str,
                        matrix_sdk::RoomDisplayName::Empty => room.room_id().to_string(),
                    }
                } else {
                    room.room_id().to_string()
                }
            };
            let id = room.room_id().to_string();

            RoomData {
                avatar_url,
                name,
                id,
            }
        }
    });

    let room_data = fetch_space_data.unwrap();
    let navigator = navigator();

    let clicked = move |_| {
        navigator.push(format!("main_interface/{}", room_data.id));
    };

    rsx! {
        rect {
            height: "10%",
            width: "fill",
            TooltipContainer {
                tooltip: rsx!(
                    Tooltip{
                        text: &room_data.name
                    }
                ),
                Button {
                    onclick: clicked,
                    if let Some(avatar_url) = room_data.avatar_url{
                        NetworkImage{
                            url: avatar_url.as_str().parse::<Url>().unwrap(),
                        }
                    }
                    else{
                        label { "{room_data.name}" }
                    }
                }
            }
        }
    }
}
