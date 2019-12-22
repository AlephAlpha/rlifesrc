use rlifesrc_web::Worker;
use yew::agent::Threaded;

fn main() {
    yew::initialize();
    Worker::register();
    yew::run_loop();
}
