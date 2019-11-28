use crate::worker::{Request, Response, Worker};
use rlifesrc_lib::{
    Config, NewState, SearchOrder,
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
    config: Config,
    status: Status,
    gen: isize,
    cells: usize,
    world: String,
    period: isize,
    worker: Box<dyn Bridge<Worker>>,
    job: Job,
}

pub enum Msg {
    Tick,
    Start,
    Pause,
    IncGen,
    DecGen,
    SetWidth(isize),
    SetHeight(isize),
    SetPeriod(isize),
    SetDx(isize),
    SetDy(isize),
    SetTrans(Transform),
    SetSym(Symmetry),
    SetRule(String),
    SetOrder(Option<SearchOrder>),
    SetChoose(NewState),
    SetMax(Option<usize>),
    SetFront,
    SetReduce,
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
        let config: Config = Default::default();
        let status = Status::Paused;
        let job = Job::new(&mut link);
        let callback = link.send_back(Msg::DataReceived);
        let worker = Worker::bridge(callback);

        let world = String::from(
            "????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????\n\
             ????????????????",
        );
        let period = config.period;

        Model {
            config,
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
            Msg::IncGen => {
                self.gen += 1;
                self.worker.send(Request::DisplayGen(self.gen));
            }
            Msg::DecGen => {
                self.gen -= 1;
                self.worker.send(Request::DisplayGen(self.gen));
            }
            Msg::SetWidth(width) => {
                self.config.width = width;
                if self.config.transform.square_world() || self.config.symmetry.square_world() {
                    self.config.height = width;
                }
            }
            Msg::SetHeight(height) => {
                self.config.height = height;
                if self.config.transform.square_world() || self.config.symmetry.square_world() {
                    self.config.width = height;
                }
            }
            Msg::SetPeriod(period) => {
                self.config.period = period;
            }
            Msg::SetDx(dx) => {
                self.config.dx = dx;
            }
            Msg::SetDy(dy) => {
                self.config.dy = dy;
            }
            Msg::SetTrans(transform) => {
                self.config.transform = transform;
            }
            Msg::SetSym(symmetry) => {
                self.config.symmetry = symmetry;
            }
            Msg::SetRule(rule_string) => {
                self.config.rule_string = rule_string;
            }
            Msg::SetOrder(search_order) => {
                self.config.search_order = search_order;
            }
            Msg::SetChoose(new_state) => {
                self.config.new_state = new_state;
            }
            Msg::SetMax(max_cell_count) => {
                self.config.max_cell_count = max_cell_count;
            }
            Msg::SetFront => {
                self.config.non_empty_front ^= true;
            }
            Msg::SetReduce => {
                self.config.reduce_max ^= true;
            }
            Msg::Reset => {
                self.gen = 0;
                self.period = self.config.period;
                self.worker.send(Request::SetWorld(self.config.clone()));
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

    fn view(&self) -> Html<Self> {
        html! {
            <div id="rlifesrc">
                { self.header() }
                { self.main() }
            </div>
        }
    }
}

impl Model {
    fn header(&self) -> Html<Self> {
        html! {
            <header id="appbar" class="mui-appbar mui--z1">
                <table class="mui-container-fluid">
                    <tr class="mui--appbar-height">
                        <td>
                            <span id="title" class="mui--text-headline">
                                { "Rust Life Search" }
                            </span>
                            <span class="mui--text-subhead mui--hidden-xs">
                                { "A Game of Life pattern searcher written in Rust." }
                            </span>
                        </td>
                        <td class="mui--text-right">
                            <a href="https://github.com/AlephAlpha/rlifesrc/"
                                class="mui--text-headline">
                                <i class="fab fa-github"></i>
                            </a>
                        </td>
                    </tr>
                </table>
            </header>
        }
    }

    fn main(&self) -> Html<Self> {
        html! {
            <main class="mui-container-fluid">
                <div class="mui-row">
                    <div class="mui-col-sm-10 mui-col-sm-offset-1 mui-col-lg-8 mui-col-lg-offset-2">
                        <div class="mui-panel">
                            { self.data() }
                            { self.world() }
                            { self.buttons() }
                        </div>
                        <div class="mui-panel">
                            { self.settings() }
                        </div>
                    </div>
                </div>
            </main>
        }
    }

    fn world(&self) -> Html<Self> {
        html! {
            <pre id="world">
                { self.world.clone() }
            </pre>
        }
    }

    fn data(&self) -> Html<Self> {
        html! {
            <ul id="data" class="mui-list--inline mui--text-body2">
                <li>
                    <abbr title="The displayed generation.">
                        { "Generation" }
                    </abbr>
                    { ": " }
                    { self.gen }
                    <button class="mui-btn mui-btn--small btn-tiny"
                        disabled={ self.gen == 0 }
                        onclick=|_| Msg::DecGen>
                        <i class="fas fa-minus"></i>
                    </button>
                    <button class="mui-btn mui-btn--small btn-tiny"
                        disabled={ self.gen == self.period - 1 }
                        onclick=|_| Msg::IncGen>
                        <i class="fas fa-plus"></i>
                    </button>
                </li>
                <li>
                    <abbr title="Number of known living cells in the current generation.">
                        { "Cell count" }
                    </abbr>
                    { ": " }
                    { self.cells }
                </li>
                <li>
                    {
                        match self.status {
                            Status::Found => "Found a result.",
                            Status::None => "No more result.",
                            Status::Searching => "Searching...",
                            Status::Paused => "Paused.",
                        }
                    }
                </li>
            </ul>
        }
    }

    fn buttons(&self) -> Html<Self> {
        html! {
            <div id="buttons">
                <button class="mui-btn mui-btn--raised"
                    disabled={self.status == Status::Searching }
                    onclick=|_| Msg::Start>
                    <i class="fas fa-play"></i>
                    <span class="mui--hidden-xs">
                        { "Start" }
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    disabled={self.status != Status::Searching }
                    onclick=|_| Msg::Pause>
                    <i class="fas fa-pause"></i>
                    <span class="mui--hidden-xs">
                        { "Pause" }
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    onclick=|_| Msg::Reset>
                    <i class="fas fa-redo"></i>
                    <span class="mui--hidden-xs">
                        <abbr title="Apply the settings and restart.">
                            { "Set World" }
                        </abbr>
                    </span>
                </button>
            </div>
        }
    }

    fn settings(&self) -> Html<Self> {
        html! {
            <div id="settings" class="mui-form">
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
                { self.set_front() }
                { self.set_reduce() }
            </div>
        }
    }

    fn set_rule(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_rule">
                    <abbr title = "Rule of the cellular automaton. \
                        Supports Life-like and isotropic non-totalistic rules.">
                    { "Rule" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_rule"
                    type="text"
                    value={ self.config.rule_string.clone() }
                    onchange=|e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetRule(v)
                        } else {
                            Msg::None
                        }
                    }/>
            </div>
        }
    }

