use stdweb::web::{self, event::IEvent};
use yew::{
    events::DoubleClickEvent, html, Component, ComponentLink, Html, NodeRef, Properties,
    ShouldRender,
};

pub struct World {
    link: ComponentLink<Self>,
    world: String,
    node_ref: NodeRef,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub world: String,
}

pub enum Msg {
    Select,
}

impl Component for World {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        World {
            link,
            world: props.world,
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Select => {
                if let Some(node) = self.node_ref.get() {
                    if let Some(selection) = web::window().get_selection() {
                        selection.select_all_children(&node);
                    }
                }
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.world != props.world && {
            self.world = props.world;
            true
        }
    }

    fn view(&self) -> Html {
        let ondblclick = self.link.callback(|e: DoubleClickEvent| {
            e.prevent_default();
            Msg::Select
        });
        html! {
            <pre id="world"
                ref=self.node_ref.clone()
                ondblclick=ondblclick>
                { &self.world }
            </pre>
        }
    }
}
