use crate::worker::{Data, Request, Response, Worker};
use rlifesrc_lib::{
    NewState::{self, Choose, Random, Stupid},
    State::{Alive, Dead},
    Status, Symmetry, Transform,
};
use std::time::Duration;
use yew::{
    html,
    html::ChangeData,
    services::{DialogService, IntervalService, Task},
    *,
};

pub struct Model {
    data: Data,
    status: Status,
    gen: isize,
    cells: u32,
    world: String,
    period: isize,
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
    SetTrans(Transform),
    SetSym(Symmetry),
    SetRule(String),
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
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let data: Data = Default::default();
        let status = Status::Paused;
        let job = Job::new(&mut link);
        let callback = link.send_back(Msg::DataReceived);
        let worker = Worker::bridge(callback);

        let world = String::from(
            "??????????????????????????\n\
             ??????????????????????????\n\
             ??????????????????????????\n\
             ??????????????????????????\n\
             ??????????????????????????\n\
             ??????????????????????????\n\
             ??????????????????????????\n\
             ??????????????????????????",
        );
        let period = data.period;

        Model {
            data,
            status,
            gen: 0,
            cells: 0,
            world,
            period,
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
                self.data.width = width;
                if self.data.transform.square_world() || self.data.symmetry.square_world() {
                    self.data.height = width;
                }
            }
            Msg::SetHeight(height) => {
                self.data.height = height;
                if self.data.transform.square_world() || self.data.symmetry.square_world() {
                    self.data.width = height;
                }
            }
            Msg::SetPeriod(period) => {
                self.data.period = period;
            }
            Msg::SetDx(dx) => {
                self.data.dx = dx;
            }
            Msg::SetDy(dy) => {
                self.data.dy = dy;
            }
            Msg::SetTrans(transform) => {
                self.data.transform = transform;
            }
            Msg::SetSym(symmetry) => {
                self.data.symmetry = symmetry;
            }
            Msg::SetRule(rule_string) => {
                self.data.rule_string = rule_string;
            }
            Msg::SetOrder(column_first) => {
                self.data.column_first = column_first;
            }
            Msg::SetChoose(new_state) => {
                self.data.new_state = new_state;
            }
            Msg::SetMax(max_cell_count) => {
                self.data.max_cell_count = max_cell_count;
            }
            Msg::Reset => {
                self.gen = 0;
                self.period = self.data.period;
                self.worker.send(Request::SetWorld(self.data.clone()));
            }
            Msg::DataReceived(response) => match response {
                Response::UpdateWorld((world, cells)) => {
                    self.world = world;
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
                        <a
                            href = "https://github.com/AlephAlpha/rlifesrc/"
                            title = "Fork me on GitHub"
                        >
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
                        { self.world() }
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
                        { self.set_trans() }
                        { self.set_sym() }
                        { self.set_max() }
                        { self.set_order() }
                        { self.set_choose() }
                    </div>
                </main>
            </div>
        }
    }
}

