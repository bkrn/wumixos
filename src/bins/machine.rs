use yew;
use yew::agent::Threaded;

fn main() {
    yew::initialize();
    cbv::webmachine::WebMachine::register();
    yew::run_loop();
}