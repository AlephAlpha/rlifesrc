use crate::search::rules::{parse_isotropic, parse_life};
use crate::search::world::State::Dead;
use crate::search::world::{Symmetry, World};
use crate::search::NewState::Choose;
use crate::search::{NewState, Search, Status, TraitSearch};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use yew::services::{Task, TimeoutService};
use yew::worker::*;
use yew::Callback;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Props {
    pub width: isize,
    pub height: isize,
    pub period: isize,
    pub dx: isize,
    pub dy: isize,
    pub symmetry: Symmetry,
    pub column_first: Option<bool>,
    pub new_state: NewState,
    pub rule_string: String,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            width: 7,
            height: 7,
            period: 3,
            dx: 0,
            dy: 0,
            symmetry: Symmetry::C1,
            column_first: None,
            new_state: Choose(Dead),
            rule_string: String::from("B3/S23"),
        }
    }
}

struct Job {
    timeout: TimeoutService,
    callback: Callback<()>,
    task: Option<Box<Task>>,
}

impl Job {
    fn new(link: &AgentLink<Worker>) -> Self {
        let timeout = TimeoutService::new();
        let callback = link.send_back(|_| Msg::Step);
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
    view_freq: usize,
    status: Status,
    search: Box<dyn TraitSearch>,
    link: AgentLink<Worker>,
    job: Job,
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    Start,
    Pause,
    SetWorld(Props),
    DisplayGen(isize),
}

impl Transferable for Request {}

#[derive(Serialize, Deserialize)]
pub struct WorldStatus {
    pub world: String,
    pub period: isize,
    pub status: Status,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    WorldStatus(WorldStatus),
    InvalidRule,
}

impl Transferable for Response {}

pub enum Msg {
    Step,
}

impl Worker {
    fn response_world_status(&mut self, id: HandlerId, gen: isize) {
        let world = self.search.display_gen(gen);
        let period = self.search.period();
        let status = self.status;
        let world_status = WorldStatus {
            world,
            period,
            status,
        };
        self.link.response(id, Response::WorldStatus(world_status));
    }
}

impl Agent for Worker {
    type Reach = Public;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let props: Props = Default::default();
        let rule = parse_life(&props.rule_string).unwrap();
        let world = World::new(
            (props.width, props.height, props.period),
            props.dx,
            props.dy,
            props.symmetry,
            rule,
            props.column_first,
        );
        let search = Box::new(Search::new(world, props.new_state));

        let view_freq = 10000;
        let status = Status::Paused;
        let job = Job::new(&link);

        Worker {
            view_freq,
            status,
            search,
            link,
            job,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Step => {
                if let Status::Searching = self.status {
                    self.status = self.search.search(Some(self.view_freq));
                    self.job.start();
                } else {
                    self.job.stop();
                }
            }
        }
    }

    fn handle(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::Start => {
                self.status = Status::Searching;
                self.job.start();
            }
            Request::Pause => {
                self.job.stop();
                self.status = Status::Paused;
            }
            Request::SetWorld(props) => {
                self.job.stop();
                self.status = Status::Paused;
                let dimensions = (props.width, props.height, props.period);
                if let Ok(rule) = parse_life(&props.rule_string) {
                    let world = World::new(
                        dimensions,
                        props.dx,
                        props.dy,
                        props.symmetry,
                        rule,
                        props.column_first,
                    );
                    self.search = Box::new(Search::new(world, props.new_state));
                    self.response_world_status(id, 0);
                } else if let Ok(rule) = parse_isotropic(&props.rule_string) {
                    let world = World::new(
                        dimensions,
                        props.dx,
                        props.dy,
                        props.symmetry,
                        rule,
                        props.column_first,
                    );
                    self.search = Box::new(Search::new(world, props.new_state));
                    self.response_world_status(id, 0);
                } else {
                    self.link.response(id, Response::InvalidRule);
                }
            }
            Request::DisplayGen(gen) => {
                self.response_world_status(id, gen);
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.response_world_status(id, 0);
    }

    fn name_of_resource() -> &'static str {
        "worker.js"
    }
}