    fn set_width(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_width">
                    <abbr title="Width of the pattern.">
                    { "Width" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_width"
                    type="number"
                    value={ self.config.width }
                    min="1"
                    onchange =|e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetWidth(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }/>
            </div>
        }
    }

    fn set_height(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_height">
                    <abbr title="Height of the pattern.">
                    { "Height" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_height"
                    type="number"
                    value={ self.config.height }
                    min="1"
                    onchange=|e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetHeight(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }/>
            </div>
        }
    }

    fn set_period(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_period">
                    <abbr title="Period of the pattern.">
                    { "Period" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_period"
                    type="number"
                    value={ self.config.period }
                    min="1"
                    onchange=|e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetPeriod(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }/>
            </div>
        }
    }

    fn set_dx(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_dx">
                    <abbr title="Horizontal translation.">
                    { "dx" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_dx"
                    type="number"
                    value=self.config.dx
                    onchange=|e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetDx(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }/>
            </div>
        }
    }

    fn set_dy(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_dy">
                    <abbr title="Vertical translation.">
                    { "dy" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_dy"
                    type="number"
                    value={ self.config.dy }
                    onchange=|e| {
                        if let ChangeData::Value(v) = e {
                            Msg::SetDy(v.parse().unwrap())
                        } else {
                            Msg::None
                        }
                    }/>
            </div>
        }
    }

