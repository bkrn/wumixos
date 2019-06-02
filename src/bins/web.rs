#![recursion_limit = "128"]

#[macro_use]
extern crate stdweb;

use stdweb::traits::IKeyboardEvent;

use cbv::webmachine::{Request, Response, WebMachine};

use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

use yew::services::{interval::IntervalTask, ConsoleService, IntervalService};
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

use yew::worker::*;

enum MachineState {
    Booting,
    Running,
    Halted,
    Blocking,
}

impl MachineState {
    fn class(&self) -> String {
        use MachineState::*;
        match self {
            Booting => "led-yellow",
            Running => "led-green",
            Halted => "led-red",
            Blocking => "led-blue",
        }
        .into()
    }
}

struct BootMedia {
    url: String,
    name: String,
    selected: bool,
}

struct Model {
    machine: Box<Bridge<WebMachine>>,
    ticker: IntervalTask,
    terminal: String,
    console: ConsoleService,
    machine_state: MachineState,
    cycles: usize,
    clock: usize,
    finger: usize,
    text: String,
    boot_media: Vec<BootMedia>,
    have_written: bool,
}

impl Model {
    fn write(&mut self, s: &str) {
        self.terminal += s;
        self.have_written = !s.is_empty();
    }
}

enum Msg {
    Ignore,
    Tick,
    Machine(Response),
    SendText,
    UpdateText(String),
    FetchMedia(String),
}

impl Component for Model {
    // Some details omitted. Explore the examples to see more.

    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let ticker =
            IntervalService::new().spawn(Duration::from_millis(200), link.send_back(|_| Msg::Tick));

        let callback = link.send_back(|d| Msg::Machine(d));
        let machine = WebMachine::bridge(callback);

        Model {
            machine,
            ticker,
            terminal: String::new(),
            console: ConsoleService::new(),
            machine_state: MachineState::Booting,
            cycles: 0,
            clock: 0,
            finger: 0,
            text: String::new(),
            have_written: false,
            boot_media: vec![
                BootMedia {
                    url: "/media/umix_os.um".into(),
                    name: "Umix OS".into(),
                    selected: false,
                },
                BootMedia {
                    url: "/media/sandmark.umz".into(),
                    name: "Sandmark".into(),
                    selected: false,
                },
            ],
        }
    }


    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::UpdateText(s) => {
                self.text = s;
                true
            }
            Msg::SendText => {
                self.write(&format!("{}\n", &self.text));
                self.machine.send(Request::Input(
                    self.text
                        .bytes()
                        .chain(vec![10u8].into_iter())
                        .map(u32::from)
                        .collect(),
                ));
                self.text = String::new();
                true
            }
            Msg::Tick => {
                self.machine.send(Request::Status);
                if self.have_written {
                    js!{
                        let objDiv = document.getElementById("terminal");
                        console.log(objDiv);
                        objDiv.scrollTop = objDiv.scrollHeight;
                    }
                    self.have_written = false;
                }
                false
            }
            Msg::FetchMedia(url) => {
                self.terminal = String::new();
                self.machine.send(Request::BootFrom(url));
                false
            }
            Msg::Machine(Response::Status {
                finger,
                halted,
                clock,
                cycles,
                output,
            }) => {
                self.machine_state = if halted {
                    MachineState::Halted
                } else if finger != self.finger {
                    MachineState::Running
                } else {
                    MachineState::Blocking
                };
                self.clock = clock;
                self.finger = finger;
                self.cycles = cycles;
                let mut term_update: String = output.into_iter().map(|u| u as u8 as char).collect();
                if self.terminal.is_empty() {
                    term_update = term_update.trim_start().into();
                }
                if !term_update.is_empty() {
                    self.write(&term_update);
                }
                true
            }
            Msg::Ignore => false,
        }
    }
}

fn hex_counter(value: usize) -> Html<Model> {
    let mut value = format!("{:0>8}", format!("{:x}", value)).to_uppercase();
    html! {
        <>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        <div class="counter",>{value.remove(0)}</div>
        </>
    }
}

fn digit_counter(value: usize) -> Html<Model> {
    let size = (value as f64).log10() as usize;
    let unit = match size / 3 {
        1 => 'k',
        2 => 'M',
        3 => 'G',
        4 => 'T',
        5 => 'P',
        6 => 'E',
        7 => 'Z',
        8 => 'Y',
        _ => '_',

    };
    let mut number = format!("{:0>3}", value / usize::pow(10, (size - (size % 3)) as u32));
    html! {
        <>
        <div class="counter",>{number.remove(0)}</div>
        <div class="counter",>{number.remove(0)}</div>
        <div class="counter",>{number.remove(0)}</div>
        <div class="counter",>{unit}</div>
        </>
    }
}


impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let media_view = move |media: &BootMedia| -> Html<Self> {
            let url = media.url.clone();
            html!(
                <div class="floppy red", onclick=|_| Msg::FetchMedia(url.clone()), >{media.name.clone()}</div>
            )
        };
        html! {
            <>
            <div class="led-box boot",>
                <div class={self.machine_state.class()},></div>
            </div>
            <div class="led-box script",>
                <div class={self.machine_state.class()},></div>
            </div>
            <div class="container",>
                
                <div class="term-container",>
                    <pre class="term-box",>< pre class="terminal", id="terminal", > {&self.terminal} </pre></pre>
                </div>
                <div class="machine-container",>
                    <input class="term-input",
                            type="text", 
                            value=&self.text, 
                            oninput=|input| Msg::UpdateText(input.value), 
                            onkeyup=|ev| if ev.key() == "Enter" {Msg::SendText} else {Msg::Ignore},></input>
                    
                    <div class="indicator",>
                        <h4>{"CYCLES PER 100mS"}</h4>
                        { digit_counter(self.clock) }
                    </div>
                    <div class="indicator",>
                        <h4>{"CYCLE COUNT"}</h4>
                        { digit_counter(self.cycles) }
                    </div>
                    <div class="indicator", style="display:block;", >
                        <h4>{"FINGER LOCATION"}</h4>
                        { hex_counter(self.finger) }
                    </div>
                </div>
                <div class="storage-container",>
                    <h4>{"MEDIA"}</h4>
                    {for self.boot_media.iter().map(media_view)}
                </div>
            </div>
            //<div class="background",/>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}