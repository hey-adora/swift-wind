use freya::prelude::*;

#[component]
pub fn FormField(
    #[props(default)] name: ReadOnlySignal<String>,
    #[props(default)] value: ReadOnlySignal<String>,
    #[props(default)] errors: ReadOnlySignal<String>,
    #[props(default)] placeholder: ReadOnlySignal<String>,
    #[props(default)] hidden: ReadOnlySignal<bool>,
    onchange: Option<Callback<String>>,
) -> Element {
    let name = use_memo(move || {
        let name = name();
        if name.is_empty() {
            return String::from("Field Name");
        }
        name
    });

    rsx!(
        rect {
            content: "flex",
            direction: "vertical",
            spacing: "5",

            label {
                font_size: "16",
                color: "#454545",

                "{name()}"
            }

            rect {

                Input {
                    width: "100%",
                    value: value(),
                    mode: if hidden() { InputMode::Hidden('*') } else { InputMode::Shown },
                    placeholder: placeholder(),
                    onchange: move |txt| {
                        let Some(callback) = onchange else {
                            return;
                        };

                        callback(txt);
                    }
                }
                label {
                     "{errors()}"
                 }
            }
        }
    )
}
