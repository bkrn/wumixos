use cbv::{spin, Machine};


use std::env;

use std::fs::File;
use std::io::{stdin, Write};
use std::io::{BufReader, Read};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;


type Tty = BufReader<File>;

fn open_tty() -> Tty {
    let f = File::open("/dev/tty").expect("Could Not Open TTY");
    BufReader::new(f)
}

fn main() {
    let mut log = File::create("session.log").expect("Could not open log");
    let (client_sender, client_receiver) = channel();
    let (machine_sender, machine_receiver) = channel();
    let mut machine = Machine::new(machine_receiver, client_sender, &mut stdin());
    for instruction in env::args().skip(1) {
        log.write_all(instruction.bytes().collect::<Vec<u8>>().as_slice()).unwrap();
        for byte in instruction.bytes() {
            machine_sender
                .send(u32::from(byte))
                .expect("Machine channel closed during initialization");
        }
        machine_sender
            .send(10u32)
            .expect("Machine channel closed during initialization");
        log.write_all(&[10u8]).unwrap();
    }

    thread::spawn(move || loop {
        for b in open_tty().bytes() {
            let b = b.expect("Read error from stdin");
            log.write_all(&[b]).unwrap();
            if machine_sender
                .send(u32::from(b))
                .is_err()
            {
                break;
            };
        }
        thread::sleep(Duration::from_millis(10));
    });

    thread::spawn(move || {
        while let Ok(i) = client_receiver.recv() {
            print!("{}", i as u8 as char)
        }
    });

    while let Some(m) = spin(machine) {
        machine = m
    }
}
