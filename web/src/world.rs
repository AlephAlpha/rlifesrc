use stdweb::web::{self, event::IEvent};
use yew::{html, Component, ComponentLink, Html, NodeRef, Properties, ShouldRender};

pub struct World {
    world: String,
    node_ref: NodeRef,
}

#[derive(Properties)]
pub struct Props {
    #[props(required)]
    pub world: String,
}

pub enum Msg {
    Select,
}

impl Component for World {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        World {
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

    fn view(&self) -> Html<Self> {
        html! {
            <pre id="world"
                ref=self.node_ref.clone()
                ondoubleclick=|e| {
                    e.prevent_default();
                    Msg::Select
                }>
                { &self.world }
            </pre>
        }
    }
}
