use crate::rules::parse::{parse_isotropic, parse_life};
use crate::search::NewState::{Choose, FirstRandomThenDead, Random};
use crate::search::{NewState, Search, Status, TraitSearch};
use crate::world::State::{Alive, Dead};
use crate::world::{Symmetry, World};
use std::time::Duration;
use yew::components::Select;
use yew::html;
use yew::html::ChangeData;
use yew::prelude::*;
use yew::services::{DialogService, IntervalService, Task};

// 这部份的很多写法是照抄 yew 自带的范例
// https://github.com/DenisKolodin/yew

pub struct Model {
    props: Props,
    view_freq: usize,
    status: Status,
    generation: isize,
    search: Box<dyn TraitSearch>,
    job: Job,
}

pub enum Msg {
    Step,
    Start,
    Pause,
    SetGeneration(isize),
    SetWidth(isize),
    SetHeight(isize),
    SetPeriod(isize),
    SetDx(isize),
    SetDy(isize),
    SetRule(String),
    SetSymmetry(Symmetry),
    SetOrder(Option<bool>),
    SetNewState(NewState),
    Reset,
    None,
}

#[derive(Clone, PartialEq)]
pub struct Props {
    width: isize,
    height: isize,
    period: isize,
    dx: isize,
    dy: isize,
    symmetry: Symmetry,
    column_first: Option<bool>,
    new_state: NewState,
    rule_string: String,
}

struct Job {
    interval: IntervalService,
    callback: Callback<()>,
    task: Option<Box<Task>>,
}

impl Job {
    fn new(link: &mut ComponentLink<Model>) -> Self {
        let interval = IntervalService::new();
        let callback = link.send_back(|_| Msg::Step);
        let task = None;
        Job {
            interval,
            callback,
            task,
        }
    }

    fn start(&mut self) {
        let handle = self
            .interval
            .spawn(Duration::from_millis(1000 / 60), self.callback.clone());
        self.task = Some(Box::new(handle));
    }

    fn stop(&mut self) {
        if let Some(mut task) = self.task.take() {
            task.cancel();
        }
    }
}

