use instant::Instant;
use log::{debug, error};
use rlifesrc_lib::{save::WorldSer, Config, Search, Status};
use serde::{Deserialize, Serialize};
use std::{option_env, time::Duration};
use yew::{
    agent::{Agent, AgentLink, HandlerId, Public},
    services::timeout::{TimeoutService, TimeoutTask},
};

const VIEW_FREQ: u64 = 100000;

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Start,
    Pause,
    SetWorld(Config),
    SetFindAll(bool),
    DisplayGen(i32),
    MaxPartial,
    Save,
    Load(WorldSer),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Update(UpdateMessage),
    Error(String),
    Save(WorldSer),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub world: Option<String>,
    pub cells: Option<u32>,
    pub status: Status,
    pub paused: bool,
    pub found_count: u32,
    pub timing: Option<Duration>,
    pub config: Option<Config>,
}

#[derive(Debug)]
pub enum WorkerMsg {
    Step,
}

pub struct Worker {
    status: Status,
    paused: bool,
    search: Box<dyn Search>,
    max_partial_count: u32,
    max_partial: String,
    find_all: bool,
    found_count: u32,
    all_found: Vec<String>,
    start_time: Option<Instant>,
    timing: Duration,
    link: AgentLink<Worker>,
    timeout_task: Option<TimeoutTask>,
}

impl Worker {
    fn start_job(&mut self) {
        self.paused = false;
        let handle = TimeoutService::spawn(
            Duration::from_millis(0),
            self.link.callback(|_| WorkerMsg::Step),
        );
        self.timeout_task = Some(handle);
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
    }

    fn stop_job(&mut self) {
        self.timeout_task.take();
        if let Some(instant) = self.start_time.take() {
            self.timing += instant.elapsed();
            self.start_time = Some(Instant::now());
        }
        self.paused = true;
    }

    fn reset_world(&mut self, search: Box<dyn Search>) {
        self.search = search;
        self.status = Status::Initial;
        self.update_max_martial(false);
        self.timing = Duration::default();
        self.found_count = 0;
        self.all_found = vec![String::new(); self.search.config().period as usize];
    }

    fn update_max_martial(&mut self, check_max: bool) {
        let (gen, cell_count) = (0..self.search.config().period)
            .map(|t| (t, self.search.cell_count_gen(t)))
            .max_by_key(|p| p.1)
            .unwrap();
        if !check_max || cell_count > self.max_partial_count {
            self.max_partial_count = cell_count;
            self.max_partial = self.search.rle_gen(gen);
        }
    }

    fn update_message(&self) -> UpdateMessageBuilder<'_> {
        let status = self.status;
        let paused = self.paused;
        let config = (status == Status::Found && self.search.config().reduce_max)
            .then(|| self.search.config().clone());
        let timing = self.paused.then(|| self.timing);
        let found_count = self.found_count;

        let msg = UpdateMessage {
            world: None,
            cells: None,
            status,
            paused,
            found_count,
            timing,
            config,
        };
        UpdateMessageBuilder { msg, worker: self }
    }
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = WorkerMsg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        debug!("Worker path: {}", Self::name_of_resource());
        let config: Config = Config::default();
        let search = config.world().unwrap();
        let all_found = vec![String::new(); config.period as usize];

        let mut worker = Worker {
            status: Status::Initial,
            paused: true,
            search,
            max_partial_count: 0,
            max_partial: String::new(),
            find_all: false,
            found_count: 0,
            all_found,
            start_time: None,
            timing: Duration::default(),
            link,
            timeout_task: None,
        };
        worker.update_max_martial(false);
        worker
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            WorkerMsg::Step => {
                self.status = self.search.search(Some(VIEW_FREQ));
                self.update_max_martial(true);
                match self.status {
                    Status::Searching => self.start_job(),
                    Status::Found => {
                        self.found_count += 1;
                        for gen in 0..self.search.config().period {
                            self.all_found[gen as usize].push_str(&self.search.rle_gen(gen));
                        }
                        if self.find_all {
                            self.start_job();
                        } else {
                            self.stop_job();
                        }
                    }
                    _ => self.stop_job(),
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Start => {
                self.start_job();
            }
            Request::Pause => {
                self.stop_job();
            }
            Request::SetWorld(config) => {
                self.stop_job();
                match config.world() {
                    Ok(search) => {
                        self.reset_world(search);
                        self.update_message().with_config().with_world(0).send(id);
                    }
                    Err(error) => {
                        let message = error.to_string();
                        error!("Error setting world: {}", message);
                        self.link.respond(id, Response::Error(message));
                    }
                }
            }
            Request::SetFindAll(find_all) => {
                self.find_all = find_all;
            }
            Request::DisplayGen(gen) => {
                self.update_message().with_world(gen).send(id);
            }
            Request::MaxPartial => {
                self.update_message().with_max_partial().send(id);
            }
            Request::Save => {
                let world_ser = self.search.ser();
                self.link.respond(id, Response::Save(world_ser));
            }
            Request::Load(world_ser) => {
                self.stop_job();
                match world_ser.world() {
                    Ok(search) => {
                        debug!("Save file loaded!");
                        self.reset_world(search);
                        self.update_message().with_config().with_world(0).send(id);
                    }
                    Err(error) => {
                        let message = error.to_string();
                        error!("Error loading save file: {}", message);
                        self.link.respond(id, Response::Error(message));
                    }
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.update_message().with_config().with_world(0).send(id);
    }

    fn name_of_resource() -> &'static str {
        option_env!("RLIFESRC_PATH").unwrap_or("rlifesrc/worker.js")
    }
}

struct UpdateMessageBuilder<'a> {
    msg: UpdateMessage,
    worker: &'a Worker,
}

impl<'a> UpdateMessageBuilder<'a> {
    fn with_config(mut self) -> Self {
        if self.msg.config.is_none() {
            self.msg.config = Some(self.worker.search.config().clone());
        }
        self
    }

    fn with_world(mut self, gen: i32) -> Self {
        self.msg.world =
            if self.worker.find_all && self.worker.paused && self.worker.found_count > 0 {
                Some(self.worker.all_found[gen as usize].clone())
            } else {
                Some(self.worker.search.rle_gen(gen))
            };
        self.msg.cells = Some(self.worker.search.cell_count_gen(gen));
        self
    }

    fn with_max_partial(mut self) -> Self {
        self.msg.world = Some(self.worker.max_partial.clone());
        self.msg.cells = Some(self.worker.max_partial_count);
        self
    }

    fn send(self, id: HandlerId) {
        self.worker.link.respond(id, Response::Update(self.msg));
    }
}
