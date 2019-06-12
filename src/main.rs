mod rules;
mod search;
mod world;

use rules::parse::{parse_isotropic, parse_life};
use search::NewState::{Choose, FirstRandomThenDead, Random};
use search::{NewState, Search, Status, TraitSearch};
use std::time::Duration;
use world::State::{Alive, Dead};
use world::{Symmetry, World};
use yew::components::Select;
use yew::html;
use yew::html::ChangeData;
use yew::prelude::*;
use yew::services::{DialogService, IntervalService, Task};

struct Model {
    props: Props,
    view_freq: usize,
    status: Status,
    generation: isize,
    search: Box<dyn TraitSearch>,
    job: Job,
}

enum Msg {
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
struct Props {
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
            Status::Found => html! {
                <p>
                    { "Found a result. " }
                    <button onclick = |_| Msg::Start,> { "Next" } </button>
                </p>
            },
            Status::None => html! {
                <p>
                    { "No more result. " }
                    <button onclick = |_| Msg::Start,> { "Restart" } </button>
                </p>
            },
            Status::Searching => html! {
                <p>
                    { "Searching... " }
                    <button onclick = |_| Msg::Pause,> { "Pause" } </button>
                </p>
            },
            Status::Paused => html! {
                <p>
                    { "Paused. " }
                    <button onclick = |_| Msg::Start,> { "Start" } </button>
                </p>
            },
        };
        let set_generation = html! {
            <p>
                { "Showing generation " }
                <input
                    type = "number",
                    value = self.generation,
                    min = "0",
                    max = self.props.period - 1,
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetGeneration(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    },
                />
            </p>
        };
        let set_width = html! {
            <p>
                { "Width: " }
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
            </p>
        };
        let set_height = html! {
            <p>
                { "Height: " }
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
            </p>
        };
        let set_period = html! {
            <p>
                { "Period: " }
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
            </p>
        };
        let set_dx = html! {
            <p>
                { "dx: " }
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
            </p>
        };
        let set_dy = html! {
            <p>
                { "dy: " }
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
            </p>
        };
        let set_rule = html! {
            <p>
                { "Rule: " }
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
            </p>
        };
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
        let set_symmetry = html! {
            <p>
                { "Symmetry: " }
                <Select<Symmetry>:
                    selected = Some(self.props.symmetry),
                    options = symmetries,
                    onchange = Msg::SetSymmetry,
                />
            </p>
        };
        let set_order = html! {
            <p>
                { "Search Order: " }
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
            </p>
        };
        let set_new_state = html! {
            <p>
                { "New state for unknown cells: " }
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
                    <option value = "frtd",> { "Smart (?)" } </option>
                </select>
            </p>
        };

        html! {
            <div>
                <pre> { self.search.display_gen(self.generation) } </pre>
                { status }
                { set_generation }
                { set_rule }
                { set_width }
                { set_height }
                { set_period }
                { set_dx }
                { set_dy }
                { set_symmetry }
                { set_order }
                { set_new_state }
                <button onclick = |_| Msg::Reset,> { "Set World" } </button>
            </div>
        }
    }
}

fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
