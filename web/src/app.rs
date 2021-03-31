use crate::{
    help::Help,
    settings::Settings,
    worker::{Request, Response, UpdateMessage, Worker},
    world::World,
};
use build_timestamp::build_time;
use js_sys::Array;
use log::{debug, error};
use rlifesrc_lib::{Config, Status};
use std::time::Duration;
use wasm_bindgen::JsValue;
use web_sys::{Blob, BlobPropertyBag, FileList, HtmlAnchorElement, HtmlElement, Url};
use yew::{
    events::WheelEvent,
    format::{Json, Text},
    html,
    html::ChangeData,
    services::{
        interval::{IntervalService, IntervalTask},
        reader::{FileData, ReaderService, ReaderTask},
        DialogService,
    },
    Bridge, Bridged, Component, ComponentLink, Html, ShouldRender,
};

build_time!("%Y-%m-%d %H:%M:%S UTC");

#[derive(Debug, PartialEq, Eq)]
pub enum Tab {
    World,
    Settings,
    Help,
}

pub struct App {
    link: ComponentLink<Self>,
    config: Config,
    status: Status,
    paused: bool,
    gen: i32,
    cells: u32,
    world: String,
    max_partial: bool,
    find_all: bool,
    found_count: u32,
    timing: Duration,
    worker: Box<dyn Bridge<Worker>>,
    interval_task: Option<IntervalTask>,
    reader_task: Option<ReaderTask>,
    tab: Tab,
}

#[derive(Debug)]
pub enum Msg {
    Tick,
    IncGen,
    DecGen,
    Start,
    Pause,
    Reset,
    Save,
    Load(FileList),
    SendFile(FileData),
    SetMaxPartial,
    SetFindAll,
    Apply(Config),
    DataReceived(Response),
    None,
    ChangeTab(Tab),
}

impl App {
    fn start_job(&mut self) {
        let handle = IntervalService::spawn(
            Duration::from_millis(1000 / 60),
            self.link.callback(|_| Msg::Tick),
        );
        self.interval_task = Some(handle);
    }

    fn stop_job(&mut self) {
        self.interval_task.take();
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let config: Config = Config::default();
        let status = Status::Initial;
        let world = "Loading...".to_owned();
        let callback = link.callback(Msg::DataReceived);
        let worker = Worker::bridge(callback);

        App {
            link,
            config,
            status,
            paused: true,
            gen: 0,
            cells: 0,
            world,
            max_partial: false,
            find_all: false,
            found_count: 0,
            timing: Duration::default(),
            worker,
            interval_task: None,
            reader_task: None,
            tab: Tab::World,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Tick => {
                if self.max_partial {
                    self.worker.send(Request::MaxPartial);
                } else {
                    self.worker.send(Request::DisplayGen(self.gen));
                }
            }
            Msg::IncGen => {
                if self.gen < self.config.period - 1 {
                    self.gen += 1;
                    self.worker.send(Request::DisplayGen(self.gen));
                    return true;
                }
            }
            Msg::DecGen => {
                if self.gen > 0 {
                    self.gen -= 1;
                    self.worker.send(Request::DisplayGen(self.gen));
                    return true;
                }
            }
            Msg::Start => {
                self.worker.send(Request::Start);
                self.start_job();
            }
            Msg::Pause => self.worker.send(Request::Pause),
            Msg::Reset => self.worker.send(Request::SetWorld(self.config.clone())),
            Msg::Save => self.worker.send(Request::Save),
            Msg::Load(files) => {
                let file = files.get(0).unwrap();
                let mut reader_service = ReaderService::new();
                let task = reader_service.read_file(file, self.link.callback(Msg::SendFile));
                match task {
                    Ok(task) => self.reader_task = Some(task),
                    Err(e) => error!("Error opening file reader: {}", e),
                }
            }
            Msg::SendFile(data) => {
                let Json(world_ser) = Ok(data.content).into();
                match world_ser {
                    Ok(world_ser) => self.worker.send(Request::Load(world_ser)),
                    Err(e) => {
                        error!("Error parsing save file: {}", e);
                        DialogService::alert("Broken saved file.");
                    }
                }
            }
            Msg::SetMaxPartial => {
                self.max_partial ^= true;
                if self.max_partial {
                    self.worker.send(Request::MaxPartial);
                } else {
                    self.worker.send(Request::DisplayGen(self.gen));
                }
                return true;
            }
            Msg::SetFindAll => {
                self.find_all ^= true;
                self.worker.send(Request::SetFindAll(self.find_all));
                if self.max_partial {
                    self.worker.send(Request::MaxPartial);
                } else {
                    self.worker.send(Request::DisplayGen(self.gen));
                }
                return true;
            }
            Msg::Apply(config) => {
                self.tab = Tab::World;
                self.config = config;
                self.gen = 0;
                self.worker.send(Request::SetWorld(self.config.clone()));
                return true;
            }
            Msg::DataReceived(response) => {
                match response {
                    Response::Update(UpdateMessage {
                        world,
                        cells,
                        status,
                        paused,
                        found_count,
                        timing,
                        config,
                    }) => {
                        if let Some(world) = world {
                            self.world = world;
                        }
                        if let Some(cells) = cells {
                            self.cells = cells;
                        }
                        if let Some(config) = config {
                            self.config = config;
                        }
                        self.paused = paused;
                        if paused {
                            self.stop_job()
                        }
                        self.status = status;
                        self.found_count = found_count;
                        if let Some(timing) = timing {
                            self.timing = timing;
                        }
                    }
                    Response::Error(error) => {
                        DialogService::alert(&error);
                    }
                    Response::Save(world_ser) => {
                        let text: Text = Json(&world_ser).into();
                        match text {
                            Ok(text) => {
                                debug!("Generated save file: {:?}", text);
                                download(&text, "save.json", "application/json").unwrap()
                            }
                            Err(e) => error!("Error generating save file: {}", e),
                        }
                    }
                };
                return true;
            }
            Msg::ChangeTab(tab) => {
                self.tab = tab;
                return true;
            }
            Msg::None => (),
        }
        false
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div id="rlifesrc" class="window">
                { self.header() }
                { self.main() }
            </div>
        }
    }
}

