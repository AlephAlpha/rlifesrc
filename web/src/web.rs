use crate::worker::{Props, Request, Response, Worker};
use rlifesrc_lib::NewState::{Choose, Random, Smart};
use rlifesrc_lib::State::{Alive, Dead};
use rlifesrc_lib::{NewState, Status, Symmetry};
use std::time::Duration;
use yew::html;
use yew::html::ChangeData;
use yew::services::{DialogService, IntervalService, Task};
use yew::*;

// 这部份的很多写法是照抄 yew 自带的范例
// https://github.com/DenisKolodin/yew

pub struct Model {
    props: Props,
    status: Status,
    gen: isize,
    cells: u32,
    world: Option<String>,
    period: Option<isize>,
    worker: Box<dyn Bridge<Worker>>,
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
    SetSym(Symmetry),
    SetOrder(Option<bool>),
    SetChoose(NewState),
    SetMax(Option<u32>),
    Reset,
    DataReceived(Response),
    None,
}

struct Job {
    interval: IntervalService,
    callback: Callback<()>,
    task: Option<Box<dyn Task>>,
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
            gen: 0,
            cells: 0,
            world: None,
            period: None,
            worker,
            job,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                self.worker.send(Request::DisplayGen(self.gen));
                return false;
            }
            Msg::Start => {
                self.worker.send(Request::Start);
            }
            Msg::Pause => {
                self.worker.send(Request::Pause);
            }
            Msg::SetGen(gen) => {
                self.gen = gen;
                self.worker.send(Request::DisplayGen(self.gen));
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
            Msg::SetSym(symmetry) => {
                self.props.symmetry = symmetry;
            }
            Msg::SetOrder(column_first) => {
                self.props.column_first = column_first;
            }
            Msg::SetChoose(new_state) => {
                self.props.new_state = new_state;
            }
            Msg::SetMax(max_cell_count) => {
                self.props.max_cell_count = max_cell_count;
            }
            Msg::Reset => {
                self.gen = 0;
                self.period = Some(self.props.period);
                self.worker.send(Request::SetWorld(self.props.clone()));
            }
            Msg::DataReceived(response) => match response {
                Response::UpdateWorld((world, cells)) => {
                    self.world = Some(world);
                    self.cells = cells;
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
        html! {
            <div id = "rlifesrc">
                <header>
                    <h1>
                        <a href = "https://github.com/AlephAlpha/rlifesrc/">
                            { "rlifesrc" }
                        </a>
                    </h1>
                    <span id = "subheading">
                        { "A Game of Life pattern searcher written in Rust. " }
                    </span>
                </header>
                <main>
                    <div id = "world">
                        { self.data() }
                        <pre>
                        { self.world.clone().unwrap_or_default() }
                        </pre>
                        { self.status() }
                    </div>
                    <div id = "settings">
                        { self.buttons() }
                        { self.set_gen() }
                        { self.set_rule() }
                        { self.set_width() }
                        { self.set_height() }
                        { self.set_period() }
                        { self.set_dx() }
                        { self.set_dy() }
                        { self.set_max() }
                        { self.set_sym() }
                        { self.set_order() }
                        { self.set_choose() }
                    </div>
                </main>
            </div>
        }
    }
}

impl Model {
    fn data(&self) -> Html<Self> {
        html! {
            <div id="data">
                <span>
                    <abbr title = "The displayed generation.">
                    { "Gen" }
                    </abbr>
                    { ": " }
                    { self.gen }
                </span>
                <span>
                    <abbr title = "Number of known living cells in generation 0.">
                    { "Cells" }
                    </abbr>
                    { ": " }
                    { self.cells }
                </span>
            </div>
        }
    }

    fn status(&self) -> Html<Self> {
        let status = match self.status {
            Status::Found => "Found a result.",
            Status::None => "No more result.",
            Status::Searching => "Searching...",
            Status::Paused => "Paused.",
        };
        html! {
            <div id = "status">
                { status }
            </div>
        }
    }

    fn buttons(&self) -> Html<Self> {
        html! {
            <div id = "buttons">
                <button
                    onclick = |_| Msg::Start,
                    disabled = self.status == Status::Searching
                >
                    { "Start" }
                </button>
                <button
                    onclick = |_| Msg::Pause,
                    disabled = self.status != Status::Searching
                >
                    { "Pause" }
                </button>
                <button
                    onclick = |_| Msg::Reset,
                    title = "Reset the world"
                >
                    { "Set World" }
                </button>
            </div>
        }
    }

    fn set_gen(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_gen">
                    <abbr title = "The displayed generation.">
                    { "Gen" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_gen",
                    type = "range",
                    value = self.gen,
                    min = "0",
                    max = self.period.unwrap_or(1) - 1,
                    oninput = |e| Msg::SetGen(e.value.parse().unwrap())
                />
            </div>
        }
    }

    fn set_rule(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_rule">
                    <abbr title = "Rule of the cellular automaton. \
                        Supports Life-like and isotropic non-totalistic rules.">
                    { "Rule" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_rule",
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
            </div>
        }
    }

    fn set_width(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_width">
                    <abbr title = "Width of the pattern.">
                    { "Width" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_width",
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
            </div>
        }
    }

    fn set_height(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_height">
                    <abbr title = "Height of the pattern.">
                    { "Height" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_height",
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
            </div>
        }
    }

    fn set_period(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_period">
                    <abbr title = "Period of the pattern.">
                    { "Period" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_period",
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
            </div>
        }
    }

    fn set_dx(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_dx">
                    <abbr title = "Horizontal translation.">
                    { "dx" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_dx",
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
            </div>
        }
    }

    fn set_dy(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_dy">
                    <abbr title = "Vertical translation.">
                    { "dx" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_dy",
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
            </div>
        }
    }

    fn set_max(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_max">
                    <abbr title = "Maximal number of living cells in the first generation. \
                        If this value is set to 0, it means there is no limitation.">
                    { "Max" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_max",
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
            </div>
        }
    }

    fn set_sym(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_sym">
                    <abbr title = "Symmetry of the pattern.">
                    { "Sym" }
                    </abbr>
                    { ":" }
                </label>
                <select
                    id = "set_sym",
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "C1" => Msg::SetSym(Symmetry::C1),
                                "C2" => Msg::SetSym(Symmetry::C2),
                                "C4" => Msg::SetSym(Symmetry::C4),
                                "D2|" => Msg::SetSym(Symmetry::D2Row),
                                "D2-" => Msg::SetSym(Symmetry::D2Column),
                                "D2\\" => Msg::SetSym(Symmetry::D2Diag),
                                "D2/" => Msg::SetSym(Symmetry::D2Antidiag),
                                "D4+" => Msg::SetSym(Symmetry::D4Ortho),
                                "D4X" => Msg::SetSym(Symmetry::D4Diag),
                                "D8" => Msg::SetSym(Symmetry::D8),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    },
                >
                    <option> { "C1" } </option>
                    <option> { "C2" } </option>
                    <option> { "C4" } </option>
                    <option> { "D2|" } </option>
                    <option> { "D2-" } </option>
                    <option> { "D2\\" } </option>
                    <option> { "D2/" } </option>
                    <option> { "D4+" } </option>
                    <option> { "D4X" } </option>
                    <option> { "D8" } </option>
                </select>
            </div>
        }
    }

    fn set_order(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_order">
                    <abbr title = "Search order. Row first or column first.">
                    { "Order" }
                    </abbr>
                    { ":" }
                </label>
                <select
                    id = "set_order",
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "Automatic" => Msg::SetOrder(None),
                                "Column" => Msg::SetOrder(Some(true)),
                                "Row" => Msg::SetOrder(Some(false)),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    },
                >
                    <option> { "Automatic" } </option>
                    <option value = "Column"> { "Column first" } </option>
                    <option value = "Row"> { "Row first" } </option>
                </select>
            </div>
        }
    }

    fn set_choose(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_choose">
                    <abbr title = "How to choose a state for unknown cells. \
                        &quot;Smart&quot; means choosing a alive for cells in the first \
                        row/column, and dead for other cells.">
                    { "Choose" }
                    </abbr>
                    { ":" }
                </label>
                <select
                    id = "set_order",
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "Dead" => Msg::SetChoose(Choose(Dead)),
                                "Alive" => Msg::SetChoose(Choose(Alive)),
                                "Random" => Msg::SetChoose(Random),
                                "Smart" => Msg::SetChoose(Smart),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    },
                >
                    <option> { "Alive" } </option>
                    <option> { "Dead" } </option>
                    <option> { "Random" } </option>
                    <option> { "Smart" } </option>
                </select>
            </div>
        }
    }
}
