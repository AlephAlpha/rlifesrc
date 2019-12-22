use crate::{
    settings::Settings,
    worker::{Request, Response, Worker},
    world::World,
};
use rlifesrc_lib::{Config, Status};
use std::time::Duration;
use stdweb::web::event::IEvent;
use yew::{
    format::Json,
    html,
    services::{storage::Area, DialogService, IntervalService, StorageService, Task},
    Bridge, Bridged, Callback, Component, ComponentLink, Html, ShouldRender,
};

const KEY: &str = "rlifesrc.world";
const INIT_WORLD: &str = "x = 16, y = 16, rule = B3/S23\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????$\n\
                          ????????????????!";

pub struct App {
    config: Config,
    status: Status,
    gen: isize,
    cells: usize,
    world: String,
    period: isize,
    worker: Box<dyn Bridge<Worker>>,
    storage: StorageService,
    job: Job,
}

pub enum Msg {
    Tick,
    IncGen,
    DecGen,
    Start,
    Pause,
    Store,
    Restore,
    Apply(Config),
    DataReceived(Response),
    None,
}

struct Job {
    interval: IntervalService,
    callback: Callback<()>,
    task: Option<Box<dyn Task>>,
}

impl Job {
    fn new(link: &mut ComponentLink<App>) -> Self {
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

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let config: Config = Config::default();
        let status = Status::Paused;
        let job = Job::new(&mut link);
        let callback = link.send_back(Msg::DataReceived);
        let worker = Worker::bridge(callback);
        let storage = StorageService::new(Area::Local);
        let world = INIT_WORLD.to_owned();
        let period = config.period;

        App {
            config,
            status,
            gen: 0,
            cells: 0,
            world,
            period,
            worker,
            storage,
            job,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                self.worker.send(Request::DisplayGen(self.gen));
                return false;
            }
            Msg::IncGen => {
                if self.gen >= self.period - 1 {
                    return false;
                }
                self.gen += 1;
                self.worker.send(Request::DisplayGen(self.gen));
            }
            Msg::DecGen => {
                if self.gen <= 0 {
                    return false;
                }
                self.gen -= 1;
                self.worker.send(Request::DisplayGen(self.gen));
            }
            Msg::Start => {
                self.worker.send(Request::Start);
                return false;
            }
            Msg::Pause => {
                self.worker.send(Request::Pause);
                return false;
            }
            Msg::Store => {
                self.worker.send(Request::Store);
                return false;
            }
            Msg::Restore => {
                if let Json(Ok(world_ser)) = self.storage.restore(KEY) {
                    self.worker.send(Request::Restore(world_ser));
                }
                return false;
            }
            Msg::Apply(config) => {
                self.config = config;
                self.gen = 0;
                self.period = self.config.period;
                self.worker.send(Request::SetWorld(self.config.clone()));
            }
            Msg::DataReceived(response) => match response {
                Response::UpdateWorld((world, cells)) => {
                    self.world = world;
                    self.cells = cells;
                }
                Response::UpdateConfig(config) => {
                    self.config = config;
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
                Response::Store(world_ser) => {
                    self.storage.store(KEY, Json(&world_ser));
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
                { self.footer() }
            </div>
        }
    }
}

impl App {
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

    fn footer(&self) -> Html<Self> {
        html! {
            <footer id="footer" class="mui-container-fluid">
                <div class="mui--text-caption mui--text-center">
                    { "Powered by " }
                    <a href="https://yew.rs">
                        { "Yew" }
                    </a>
                    { " & " }
                    <a href="https://www.muicss.com">
                        { "MUI CSS" }
                    </a>
                </div>
            </footer>
        }
    }

    fn main(&self) -> Html<Self> {
        html! {
            <main class="mui-container-fluid">
                <div class="mui-row">
                    <div class="mui-col-sm-10 mui-col-sm-offset-1 mui-col-lg-8 mui-col-lg-offset-2">
                        <div class="mui-panel">
                            <ul class="mui-tabs__bar">
                                <li class="mui--is-active">
                                    <a data-mui-toggle="tab" data-mui-controls="pane-world">
                                        { "World" }
                                    </a>
                                </li>
                                <li>
                                    <a data-mui-toggle="tab" data-mui-controls="pane-settings">
                                        { "Settings" }
                                    </a>
                                </li>
                            </ul>
                            <div class="mui-tabs__pane mui--is-active" id="pane-world">
                                { self.data() }
                                <World world= &self.world />
                                { self.buttons() }
                            </div>
                            <div class="mui-tabs__pane" id="pane-settings">
                                <Settings config=&self.config callback=Msg::Apply/>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        }
    }

    fn data(&self) -> Html<Self> {
        html! {
            <ul id="data" class="mui-list--inline mui--text-body2">
                <li onmousewheel=|e| {
                    e.prevent_default();
                    if e.delta_y() < 0.0 {
                        Msg::IncGen
                    } else {
                        Msg::DecGen
                    }
                }>
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
            <div class="buttons">
                <button class="mui-btn mui-btn--raised"
                    disabled={ self.status == Status::Searching }
                    onclick=|_| Msg::Start>
                    <i class="fas fa-play"></i>
                    <span class="mui--hidden-xs">
                        { "Start" }
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    disabled={ self.status != Status::Searching }
                    onclick=|_| Msg::Pause>
                    <i class="fas fa-pause"></i>
                    <span class="mui--hidden-xs">
                        { "Pause" }
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    disabled={ self.status == Status::Searching }
                    onclick=|_| Msg::Store>
                    <i class="fas fa-save"></i>
                    <span class="mui--hidden-xs">
                        <abbr title="Store the search status in the browser.">
                            { "Save" }
                        </abbr>
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    onclick=|_| Msg::Restore>
                    <i class="fas fa-file-import"></i>
                    <span class="mui--hidden-xs">
                        <abbr title="Load the saved search status.">
                            { "Load" }
                        </abbr>
                    </span>
                </button>
            </div>
        }
    }
}
