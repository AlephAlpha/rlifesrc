use ca_rules::ParseBSRules;
use rlifesrc_lib::rules::{isotropic, life};
use rlifesrc_lib::NewState::Choose;
use rlifesrc_lib::State::Dead;
use rlifesrc_lib::{NewState, Search, Status, Symmetry, TraitSearch, World};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use yew::services::{Task, TimeoutService};
use yew::worker::*;
use yew::Callback;
use yew::Properties;

// 这部份的很多写法是照抄 yew 自带的范例
// https://github.com/DenisKolodin/yew

#[derive(Clone, PartialEq, Properties, Serialize, Deserialize)]
pub struct Props {
    #[props(required)]
    pub width: isize,
    #[props(required)]
    pub height: isize,
    #[props(required)]
    pub period: isize,
    pub dx: isize,
    pub dy: isize,
    pub symmetry: Symmetry,
    #[props(required)]
    pub column_first: Option<bool>,
    #[props(required)]
    pub new_state: NewState,
    #[props(required)]
    pub max_cell_count: Option<u32>,
    #[props(required)]
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
            max_cell_count: None,
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
pub enum Response {
    UpdateWorld(String),
    UpdateStatus(Status),
    InvalidRule,
}

impl Transferable for Response {}

pub enum Msg {
    Step,
}

impl Worker {
    fn update_world(&mut self, id: HandlerId, gen: isize) {
        let world = self.search.display_gen(gen);
        self.link.response(id, Response::UpdateWorld(world));
        self.update_status(id);
    }

    fn update_status(&mut self, id: HandlerId) {
        let status = self.status;
        self.link.response(id, Response::UpdateStatus(status));
    }
}

impl Agent for Worker {
    type Reach = Public;
    type Message = Msg;
    type Input = Request;
    type Output = Response;

    fn create(link: AgentLink<Self>) -> Self {
        let props: Props = Default::default();
        let rule = life::Life::parse_rule(&props.rule_string).unwrap();
        let world = World::new(
            (props.width, props.height, props.period),
            props.dx,
            props.dy,
            props.symmetry,
            rule,
            props.column_first,
        );
        let search = Box::new(Search::new(world, props.new_state, props.max_cell_count));

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
                self.update_status(id);
                self.job.start();
            }
            Request::Pause => {
                self.job.stop();
                self.status = Status::Paused;
                self.update_status(id);
            }
            Request::SetWorld(props) => {
                self.job.stop();
                self.status = Status::Paused;
                let dimensions = (props.width, props.height, props.period);
                if let Ok(rule) = life::Life::parse_rule(&props.rule_string) {
                    let world = World::new(
                        dimensions,
                        props.dx,
                        props.dy,
                        props.symmetry,
                        rule,
                        props.column_first,
                    );
                    self.search =
                        Box::new(Search::new(world, props.new_state, props.max_cell_count));
                    self.update_world(id, 0);
                } else if let Ok(rule) = isotropic::Life::parse_rule(&props.rule_string) {
                    let world = World::new(
                        dimensions,
                        props.dx,
                        props.dy,
                        props.symmetry,
                        rule,
                        props.column_first,
                    );
                    self.search =
                        Box::new(Search::new(world, props.new_state, props.max_cell_count));
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
