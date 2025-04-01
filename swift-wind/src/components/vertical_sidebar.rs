use freya::prelude::*;

///A very thin wrapper over Scrollview that stacks things vertically with no scrollbar
#[component]
pub fn VerticalSideBar(children: Element) -> Element {
    rsx!(ScrollView{
        width: "fill",
        height: "fill",
        direction: "vertical",
        show_scrollbar: false,
        {children}
    })
}
