use yew;
use yew::agent::Threaded;
use yew::worker::*;

fn main() {
    yew::initialize();
    cbv::webmachine::WebMachine::register();
    yew::run_loop();
}