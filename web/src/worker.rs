use rlifesrc_lib::{save::WorldSer, Config, Search, Status};
use serde::{Deserialize, Serialize};
use std::{option_env, time::Duration};
use yew::{
    agent::{Agent, AgentLink, HandlerId, Public},
    services::timeout::{TimeoutService, TimeoutTask},
};

const VIEW_FREQ: u64 = 50000;

#[derive(Serialize, Deserialize)]
pub enum Request {
    Start,
    Pause,
    SetWorld(Config),
    DisplayGen(i32),
    MaxPartial,
    Save,
    Load(WorldSer),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    UpdateWorld((String, usize)),
    UpdateStatus(Status),
    UpdateConfig(Config),
    Error(String),
    Save(WorldSer),
}

pub enum WorkerMsg {
    Step,
}

pub struct Worker {
    status: Status,
    search: Box<dyn Search>,
    max_partial_count: usize,
    max_partial: String,
    link: AgentLink<Worker>,
    timeout_task: Option<TimeoutTask>,
}

impl Worker {
    fn start_job(&mut self) {
        let handle = TimeoutService::spawn(
            Duration::from_millis(0),
            self.link.callback(|_| WorkerMsg::Step),
        );
        self.timeout_task = Some(handle);
    }

    fn stop_job(&mut self) {
        self.timeout_task.take();
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

    fn update_world(&mut self, id: HandlerId, gen: i32) {
        let world = self.search.rle_gen(gen);
        let count = self.search.cell_count_gen(gen);
        self.link.respond(id, Response::UpdateWorld((world, count)));
        self.update_status(id);
    }

    fn update_status(&mut self, id: HandlerId) {
        let status = self.status;
        if Status::Found == status && self.search.config().reduce_max {
            self.link
                .respond(id, Response::UpdateConfig(self.search.config().clone()));
        }
        self.link.respond(id, Response::UpdateStatus(status));
    }
}

impl Agent for Worker {
    type Reach = Public<Self>;
    type Message = WorkerMsg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let config: Config = Config::default();
        let search = config.world().unwrap();

        let mut worker = Worker {
            status: Status::Initial,
            search,
            max_partial_count: 0,
            max_partial: String::new(),
            link,
            timeout_task: None,
        };
        worker.update_max_martial(false);
        worker
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            WorkerMsg::Step => {
                if let Status::Searching = self.status {
                    self.status = self.search.search(Some(VIEW_FREQ));
                    self.update_max_martial(true);
                    self.start_job();
                } else {
                    self.stop_job();
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Start => {
                self.status = Status::Searching;
                self.update_status(id);
                self.start_job();
            }
            Request::Pause => {
                self.stop_job();
                self.status = Status::Paused;
                self.update_status(id);
            }
            Request::SetWorld(config) => {
                self.stop_job();
                self.status = Status::Initial;
                match config.world() {
                    Ok(search) => {
                        self.search = search;
                        self.update_max_martial(false);
                        self.update_world(id, 0);
                    }
                    Err(error) => {
                        let message = error.to_string();
                        self.link.respond(id, Response::Error(message));
                    }
                }
            }
            Request::DisplayGen(gen) => {
                self.update_world(id, gen);
            }
            Request::MaxPartial => {
                self.link.respond(
                    id,
                    Response::UpdateWorld((self.max_partial.clone(), self.max_partial_count)),
                );
                self.update_status(id);
            }
            Request::Save => {
                let world_ser = self.search.ser();
                self.link.respond(id, Response::Save(world_ser));
            }
            Request::Load(world_ser) => {
                self.stop_job();
                self.status = Status::Paused;
                match world_ser.world() {
                    Ok(search) => {
                        self.search = search;
                        self.update_max_martial(false);
                        self.link
                            .respond(id, Response::UpdateConfig(self.search.config().clone()));
                        self.update_world(id, 0);
                    }
                    Err(error) => {
                        let message = error.to_string();
                        self.link.respond(id, Response::Error(message));
                    }
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.update_world(id, 0);
    }

    fn name_of_resource() -> &'static str {
        option_env!("RLIFESRC_PATH").unwrap_or("rlifesrc/worker.js")
    }
}
