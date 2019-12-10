use rlifesrc_lib::{Config, Search, Status, WorldSer};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use yew::{
    services::{Task, TimeoutService},
    worker::*,
    Callback,
};

const VIEW_FREQ: u64 = 50000;

struct Job {
    timeout: TimeoutService,
    callback: Callback<()>,
    task: Option<Box<dyn Task>>,
}

impl Job {
    fn new(link: &AgentLink<Worker>) -> Self {
        let timeout = TimeoutService::new();
        let callback = link.send_back(|_| Step);
        let task = None;
        Job {
            timeout,
            callback,
            task,
        }
    }

    fn start(&mut self) {
        let handle = self
            .timeout
            .spawn(Duration::from_millis(0), self.callback.clone());
        self.task = Some(Box::new(handle));
    }

    fn stop(&mut self) {
        if let Some(mut task) = self.task.take() {
            task.cancel();
        }
    }
}

pub struct Worker {
    status: Status,
    search: Box<dyn Search>,
    link: AgentLink<Worker>,
    job: Job,
}

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

pub struct Step;

impl Worker {
    fn update_world(&mut self, id: HandlerId, gen: isize) {
        let world = self.search.display_gen(gen);
        let count = self.search.cell_count_gen(gen);
        self.link
            .response(id, Response::UpdateWorld((world, count)));
        self.update_status(id);
    }

    fn update_status(&mut self, id: HandlerId) {
        let status = self.status;
        if Status::Found == status && self.search.config().reduce_max {
            self.link
                .response(id, Response::UpdateConfig(self.search.config()));
        }
        self.link.response(id, Response::UpdateStatus(status));
    }
}

impl Agent for Worker {
    type Reach = Public;
    type Message = Step;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let config: Config = Default::default();
        let search = config.world().unwrap();

        let status = Status::Paused;
        let job = Job::new(&link);

        Worker {
            status,
            search,
            link,
            job,
        }
    }

    fn update(&mut self, _msg: Self::Message) {
        if let Status::Searching = self.status {
            self.status = self.search.search(Some(VIEW_FREQ));
            self.job.start();
        } else {
            self.job.stop();
        }
    }

    fn handle(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Start => {
                self.status = Status::Searching;
                self.update_status(id);
                self.job.start();
            }
            Request::Pause => {
                self.job.stop();
                self.status = Status::Paused;
                self.update_status(id);
            }
            Request::SetWorld(config) => {
                self.job.stop();
                self.status = Status::Paused;
                if let Ok(search) = config.world() {
                    self.search = search;
                    self.update_world(id, 0);
                } else {
                    self.link.response(id, Response::InvalidRule);
                }
            }
            Request::DisplayGen(gen) => {
                self.update_world(id, gen);
            }
            Request::Store => {
                let world_ser = self.search.ser();
                self.link.response(id, Response::Store(world_ser));
            }
            Request::Restore(world_ser) => {
                self.job.stop();
                self.status = Status::Paused;
                if let Ok(search) = world_ser.world() {
                    self.search = search;
                    self.link
                        .response(id, Response::UpdateConfig(self.search.config()));
                    self.update_world(id, 0);
                } else {
                    self.link.response(id, Response::InvalidRule);
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
