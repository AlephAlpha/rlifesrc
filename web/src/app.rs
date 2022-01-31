#![allow(clippy::unused_unit)]

use crate::{
    help::Help,
    settings::Settings,
    worker::{Request, Response, UpdateMessage, Worker},
    world::World,
};
use build_time::build_time_utc;
use gloo::{
    dialogs,
    file::{
        callbacks::{read_as_text, FileReader},
        FileList as GlooFileList,
    },
    timers::callback::Interval,
};
use js_sys::Array;
use log::{debug, error};
use rlifesrc_lib::{Config, Status};
use std::time::Duration;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{
    Blob, BlobPropertyBag, Event, FileList, HtmlAnchorElement, HtmlElement, HtmlInputElement, Url,
};
use yew::{events::WheelEvent, html, Component, Context, Html};
use yew_agent::{Bridge, Bridged};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["mui", "tabs"])]
    fn activate(tab: &str);
}

pub struct App {
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
    interval: Option<Interval>,
    reader: Option<FileReader>,
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
    SendFile(String),
    SetMaxPartial,
    SetFindAll,
    Apply(Config),
    DataReceived(Response),
}

impl App {
    fn start_job(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let handle = Interval::new(1000 / 60, move || link.send_message(Msg::Tick));
        self.interval = Some(handle);
    }

    fn stop_job(&mut self) {
        self.interval.take();
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let config: Config = Config::default();
        let status = Status::Initial;
        let world = "Loading...".to_owned();
        let callback = ctx.link().callback(Msg::DataReceived);
        let worker = Worker::bridge(callback);

        Self {
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
            interval: None,
            reader: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                self.start_job(ctx);
            }
            Msg::Pause => self.worker.send(Request::Pause),
            Msg::Reset => self.worker.send(Request::SetWorld(self.config.clone())),
            Msg::Save => self.worker.send(Request::Save),
            Msg::Load(files) => {
                let file = &GlooFileList::from(files)[0];
                let link = ctx.link().clone();
                let task = read_as_text(file, move |content| match content {
                    Ok(content) => link.send_message(Msg::SendFile(content)),
                    Err(e) => error!("Error opening file reader: {}", e),
                });
                self.reader = Some(task)
            }
            Msg::SendFile(data) => {
                let world_ser = serde_json::from_str(&data);
                match world_ser {
                    Ok(world_ser) => self.worker.send(Request::Load(world_ser)),
                    Err(e) => {
                        error!("Error parsing save file: {}", e);
                        dialogs::alert("Broken saved file.");
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
                self.config = config;
                self.gen = 0;
                self.worker.send(Request::SetWorld(self.config.clone()));
                activate("pane-world");
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
                    Response::Error {
                        message,
                        goto_config,
                    } => {
                        dialogs::alert(&message);
                        if goto_config {
                            activate("pane-settings");
                        }
                    }
                    Response::Save(world_ser) => match serde_json::to_string(&world_ser) {
                        Ok(text) => {
                            debug!("Generated save file: {:?}", text);
                            download(&text, "save.json", "application/json").unwrap()
                        }
                        Err(e) => error!("Error generating save file: {}", e),
                    },
                };
                return true;
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="rlifesrc">
                { self.header() }
                { self.main(ctx) }
                { self.footer() }
            </div>
        }
    }
}

impl App {
    fn header(&self) -> Html {
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

    fn footer(&self) -> Html {
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
                <div class="mui--text-caption mui--text-center">
                    <a href="https://github.com/AlephAlpha/rlifesrc/blob/master/CHANGELOG.md">
                        { format!("Last updated at {}", build_time_utc!()) }
                    </a>
                </div>
            </footer>
        }
    }

    fn main(&self, ctx: &Context<Self>) -> Html {
        html! {
            <main class="mui-container-fluid">
                <div class="mui-row">
                    <div class="mui-col-sm-10 mui-col-sm-offset-1 mui-col-lg-8 mui-col-lg-offset-2">
                        <div class="mui-panel">
                            <ul class="mui-tabs__bar">
                                <li class="mui--is-active">
                                    <a data-mui-toggle="tab" data-mui-controls="pane-world">
                                        <i class="fas fa-globe"></i>
                                        <span class="mui--hidden-xs"> { "World" } </span>
                                    </a>
                                </li>
                                <li>
                                    <a data-mui-toggle="tab" data-mui-controls="pane-settings">
                                        <i class="fas fa-cog"></i>
                                        <span class="mui--hidden-xs"> { "Settings" } </span>
                                    </a>
                                </li>
                                <li>
                                    <a data-mui-toggle="tab" data-mui-controls="pane-help">
                                        <i class="fas fa-question-circle"></i>
                                        <span class="mui--hidden-xs"> { "Help" } </span>
                                    </a>
                                </li>
                            </ul>
                            <div class="mui-tabs__pane mui--is-active" id="pane-world">
                                { self.data(ctx) }
                                <div class="mui-checkbox">
                                    <label>
                                        <input id="max-partial"
                                            type="checkbox"
                                            checked={self.max_partial}
                                            onclick={ctx.link().callback(|_| Msg::SetMaxPartial)}/>
                                        <abbr title="Show maximal partial result.">
                                            { "Show max partial." }
                                        </abbr>
                                    </label>
                                </div>
                                <div class="mui-checkbox">
                                    <label>
                                        <input id="find-all"
                                            type="checkbox"
                                            checked={self.find_all}
                                            onclick={ctx.link().callback(|_| Msg::SetFindAll)}/>
                                        <abbr title="Find all results. Will not stop until all results \
                                                     are found.">
                                            { "Find all results. Won't stop when found one." }
                                        </abbr>
                                    </label>
                                </div>
                                <World world={self.world.clone()}/>
                                { self.buttons(ctx) }
                            </div>
                            <div class="mui-tabs__pane" id="pane-settings">
                                <Settings config={self.config.clone()}
                                    callback={ctx.link().callback(Msg::Apply)}/>
                            </div>
                            <div class="mui-tabs__pane" id="pane-help">
                                <Help/>
                            </div>
                        </div>
                    </div>
                </div>
            </main>
        }
    }

