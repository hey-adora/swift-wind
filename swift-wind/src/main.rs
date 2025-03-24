use freya::prelude::*;

fn main() {
    launch(app);
}

fn app() -> Element {
    rsx!(
        rect {
            height: "100%",
            width: "100%",
            main_align: "center",
            cross_align: "center",
            background: "rgb(255, 0, 0)",
            color: "white",
            shadow: "0 4 20 5 rgb(0, 0, 0, 80)",
            label {
                font_size: "75",
                font_weight: "bold",
                "hey >:3"
            }
        }
    )
}


