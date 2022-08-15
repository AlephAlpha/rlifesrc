use yew::{events::MouseEvent, html, Component, Context, Html, NodeRef, Properties};

pub struct World {
    node_ref: NodeRef,
}

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props {
    pub world: String,
}

pub enum Msg {
    Select,
}

impl Component for World {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Select => {
                if let Some(node) = self.node_ref.get() {
                    if let Ok(Some(selection)) = web_sys::window().unwrap().get_selection() {
                        selection.select_all_children(&node).unwrap();
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let ondblclick = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Select
        });
        html! {
            <pre id="world"
                ref={self.node_ref.clone()}
                ondblclick={ondblclick}>
                { &ctx.props().world }
            </pre>
        }
    }
}
