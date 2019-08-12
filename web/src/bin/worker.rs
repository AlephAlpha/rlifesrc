use rlifesrc_web::worker::Worker;
use yew::agent::Threaded;

fn main() {
    yew::initialize();
    Worker::register();
    yew::run_loop();
}
