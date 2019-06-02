use std::sync::mpsc::{channel, Receiver, Sender};

use http::request::Request as HttpRequest;
use http::response::Response as HttpResponse;

use crate::{spin, Machine};
use failure::Error;

use serde_derive::{Deserialize, Serialize};
use std::time::Duration;
use yew::services::{fetch::FetchTask, interval::IntervalTask, FetchService, IntervalService};
use yew::format::Nothing;
use yew::Callback;
use yew::worker::*;
use stdweb::web::Date;

struct MachineWrapper {
    to_machine: Sender<u32>,
    to_client: Receiver<u32>,
    machine: Machine,
}

impl MachineWrapper {
    fn run(mut self, iters: usize) -> Option<(usize, Self)> {
        let mut last_finger = 0;
        let mut cycles = 0;
        for _ in 0..iters {
            if let Some(machine) = spin(self.machine) {
                self.machine = machine;
                if self.machine.finger() == last_finger {
                    cycles -= 1;
                    return Some((cycles, self));
                }
            } else {
                return None;
            }
            cycles += 1;
            last_finger = self.machine.finger();
        }
        Some((cycles, self))
    }
}

pub struct WebMachine {
    link: AgentLink<WebMachine>,
    ticker: IntervalTask,
    machine: Option<MachineWrapper>,
    buffer: Vec<u32>,
    clock: usize,
    cycles: usize,
    media_fetcher: Option<FetchTask>,
    media_callback: Callback<HttpResponse<Result<Vec<u8>, Error>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    BootFrom(String),
    Input(Vec<u32>),
    Shutdown,
    Status,
}

impl Transferable for Request {}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    Status {
        finger: usize,
        halted: bool,
        cycles: usize,
        clock: usize,
        output: Vec<u32>,
    },
}

impl Transferable for Response {}

pub enum MachineMsg {
    Tick,
    BootAs(Vec<u8>),
}

impl WebMachine {
    fn set_clock(&mut self, start: f64) {
        let elapsed = Date::now() - start;
        if elapsed > 90.0 || elapsed < 80.0 {
            self.clock = (self.clock as f64 * (100.0 / (elapsed + 10.0))) as usize;
        }
    }
}

impl Agent for WebMachine {

    type Reach = Public;
    type Message = MachineMsg;
    type Input = Request;
    type Output = Response;

    // Create an instance with a link to agent's environment.
    fn create(link: AgentLink<Self>) -> Self {
        let media_callback = link.send_back(|response: HttpResponse<Result<Vec<u8>, Error>>| {
            let body = response.body().as_ref();
            MachineMsg::BootAs(body.map(|a| a.clone()).unwrap_or_default())
        });
        let ticker = IntervalService::new().spawn(
            Duration::from_millis(100),
            link.send_back(|_| MachineMsg::Tick),
        );
        WebMachine {
            link,
            ticker,
            machine: None,
            buffer: Vec::new(),
            clock: 100_000,
            cycles: 0,
            media_fetcher: None,
            media_callback,
        }
    }

    // Handle inner messages (of services of `send_back` callbacks)
    fn update(&mut self, msg: Self::Message) {
        match msg {
            MachineMsg::Tick => {
                if let Some(wrapper) = self.machine.take() {
                    let start = Date::now();
                    if let Some((cycles, wrapper)) = wrapper.run(self.clock) {
                        if cycles == self.clock {
                            self.set_clock(start);
                        }
                        self.cycles += cycles;
                        while let Ok(i) = wrapper.to_client.try_recv() {
                            self.buffer.push(i);
                        }
                        self.machine = Some(wrapper);
                    }
                };
            }
            MachineMsg::BootAs(u) => {
                self.media_fetcher = None;
                let (client_sender, to_client) = channel();
                let (to_machine, machine_receiver) = channel();
                let machine = Machine::new(machine_receiver, client_sender, &mut u.as_slice());
                self.cycles = 0;
                self.buffer = Vec::new();
                self.machine = Some(MachineWrapper {
                    machine,
                    to_machine,
                    to_client,
                });
            }
        }
    }

    // Handle incoming messages from components of other agents.
    fn handle(&mut self, msg: Self::Input, who: HandlerId) {
        match msg {
            Request::Status => self.link.response(
                who,
                Response::Status {
                    finger: self
                        .machine
                        .as_ref()
                        .map(|w| w.machine.finger())
                        .unwrap_or_default(),
                    halted: self.machine.is_none(),
                    cycles: self.cycles,
                    clock: self.clock,
                    output: self.buffer.drain(0..).collect(),
                },
            ),
            Request::Input(u) => {
                if let Some(wrapper) = self.machine.as_ref() {
                    for v in u {
                        wrapper.to_machine.send(v).unwrap();

                    }
                }
            }

            Request::BootFrom(url) => {
                if self.media_fetcher.is_none() {
                    self.cycles = 0;
                    self.buffer = Vec::new();
                    self.machine = None;
                    let req = HttpRequest::get(url).body(Nothing).unwrap();
                    self.media_fetcher =
                        Some(FetchService::new().fetch_binary(req, self.media_callback.clone()));
                }
            },
            Request::Shutdown => {
                self.cycles = 0;
                self.buffer = Vec::new();
                self.machine = None;
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "machine/machine.js"
    }
}