    fn set_max(&self) -> Html<Self> {
        html! {
            <div class="mui-textfield">
                <label for="set_max">
                    <abbr title="Upper bound of numbers of minimum living cells in all generations. \
                        If this value is set to 0, it means there is no limitation.">
                    { "Max cell count" }
                    </abbr>
                    { ":" }
                </label>
                <input id="set_max",
                    type="number",
                    value={
                        match self.config.max_cell_count {
                            None => 0,
                            Some(i) => i,
                        }
                    },
                    min="0",
                    onchange=|e| {
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
                    }/>
            </div>
        }
    }

    fn set_front(&self) -> Html<Self> {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_front",
                        type="checkbox",
                        checked={ self.config.non_empty_front },
                        onclick=|_| Msg::SetFront/>
                    <abbr title="Force the first row or column to be nonempty.\n\
                        Here 'front' means the first row or column to search, \
                        according to the search order.">
                    { "Non empty front" }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_reduce(&self) -> Html<Self> {
        html! {
            <div class="mui-checkbox">
                <label>
                    <input id="set_reduce",
                        type="checkbox",
                        checked={ self.config.reduce_max },
                        onclick=|_| Msg::SetReduce/>
                    <abbr title="Reduce the max cell count when a result is found.\n\
                        The new max cell count will be set to the cell count of\
                        the current result minus one.">
                    { "Reduce max cell count" }
                    </abbr>
                </label>
            </div>
        }
    }

    fn set_trans(&self) -> Html<Self> {
        html! {
            <div class="mui-select">
                <label for="set_trans">
                    <abbr title="Transformations after the last generation.\n\
                        After the last generation, the pattern will return to \
                        the first generation, applying this transformation first, \
                        and then the translation defined by dx and dy.">
                    { "Transformation" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_trans"
                    onchange=|e| {
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
                    }>
                    <option> { "Id" } </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "Rotate 90°" }
                    </option>
                    <option> { "Rotate 180°" } </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "Rotate 270°" }
                    </option>
                    <option> { "Flip |" } </option>
                    <option> { "Flip -" } </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "Flip \\" }
                    </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "Flip /" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_sym(&self) -> Html<Self> {
        html! {
            <div class="mui-select">
                <label for="set_sym">
                    <abbr title="Symmetry of the pattern.">
                    { "Symmetry" }
                    </abbr>
                    { ":" }
                </label>
                <select id ="set_sym"
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
                    }>
                    <option> { "C1" } </option>
                    <option> { "C2" } </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "C4" }
                    </option>
                    <option> { "D2|" } </option>
                    <option> { "D2-" } </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "D2\\" }
                    </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "D2/" }
                    </option>
                    <option> { "D4+" } </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "D4X" }
                    </option>
                    <option disabled={ self.config.width != self.config.height}>
                        { "D8" }
                    </option>
                </select>
            </div>
        }
    }

    fn set_order(&self) -> Html<Self> {
        html! {
            <div class="mui-select">
                <label for="set_order">
                    <abbr title="The order to find a new unknown cell.\n\
                        It will always search all generations of a cell first, \
                        and then go to another cell.">
                    { "Search order" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_order"
                    onchange=|e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "Automatic" => Msg::SetOrder(None),
                                "Column" => Msg::SetOrder(Some(SearchOrder::ColumnFirst)),
                                "Row" => Msg::SetOrder(Some(SearchOrder::RowFirst)),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    } >
                    <option> { "Automatic" } </option>
                    <option value="Column"> { "Column first" } </option>
                    <option value="Row"> { "Row first" } </option>
                </select>
            </div>
        }
    }

    fn set_choose(&self) -> Html<Self> {
        html! {
            <div class="mui-select">
                <label for="set_choose">
                    <abbr title="How to choose a state for unknown cells.">
                    { "Choice of state for unknown cells" }
                    </abbr>
                    { ":" }
                </label>
                <select id="set_order"
                    onchange=|e| {
                        if let ChangeData::Select(s) = e {
                            match s.raw_value().as_ref() {
                                "Dead" => Msg::SetChoose(NewState::Choose(Dead)),
                                "Alive" => Msg::SetChoose(NewState::Choose(Alive)),
                                "Random" => Msg::SetChoose(NewState::Random),
                                _ => Msg::None,
                            }
                        } else {
                            Msg::None
                        }
                    }>
                    <option> { "Alive" } </option>
                    <option> { "Dead" } </option>
                    <option> { "Random" } </option>
                </select>
            </div>
        }
    }
}
