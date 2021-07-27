use once_cell::sync::Lazy;
use pulldown_cmark::{html::push_html, Parser};
use std::include_str;
use web_sys::Node;
use yew::{virtual_dom::VNode, Component, ComponentLink, Html, ShouldRender};

const HELP_TEXT: &str = include_str!("help.md");

static HELP_HTML: Lazy<String> = Lazy::new(|| {
    let mut html_output = String::new();
    push_html(&mut html_output, Parser::new(HELP_TEXT));
    html_output
});

pub struct Help;

impl Component for Help {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Help
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let html = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        html.set_inner_html(&HELP_HTML);
        html.set_attribute("class", "mui-container").unwrap();
        VNode::VRef(Node::from(html))
    }
}
