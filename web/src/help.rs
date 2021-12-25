use once_cell::sync::Lazy;
use pulldown_cmark::{html::push_html, Parser};
use std::include_str;
use web_sys::Node;
use yew::{virtual_dom::VNode, Component, Context, Html};

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

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
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