impl Model {
    fn world(&self) -> Html<Self> {
        html! {
            <pre>
                { self.world.clone() }
            </pre>
        }
    }

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
                    onclick = |_| Msg::Start
                    disabled = self.status == Status::Searching
                >
                    { "Start" }
                </button>
                <button
                    onclick = |_| Msg::Pause
                    disabled = self.status != Status::Searching
                >
                    { "Pause" }
                </button>
                <button
                    onclick = |_| Msg::Reset
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
                    id = "set_gen"
                    type = "range"
                    value = self.gen
                    min = "0"
                    max = self.period - 1
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
                    id = "set_rule"
                    type = "text"
                    value = self.data.rule_string.clone()
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetRule(v)
                        } else {
                            Msg::None
                        }
                    }
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
                    id = "set_width"
                    type = "number"
                    value = self.data.width
                    min = "1"
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetWidth(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }
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
                    id = "set_height"
                    type = "number"
                    value = self.data.height
                    min = "1"
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetHeight(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }
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
                    id = "set_period"
                    type = "number"
                    value = self.data.period
                    min = "1"
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetPeriod(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }
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
                    id = "set_dx"
                    type = "number"
                    value = self.data.dx
                    onchange = |e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetDx(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }
                />
            </div>
        }
    }

    fn set_dy(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_dy">
                    <abbr title = "Vertical translation.">
                    { "dy" }
                    </abbr>
                    { ":" }
                </label>
                <input
                    id = "set_dy"
                    type = "number"
                    value = self.data.dy
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
                    value = match self.data.max_cell_count {
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
                    }
                />
            </div>
        }
    }

    fn set_trans(&self) -> Html<Self> {
        html! {
            <div class = "setting">
                <label for = "set_trans">
                    <abbr title = "How the pattern transform after a period. \
                    It will apply this transformation before the translation.">
                    { "Trans" }
                    </abbr>
                    { ":" }
                </label>
                <select
                    id = "set_trans"
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "Id" => Msg::SetTrans(Transform::Id),
                                "Rotate 90°" => Msg::SetTrans(Transform::Rotate90),
                                "Rotate 180°" => Msg::SetTrans(Transform::Rotate180),
                                "Rotate 270°" => Msg::SetTrans(Transform::Rotate270),
                                "Flip -" => Msg::SetTrans(Transform::FlipRow),
                                "Flip |" => Msg::SetTrans(Transform::FlipCol),
                                "Flip \\" => Msg::SetTrans(Transform::FlipDiag),
                                "Flip /" => Msg::SetTrans(Transform::FlipAntidiag),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    }
                >
                    <option> { "Id" } </option>
                    <option disabled = self.data.width != self.data.height>
                        { "Rotate 90°" }
                    </option>
                    <option> { "Rotate 180°" } </option>
                    <option disabled = self.data.width != self.data.height>
                        { "Rotate 270°" }
                    </option>
                    <option> { "Flip |" } </option>
                    <option> { "Flip -" } </option>
                    <option disabled = self.data.width != self.data.height>
                        { "Flip \\" }
                    </option>
                    <option disabled = self.data.width != self.data.height>
                        { "Flip /" }
                    </option>
                </select>
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
                    id = "set_sym"
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "C1" => Msg::SetSym(Symmetry::C1),
                                "C2" => Msg::SetSym(Symmetry::C2),
                                "C4" => Msg::SetSym(Symmetry::C4),
                                "D2-" => Msg::SetSym(Symmetry::D2Row),
                                "D2|" => Msg::SetSym(Symmetry::D2Col),
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
                    }
                >
                    <option> { "C1" } </option>
                    <option> { "C2" } </option>
                    <option disabled = self.data.width != self.data.height>
                        { "C4" }
                    </option>
                    <option> { "D2|" } </option>
                    <option> { "D2-" } </option>
                    <option disabled = self.data.width != self.data.height>
                        { "D2\\" }
                    </option>
                    <option disabled = self.data.width != self.data.height>
                        { "D2/" }
                    </option>
                    <option> { "D4+" } </option>
                    <option disabled = self.data.width != self.data.height>
                        { "D4X" }
                    </option>
                    <option disabled = self.data.width != self.data.height>
                        { "D8" }
                    </option>
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
                    id = "set_order"
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
                    }
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
                        &quot;Stupid&quot; means choosing a alive for cells in the first \
                        row/column, and dead for other cells.">
                    { "Choose" }
                    </abbr>
                    { ":" }
                </label>
                <select
                    id = "set_order"
                    onchange = |e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "Dead" => Msg::SetChoose(Choose(Dead)),
                                "Alive" => Msg::SetChoose(Choose(Alive)),
                                "Random" => Msg::SetChoose(Random),
                                "Stupid" => Msg::SetChoose(Stupid),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    }
                >
                    <option> { "Alive" } </option>
                    <option> { "Dead" } </option>
                    <option> { "Random" } </option>
                    <option> { "Stupid" } </option>
                </select>
            </div>
        }
    }
}
