use rlifesrc_lib::{
    rules::{Life, NtLife},
    NewState::{self, Choose},
    Search,
    State::Alive,
    Status, Symmetry, TraitSearch, Transform, World,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use yew::{
    services::{Task, TimeoutService},
    worker::*,
    Callback,
};

const VIEW_FREQ: u32 = 20000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub width: isize,
    pub height: isize,
    pub period: isize,
    pub dx: isize,
    pub dy: isize,
    pub transform: Transform,
    pub symmetry: Symmetry,
    pub column_first: Option<bool>,
    pub new_state: NewState,
    pub max_cell_count: Option<u32>,
    pub rule_string: String,
}

impl Default for Data {
    fn default() -> Self {
        Data {
            width: 26,
            height: 8,
            period: 4,
            dx: 0,
            dy: 1,
            transform: Transform::Id,
            symmetry: Symmetry::C1,
            column_first: None,
            new_state: Choose(Alive),
            max_cell_count: None,
            rule_string: String::from("B3/S23"),
        }
    }
}

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
    search: Box<dyn TraitSearch>,
    link: AgentLink<Worker>,
    job: Job,
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    Start,
    Pause,
    SetWorld(Data),
    DisplayGen(isize),
}

impl Transferable for Request {}

#[derive(Serialize, Deserialize)]
pub enum Response {
    UpdateWorld((String, u32)),
    UpdateStatus(Status),
    InvalidRule,
}

impl Transferable for Response {}

pub struct Step;

impl Worker {
    fn update_world(&mut self, id: HandlerId, gen: isize) {
        let world = self.search.display_gen(gen);
        let count = self.search.cell_count();
        self.link
            .response(id, Response::UpdateWorld((world, count)));
        self.update_status(id);
    }

    fn update_status(&mut self, id: HandlerId) {
        let status = self.status;
        self.link.response(id, Response::UpdateStatus(status));
    }
}

impl Agent for Worker {
    type Reach = Public;
    type Message = Step;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let data: Data = Default::default();
        let rule = Life::parse_rule(&data.rule_string).unwrap();
        let world = World::new(
            (data.width, data.height, data.period),
            data.dx,
            data.dy,
            data.transform,
            data.symmetry,
            rule,
            data.column_first,
        );
        let search = Box::new(Search::new(world, data.new_state, data.max_cell_count));

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
            Request::SetWorld(data) => {
                self.job.stop();
                self.status = Status::Paused;
                let dimensions = (data.width, data.height, data.period);
                if let Ok(rule) = Life::parse_rule(&data.rule_string) {
                    let world = World::new(
                        dimensions,
                        data.dx,
                        data.dy,
                        data.transform,
                        data.symmetry,
                        rule,
                        data.column_first,
                    );
                    self.search = Box::new(Search::new(world, data.new_state, data.max_cell_count));
                    self.update_world(id, 0);
                } else if let Ok(rule) = NtLife::parse_rule(&data.rule_string) {
                    let world = World::new(
                        dimensions,
                        data.dx,
                        data.dy,
                        data.transform,
                        data.symmetry,
                        rule,
                        data.column_first,
                    );
                    self.search = Box::new(Search::new(world, data.new_state, data.max_cell_count));
                    self.update_world(id, 0);
                } else {
                    self.link.response(id, Response::InvalidRule);
                }
            }
            Request::DisplayGen(gen) => {
                self.update_world(id, gen);
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