impl App {
    fn header(&self) -> Html {
        html! {
            <header id="appbar" class="title-bar">
                <div class="title-bar-text">
                    { "Rust Life Search" }
                </div>
                <div class="title-bar-controls">
                    <button aria-label="Help"
                        onclick=self.link.callback(|_| Msg::ChangeTab(Tab::Help)) />
                    <button aria-label="Close" />
                </div>
            </header>
        }
    }

    fn main(&self) -> Html {
        html! {
            <main class="window-body">
                <menu role="tablist">
                    <button aria-selected=self.tab==Tab::World
                        aria-controls="pane-world"
                        onclick=self.link.callback(|_| Msg::ChangeTab(Tab::World))>
                        { "World" }
                    </button>
                    <button aria-selected=self.tab==Tab::Settings
                        aria-controls="pane-settings"
                        onclick=self.link.callback(|_| Msg::ChangeTab(Tab::Settings))>
                        { "Settings" }
                    </button>
                    <button aria-selected=self.tab==Tab::Help
                        aria-controls="pane-help"
                        onclick=self.link.callback(|_| Msg::ChangeTab(Tab::Help))>
                        { "Help" }
                    </button>
                </menu>
                <article hidden=self.tab!=Tab::World role="tabpanel" id="pane-world">
                    { self.data() }
                    <fieldset>
                        <div class="field-row">
                            <input id="max-partial"
                                type="checkbox"
                                checked=self.max_partial
                                onclick=self.link.callback(|_| Msg::SetMaxPartial)/>
                            <label for="max-partial">
                                <abbr title="Show maximal partial result.">
                                    { "Show max partial." }
                                </abbr>
                            </label>
                        </div>
                        <div class="field-row">
                            <input id="find-all"
                                type="checkbox"
                                checked=self.find_all
                                onclick=self.link.callback(|_| Msg::SetFindAll)/>
                            <label for="find-all">
                                <abbr title="Find all results. Will not stop until all results \
                                             are found.">
                                    { "Find all results. Won't stop when found one." }
                                </abbr>
                            </label>
                        </div>
                    </fieldset>
                    <World world=&self.world/>
                    { self.buttons() }
                </article>
                <article hidden=self.tab!=Tab::Settings role="tabpanel" id="pane-settings">
                    <Settings config=&self.config
                        callback=self.link.callback(Msg::Apply)/>
                </article>
                <article hidden=self.tab!=Tab::Help role="tabpanel" id="pane-help">
                    <Help/>
                </article>
            </main>
        }
    }