    fn data(&self, ctx: &Context<Self>) -> Html {
        let onwheel = ctx.link().callback(|e: WheelEvent| {
            e.prevent_default();
            if e.delta_y() < 0.0 {
                Msg::IncGen
            } else {
                Msg::DecGen
            }
        });
        html! {
            <ul id="data" class="mui-list--inline mui--text-body2">
                <li onwheel={onwheel}
                    class={if self.max_partial { "mui--hide" } else { "" }}>
                    <abbr title="The displayed generation.">
                        { "Generation" }
                    </abbr>
                    { ": " }
                    { self.gen }
                    <button class="mui-btn mui-btn--small btn-tiny"
                        disabled={self.gen == 0}
                        onclick={ctx.link().callback(|_| Msg::DecGen)}>
                        <i class="fas fa-minus"></i>
                    </button>
                    <button class="mui-btn mui-btn--small btn-tiny"
                        disabled={self.gen == self.config.period - 1}
                        onclick={ctx.link().callback(|_| Msg::IncGen)}>
                        <i class="fas fa-plus"></i>
                    </button>
                </li>
                <li>
                    <abbr title="Number of known living cells in the current generation. \
                        For Generations rules, dying cells are not counted.">
                        { "Cell count" }
                    </abbr>
                    { ": " }
                    { self.cells }
                </li>
                <li class={if self.find_all { "" } else { "mui--hide" }}>
                    <abbr title="Number of found results.">
                        { "Found" }
                    </abbr>
                    { ": " }
                    { self.found_count }
                </li>
                <li class={if self.paused { "" } else { "mui--hide" }}>
                    <abbr title="Time taken by the search.">
                        { "Time" }
                    </abbr>
                    { ": " }
                    { format!("{:?}", self.timing) }
                </li>
                <li>
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
                </li>
            </ul>
        }
    }

    fn buttons(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="buttons">
                <button class="mui-btn mui-btn--raised"
                    disabled={!self.paused}
                    onclick={ctx.link().callback(|_| Msg::Start)}>
                    <i class="fas fa-play"></i>
                    <span class="mui--hidden-xs">
                        { "Start" }
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    disabled={self.paused}
                    onclick={ctx.link().callback(|_| Msg::Pause)}>
                    <i class="fas fa-pause"></i>
                    <span class="mui--hidden-xs">
                        { "Pause" }
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    disabled={!self.paused}
                    onclick={ctx.link().callback(|_| Msg::Reset)}>
                    <i class="fas fa-redo"></i>
                    <span class="mui--hidden-xs">
                        <abbr title="Reset the world.">
                            { "Reset" }
                        </abbr>
                    </span>
                </button>
                <div class="mui--visible-xs-block"></div>
                <button class="mui-btn mui-btn--raised"
                    disabled={!self.paused}
                    onclick={ctx.link().callback(|_| Msg::Save)}>
                    <i class="fas fa-save"></i>
                    <span class="mui--hidden-xs">
                        <abbr title="Save the search status in a json file.">
                            { "Save" }
                        </abbr>
                    </span>
                </button>
                <button class="mui-btn mui-btn--raised"
                    onclick={ctx.link().batch_callback(|_| {
                        click_button("load").unwrap();
                        None
                    })}>
                    <i class="fas fa-file-import"></i>
                    <span class="mui--hidden-xs">
                        <abbr title="Load the search status from a json file.">
                            { "Load" }
                        </abbr>
                    </span>
                </button>
                <input id="load"
                    type="file"
                    hidden=true
                    onchange={ctx.link().batch_callback(|e: Event| {
                        let input =  e.target()?.dyn_into::<HtmlInputElement>().ok()?;
                        input.files().map(Msg::Load)
                    })}/>
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
