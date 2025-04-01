use dioxus_router::prelude::navigator;
use freya::prelude::*;
use ruma::RoomId;
use tracing::{info, warn};

use crate::{CLIENT, CURRENT_SPACE, MatrixClientState};

#[derive(Clone)]
struct RoomData {
    name: String,
    id: String,
}

/// A light modification of the space selection button. Removes a lot of the fancy stuff like avatar and tooltip. Effectively a button with a name
///
#[component]
pub fn RoomSelectionButton(room: String, selected: bool) -> Element {
    //TODO: Use selected prop to place a visible marker that denotes that the room is already selected
    //TODO: Add a rightclick menu to show other parented spaces

    let room_id = RoomId::parse(&room).unwrap();
    //Like the message component, try to gracefully return instead of just panicing, again with a caching layer
    let fetch_room_data = use_resource(move || {
        let value = room_id.clone();
        async move {
            let MatrixClientState::Connected(client) = CLIENT() else {
                warn!("trying to read space data before connected");
                panic!("Tried to read space before connected");
            };

            let room = client.get_room(&value).unwrap();

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

            RoomData { name, id }
        }
    });

    let room_data = fetch_room_data.unwrap();
    let navigator = navigator();

    let clicked = move |_| {
        navigator.push(format!(
            "main_interface/{}/{}",
            CURRENT_SPACE.as_ref().unwrap(),
            room_data.id
        ));
    };

    rsx! {
        rect {
            height: "10%",
            width: "fill",
            Button {
                onclick: clicked,
                label { "{room_data.name}" },
            }
        }
    }
}
