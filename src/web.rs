use crate::search::NewState::{Choose, FirstRandomThenDead, Random};
use crate::search::{NewState, Status};
use crate::worker::{Props, Request, Response, Worker};
use crate::world::State::{Alive, Dead};
use crate::world::Symmetry;
use std::time::Duration;
use yew::components::Select;
use yew::html;
use yew::html::ChangeData;
use yew::services::{DialogService, IntervalService, Task};
use yew::*;

// 这部份的很多写法是照抄 yew 自带的范例
// https://github.com/DenisKolodin/yew

pub struct Model {
    props: Props,
    status: Status,
    generation: isize,
    world: Option<String>,
    period: Option<isize>,
    worker: Box<Bridge<Worker>>,
    job: Job,
}

pub enum Msg {
    Tick,
    Start,
    Pause,
    SetGen(isize),
    SetWidth(isize),
    SetHeight(isize),
    SetPeriod(isize),
    SetDx(isize),
    SetDy(isize),
    SetRule(String),
    SetSymmetry(Symmetry),
    SetOrder(Option<bool>),
    SetNewState(NewState),
    SetMax(Option<u32>),
    Reset,
    DataReceived(Response),
    None,
}

struct Job {
    interval: IntervalService,
    callback: Callback<()>,
    task: Option<Box<Task>>,
}

impl Job {
    fn new(link: &mut ComponentLink<Model>) -> Self {
        let interval = IntervalService::new();
        let callback = link.send_back(|_| Msg::Tick);
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

impl Component for Model {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let status = Status::Paused;
        let job = Job::new(&mut link);
        let callback = link.send_back(Msg::DataReceived);
        let worker = Worker::bridge(callback);

        Model {
            props,
            status,
            generation: 0,
            world: None,
            period: None,
            worker,
            job,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                self.worker.send(Request::DisplayGen(self.generation));
                return false;
            }
            Msg::Start => {
                self.worker.send(Request::Start);
            }
            Msg::Pause => {
                self.worker.send(Request::Pause);
            }
            Msg::SetGen(gen) => {
                self.generation = gen;
                self.worker.send(Request::DisplayGen(self.generation));
            }
            Msg::SetWidth(width) => {
                self.props.width = width;
            }
            Msg::SetHeight(height) => {
                self.props.height = height;
            }
            Msg::SetPeriod(period) => {
                self.props.period = period;
            }
            Msg::SetDx(dx) => {
                self.props.dx = dx;
            }
            Msg::SetDy(dy) => {
                self.props.dy = dy;
            }
            Msg::SetRule(rule_string) => {
                self.props.rule_string = rule_string;
            }
            Msg::SetSymmetry(symmetry) => {
                self.props.symmetry = symmetry;
            }
            Msg::SetOrder(column_first) => {
                self.props.column_first = column_first;
            }
            Msg::SetNewState(new_state) => {
                self.props.new_state = new_state;
            }
            Msg::SetMax(max_cell_count) => {
                self.props.max_cell_count = max_cell_count;
            }
            Msg::Reset => {
                self.generation = 0;
                self.period = Some(self.props.period);
                self.worker.send(Request::SetWorld(self.props.clone()));
            }
            Msg::DataReceived(response) => match response {
                Response::UpdateWorld(world) => {
                    self.world = Some(world);
                }
                Response::UpdateStatus(status) => {
                    let old_status = self.status;
                    if self.status != status {
                        match (old_status, status) {
                            (Status::Searching, _) => self.job.stop(),
                            (_, Status::Searching) => self.job.start(),
                            _ => (),
                        }
                        self.status = status;
                    }
                }
                Response::InvalidRule => {
                    let mut dialog = DialogService::new();
                    dialog.alert("Invalid rule!");
                    return false;
                }
            },
            Msg::None => return false,
        }
        true
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
                    <span id = "generation">
                        { &format!("Showing generation {}: ", self.generation) }
                    </span>
                    <input
                        id = "set_generation",
                        type = "range",
                        value = self.generation,
                        min = "0",
                        max = self.period.unwrap_or(1) - 1,
                        oninput = |e| Msg::SetGen(e.value.parse().unwrap()),
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
                    <option value = "a"> { "Automatic" } </option>
                    <option value = "c"> { "Column first" } </option>
                    <option value = "r"> { "Row first" } </option>
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
                    <option value = "d"> { "Dead" } </option>
                    <option value = "a"> { "Alive" } </option>
                    <option value = "r"> { "Random" } </option>
                    <option value = "frtd"> { "Smart" } </option>
                </select>
            },
        );
        let set_max = view_setting(
            "Max cells: ",
            "Maximal number of living cells in the first generation\n\
             If this value is set to 0, it means there is no limitation.\n",
            html! {
                <input
                    type = "number",
                    value = match self.props.max_cell_count {
                        None => 0,
                        Some(i) => i,
                    },
                    min = "0",
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            let max_cell_count = v.parse().unwrap();
                            let max_cell_count = match max_cell_count {
                                0 => None,
                                i => Some(i),
                            };
                            Msg::SetMax(max_cell_count)
                        } else {
                            Msg::None
                        }
                    },
                />
            },
        );

        let settings = html! {
            <div id = "settings">
                <div id = "buttons">
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
                    { set_max }
                    { set_symmetry }
                    { set_order }
                    { set_new_state }
                </div>
            </div>
        };

        html! {
            <div id = "rlifesrc">
                <h1>
                    <a href = "https://github.com/AlephAlpha/rlifesrc/">
                        { "rlifesrc" }
                    </a>
                    <span id = "subheading">
                        { "A Game of Life pattern searcher written in Rust. " }
                    </span>
                </h1>
                <pre>
                    { self.world.clone().unwrap_or_default() }
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
        <div title = description,  class = "setting">
            <label>
                <span class = "label">
                    { label }
                </span>
                { setting }
            </label>
        </div>
    }
}
