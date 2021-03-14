use rlifesrc_web::Worker;
use yew::agent::Threaded;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::initialize();
    Worker::register();
    yew::run_loop();
}