    fn data(&self) -> Html {
        let onwheel = self.link.callback(|e: WheelEvent| {
            e.prevent_default();
            if e.delta_y() < 0.0 {
                Msg::IncGen
            } else {
                Msg::DecGen
            }
        });
        html! {
            <div class="status-bar" id="data">
                <div onwheel=onwheel class="status-bar-field" hidden=self.max_partial>
                    <abbr title="The displayed generation.">
                        { "Generation" }
                    </abbr>
                    { ": " }
                    { self.gen }
                    <button class="set-gen"
                        disabled=self.gen == 0
                        onclick=self.link.callback(|_| Msg::DecGen)>
                        { "-" }
                    </button>
                    <button class="set-gen"
                        disabled=self.gen == self.config.period - 1
                        onclick=self.link.callback(|_| Msg::IncGen)>
                        { "+" }
                    </button>
                </div>
                <div class="status-bar-field">
                    <abbr title="Number of known living cells in the current generation. \
                        For Generations rules, dying cells are not counted.">
                        { "Cell count" }
                    </abbr>
                    { ": " }
                    { self.cells }
                </div>
                <div class="status-bar-field" hidden=!self.find_all>
                    <abbr title="Number of found results.">
                        { "Found" }
                    </abbr>
                    { ": " }
                    { self.found_count }
                </div>
                <div class="status-bar-field" hidden=!self.paused>
                    <abbr title="Time taken by the search.">
                        { "Time" }
                    </abbr>
                    { ": " }
                    { format!("{:?}", self.timing) }
                </div>
                <div class="status-bar-field">
                    {
                        match self.status {
                            Status::Initial => "",
                            Status::Found => "Found a result.",
                            Status::None => "No more result.",
                            Status::Searching => if !self.paused {
                                "Searching..."
                            } else {
                                "Paused."
                            },
                        }
                    }
                </div>
            </div>
        }
    }

    fn buttons(&self) -> Html {
        html! {
            <div class="buttons">
                <button disabled=!self.paused
                    onclick=self.link.callback(|_| Msg::Start)>
                    { "Start" }
                </button>
                <button disabled=self.paused
                    onclick=self.link.callback(|_| Msg::Pause)>
                    { "Pause" }
                </button>
                <button disabled=!self.paused
                    onclick=self.link.callback(|_| Msg::Reset)>
                    <abbr title="Reset the world.">
                        { "Reset" }
                    </abbr>
                </button>
                <button disabled=!self.paused
                    onclick=self.link.callback(|_| Msg::Save)>
                    <abbr title="Save the search status in a json file.">
                        { "Save" }
                    </abbr>
                </button>
                <button onclick=self.link.callback(|_| {
                        click_button("load").unwrap();
                        Msg::None
                    })>
                    <abbr title="Load the search status from a json file.">
                        { "Load" }
                    </abbr>
                </button>
                <input id="load"
                    type="file"
                    hidden=true
                    onchange=self.link.callback(|e| match e {
                        ChangeData::Files(files) => Msg::Load(files),
                        _ => Msg::None,
                    })/>
            </div>
        }
    }
}

fn download(text: &str, name: &str, mime: &str) -> Result<(), JsValue> {
    let a = HtmlAnchorElement::from(JsValue::from(
        web_sys::window()
            .ok_or(JsValue::UNDEFINED)?
            .document()
            .ok_or(JsValue::UNDEFINED)?
            .create_element("a")?,
    ));
    a.set_download(name);

    let array = Array::new();
    array.push(&JsValue::from_str(text));

    let blob = Blob::new_with_str_sequence_and_options(&array, BlobPropertyBag::new().type_(mime))?;

    a.set_href(&Url::create_object_url_with_blob(&blob)?);
    a.click();
    Url::revoke_object_url(&a.href())
}

fn click_button(id: &str) -> Result<(), JsValue> {
    let button = HtmlElement::from(JsValue::from(
        web_sys::window()
            .ok_or(JsValue::UNDEFINED)?
            .document()
            .ok_or(JsValue::UNDEFINED)?
            .get_element_by_id(id)
            .ok_or(JsValue::UNDEFINED)?,
    ));
    button.click();
    Ok(())
}