impl Default for Props {
    fn default() -> Self {
        Props {
            width: 7,
            height: 7,
            period: 3,
            dx: 0,
            dy: 0,
            symmetry: Symmetry::C1,
            column_first: None,
            new_state: Choose(Dead),
            rule_string: String::from("B3/S23"),
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let rule = parse_life(&props.rule_string).unwrap();
        let world = World::new(
            (props.width, props.height, props.period),
            props.dx,
            props.dy,
            props.symmetry,
            rule,
            props.column_first,
        );
        let search = Box::new(Search::new(world, props.new_state));

        let view_freq = 10000;
        let status = Status::Paused;
        let generation = 0;
        let job = Job::new(&mut link);

        Model {
            props,
            view_freq,
            status,
            generation,
            search,
            job,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Step => {
                if let Status::Searching = self.status {
                    self.status = self.search.search(Some(self.view_freq));
                } else {
                    self.job.stop();
                }
            }
            Msg::Start => {
                self.job.start();
                self.status = Status::Searching;
            }
            Msg::Pause => {
                self.job.stop();
                self.status = Status::Paused;
            }
            Msg::SetGeneration(generation) => {
                self.generation = generation;
            }
            Msg::SetWidth(width) => {
                self.props = Props {
                    width,
                    ..self.props.clone()
                };
            }
            Msg::SetHeight(height) => {
                self.props = Props {
                    height,
                    ..self.props.clone()
                };
            }
            Msg::SetPeriod(period) => {
                self.props = Props {
                    period,
                    ..self.props.clone()
                };
            }
            Msg::SetDx(dx) => {
                self.props = Props {
                    dx,
                    ..self.props.clone()
                };
            }
            Msg::SetDy(dy) => {
                self.props = Props {
                    dy,
                    ..self.props.clone()
                };
            }
            Msg::SetRule(rule_string) => {
                self.props = Props {
                    rule_string,
                    ..self.props.clone()
                };
            }
            Msg::SetSymmetry(symmetry) => {
                self.props = Props {
                    symmetry,
                    ..self.props.clone()
                };
            }
            Msg::SetOrder(column_first) => {
                self.props = Props {
                    column_first,
                    ..self.props.clone()
                };
            }
            Msg::SetNewState(new_state) => {
                self.props = Props {
                    new_state,
                    ..self.props.clone()
                };
            }
            Msg::Reset => {
                self.change(self.props.clone());
            }
            Msg::None => return false,
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.job.stop();
        self.status = Status::Paused;
        if let Ok(rule) = parse_life(&props.rule_string) {
            self.props = props.clone();
            let world = World::new(
                (props.width, props.height, props.period),
                props.dx,
                props.dy,
                props.symmetry,
                rule,
                props.column_first,
            );
            self.search = Box::new(Search::new(world, props.new_state));
            true
        } else if let Ok(rule) = parse_isotropic(&props.rule_string) {
            self.props = props.clone();
            let world = World::new(
                (props.width, props.height, props.period),
                props.dx,
                props.dy,
                props.symmetry,
                rule,
                props.column_first,
            );
            self.search = Box::new(Search::new(world, props.new_state));
            true
        } else {
            let mut dialog = DialogService::new();
            dialog.alert("Invalid rule!");
            false
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let status = match self.status {
            Status::Found => "Found a result.",
            Status::None => "No more result.",
            Status::Searching => "Searching...",
            Status::Paused => "Paused.",
        };
        let set_generation = html! {
            <div>
                <label>
                    <span id = "generation",>
                        { &format!("Showing generation {}: ", self.generation) }
                    </span>
                    <input
                        id = "set_generation",
                        type = "range",
                        value = self.generation,
                        min = "0",
                        max = self.search.period() - 1,
                        oninput = |e| Msg::SetGeneration(e.value.parse().unwrap()),
                    />
                </label>
            </div>
        };
        let set_width = view_setting(
            "Width: ",
            "Width of the pattern",
            html! {
                <input
                    type = "number",
                    value = self.props.width,
                    min = "1",
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetWidth(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );
        let set_height = view_setting(
            "Height: ",
            "Height of the pattern",
            html! {
                <input
                    type = "number",
                    value = self.props.height,
                    min = "1",
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetHeight(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );
        let set_period = view_setting(
            "Period: ",
            "Period of the pattern",
            html! {
                <input
                    type = "number",
                    value = self.props.period,
                    min = "1",
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetPeriod(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );
        let set_dx = view_setting(
            "dx: ",
            "Horizontal translation",
            html! {
                <input
                    type = "number",
                    value = self.props.dx,
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetDx(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );
        let set_dy = view_setting(
            "dy: ",
            "Vertical translation",
            html! {
                <input
                    type = "number",
                    value = self.props.dy,
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetDy(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );
        let set_rule = view_setting(
            "Rule: ",
            "Rule of the cellular automaton\n\
             Supports Life-like and isotropic non-totalistic rules.",
            html! {
                <input
                    type = "text",
                    value = self.props.rule_string.clone(),
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetRule(v)
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );
        let symmetries = vec![
            Symmetry::C1,
            Symmetry::C2,
            Symmetry::C4,
            Symmetry::D2Row,
            Symmetry::D2Column,
            Symmetry::D2Diag,
            Symmetry::D2Antidiag,
            Symmetry::D4Ortho,
            Symmetry::D4Diag,
            Symmetry::D8,
        ];
        let set_symmetry = view_setting(
            "Symmetry: ",
            "Symmetry of the pattern",
            html! {
                <Select<Symmetry>:
                    selected = Some(self.props.symmetry),
                    options = symmetries,
                    onchange = Msg::SetSymmetry,
                />
            },
        );
        let set_order = view_setting(
            "Order: ",
            "Search order\n\
             Row first or column first",
            html! {
                <select
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "a" => Msg::SetOrder(None),
                                "c" => Msg::SetOrder(Some(true)),
                                "r" => Msg::SetOrder(Some(false)),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    },
                >
                    <option value = "a",> { "Automatic" } </option>
                    <option value = "c",> { "Column first" } </option>
                    <option value = "r",> { "Row first" } </option>
                </select>
            },
        );
        let set_new_state = view_setting(
            "New state: ",
            "How to choose a state for unknown cells\n\
             \"Smart\" means choosing a random state for cells in the first row/column, \
             and dead for other cells.\n",
            html! {
                <select
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "d" => Msg::SetNewState(Choose(Dead)),
                                "a" => Msg::SetNewState(Choose(Alive)),
                                "r" => Msg::SetNewState(Random),
                                "frtd" => Msg::SetNewState(FirstRandomThenDead(0)),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    },
                >
                    <option value = "d",> { "Dead" } </option>
                    <option value = "a",> { "Alive" } </option>
                    <option value = "r",> { "Random" } </option>
                    <option value = "frtd",> { "Smart" } </option>
                </select>
            },
        );

        let settings = html! {
            <div id = "settings",>
                <div id = "buttons",>
                    <button
                        onclick = |_| Msg::Start,
                        disabled = self.status == Status::Searching,
                    >
                        { "Start" }
                    </button>
                    <button
                        onclick = |_| Msg::Pause,
                        disabled = self.status != Status::Searching,
                    >
                        { "Pause" }
                    </button>
                    <button
                        onclick = |_| Msg::Reset,
                        title = "Reset the world",
                    >
                        { "Set World" }
                    </button>
                    { set_rule }
                    { set_width }
                    { set_height }
                    { set_period }
                    { set_dx }
                    { set_dy }
                    { set_symmetry }
                    { set_order }
                    { set_new_state }
                </div>
            </div>
        };

        html! {
            <div id = "rlifesrc",>
                <h1>
                    <a href = "https://github.com/AlephAlpha/rlifesrc/tree/web",>
                        { "rlifesrc" }
                    </a>
                    <span id = "subheading",>
                        { "A Game of Life pattern searcher written in Rust. " }
                    </span>
                </h1>
                <pre>
                    { self.search.display_gen(self.generation) }
                </pre>
                <div>
                    { status }
                </div>
                { set_generation }
                { settings }
            </div>
        }
    }
}

fn view_setting(label: &str, description: &str, setting: Html<Model>) -> Html<Model> {
    html! {
        <div title = description,  class = "setting",>
            <label>
                <span class = "label",>
                    { label }
                </span>
                { setting }
            </label>
        </div>
    }
}
