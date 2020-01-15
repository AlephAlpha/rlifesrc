use rlifesrc_lib::{Config, Search, Status, WorldSer};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use yew::{
    agent::{Agent, AgentLink, HandlerId, Public},
    services::{Task, TimeoutService},
};

const VIEW_FREQ: u64 = 50000;

#[derive(Serialize, Deserialize)]
pub enum Request {
    Start,
    Pause,
    SetWorld(Config),
    DisplayGen(isize),
    Store,
    Restore(WorldSer),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    UpdateWorld((String, usize)),
    UpdateStatus(Status),
    UpdateConfig(Config),
    InvalidRule,
    Store(WorldSer),
}

pub enum WorkerMsg {
    Step,
}

pub struct Worker {
    status: Status,
    search: Box<dyn Search>,
    link: AgentLink<Worker>,
    timeout: TimeoutService,
    job: Option<Box<dyn Task>>,
}

impl Worker {
    fn start_job(&mut self) {
        let handle = self.timeout.spawn(
            Duration::from_millis(0),
            self.link.callback(|_| WorkerMsg::Step),
        );
        self.job = Some(Box::new(handle));
    }

    fn stop_job(&mut self) {
        if let Some(mut task) = self.job.take() {
            task.cancel();
        }
    }

    fn update_world(&mut self, id: HandlerId, gen: isize) {
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
    type Reach = Public;
    type Message = WorkerMsg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let config: Config = Config::default();
        let search = config.world().unwrap();
        let timeout = TimeoutService::new();

        Worker {
            status: Status::Initial,
            search,
            link,
            timeout,
            job: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            WorkerMsg::Step => {
                if let Status::Searching = self.status {
                    self.status = self.search.search(Some(VIEW_FREQ));
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
                self.status = Status::Paused;
                if let Ok(search) = config.world() {
                    self.search = search;
                    self.update_world(id, 0);
                } else {
                    self.link.respond(id, Response::InvalidRule);
                }
            }
            Request::DisplayGen(gen) => {
                self.update_world(id, gen);
            }
            Request::Store => {
                let world_ser = self.search.ser();
                self.link.respond(id, Response::Store(world_ser));
            }
            Request::Restore(world_ser) => {
                self.stop_job();
                self.status = Status::Paused;
                if let Ok(search) = world_ser.world() {
                    self.search = search;
                    self.link
                        .respond(id, Response::UpdateConfig(self.search.config().clone()));
                    self.update_world(id, 0);
                } else {
                    self.link.respond(id, Response::InvalidRule);
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.update_world(id, 0);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}
