// #[cfg(feature = "stdweb")]
// #[macro_use]
// extern crate stdweb;


use std::sync::mpsc::{Receiver, Sender};

#[cfg(feature = "yew")]
pub mod webmachine;

type Stacks = Vec<Vec<u32>>;

#[derive(Debug, PartialEq)]
struct Pointers {
    a: usize,
    b: usize,
    c: usize,
}

impl Pointers {
    fn new(i: u32) -> Self {
        Pointers {
            c: (i & 7) as usize,
            b: ((i >> 3) & 7) as usize,
            a: ((i >> 6) & 7) as usize,
        }
    }
}

#[derive(Debug, PartialEq)]
struct OrthoPointers {
    a: usize,
    value: u32,
}

impl OrthoPointers {
    fn new(i: u32) -> Self {
        OrthoPointers {
            a: ((i >> 25) & 7) as usize,
            value: i & 0x1FF_FFFF,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Instruction {
    Move(Pointers),
    Index(Pointers),
    Amend(Pointers),
    Add(Pointers),
    Mul(Pointers),
    Div(Pointers),
    Nand(Pointers),
    Halt(Pointers),
    Allocate(Pointers),
    Abandon(Pointers),
    Out(Pointers),
    In(Pointers),
    Load(Pointers),
    Ortho(OrthoPointers),
}

use self::Instruction::*;

pub struct Machine {
    fin: usize,
    reg: [u32; 8],
    stacks: Stacks,
    available: Vec<usize>,

    inbox: Receiver<u32>,
    outbox: Sender<u32>,
}

impl Machine {

    pub fn finger(&self) -> usize {
        self.fin
    }

    fn advance(&mut self) -> Instruction {
        let instruction = self.instruction();
        self.fin += 1;
        instruction
    }

    fn instruction(&self) -> Instruction {
        self.stacks
            .get(0)
            .map(|scroll| scroll[self.fin])
            .expect("Finger or stack invalid")
            .into()
    }

    fn insert_stack(&mut self, stack_length: usize) -> u32 {
        if let Some(key) = self.available.pop() {
            self.stacks[key] = vec![0; stack_length];
            key as u32
        } else {
            let key = self.stacks.len();
            self.stacks.push(vec![0; stack_length]);
            key as u32
        }
    }

    pub fn new(r: Receiver<u32>, s: Sender<u32>, scroll: &mut std::io::Read) -> Self {
        Self {
            fin: 0,
            reg: [0; 8],
            stacks: vec![read_scroll(scroll)],
            available: Vec::new(),
            inbox: r,
            outbox: s,
        }
    }
}

impl From<u32> for Instruction {
    fn from(i: u32) -> Self {
        match i >> 28 {
            0 => Instruction::Move(Pointers::new(i)),
            1 => Instruction::Index(Pointers::new(i)),
            2 => Instruction::Amend(Pointers::new(i)),
            3 => Instruction::Add(Pointers::new(i)),
            4 => Instruction::Mul(Pointers::new(i)),
            5 => Instruction::Div(Pointers::new(i)),
            6 => Instruction::Nand(Pointers::new(i)),
            7 => Instruction::Halt(Pointers::new(i)),
            8 => Instruction::Allocate(Pointers::new(i)),
            9 => Instruction::Abandon(Pointers::new(i)),
            10 => Instruction::Out(Pointers::new(i)),
            11 => Instruction::In(Pointers::new(i)),
            12 => Instruction::Load(Pointers::new(i)),
            13 => Instruction::Ortho(OrthoPointers::new(i)),
            _ => panic!("Bad Instruction"),
        }
    }
}

fn as_u32(word: [u8; 4]) -> u32 {
    word.iter()
        .enumerate()
        .map(|(i, b)| u32::from(*b) << ((3 - i) * 8))
        .sum()
}

fn read_scroll(r: &mut std::io::Read) -> Vec<u32> {
    let mut scroll: Vec<u32> = Vec::new();
    let mut word: [u8; 4] = [0; 4];
    while let Ok(_) = r.read_exact(&mut word) {
        scroll.push(as_u32(word));
    }
    scroll
}

static NULL_STACK_ERR: &'static str = "Attempted operation on unallocated stack";

#[cfg(feature = "yew")]
fn read_byte(machine: &mut Machine) -> Option<u32> {
    machine.inbox.try_recv().ok()
}

#[cfg(not(feature = "yew"))]
fn read_byte(machine: &mut Machine) -> Option<u32> {
    Some(machine.inbox.recv().unwrap())
}

pub fn spin(mut machine: Machine) -> Option<Machine> {
    match machine.advance() {
        Move(Pointers { a, b, c }) => {
            machine.reg[a] = if machine.reg[c] > 0 {
                machine.reg[b]
            } else {
                machine.reg[a]
            }
        }
        Index(Pointers { a, b, c }) => {
            machine.reg[a] = machine
                .stacks
                .get(machine.reg[b] as usize)
                .expect(NULL_STACK_ERR)[machine.reg[c] as usize]
        }
        Amend(Pointers { a, b, c }) => {
            machine
                .stacks
                .get_mut(machine.reg[a] as usize)
                .expect(NULL_STACK_ERR)[machine.reg[b] as usize] = machine.reg[c]
        }
        Add(Pointers { a, b, c }) => machine.reg[a] = machine.reg[b].wrapping_add(machine.reg[c]),
        Mul(Pointers { a, b, c }) => machine.reg[a] = machine.reg[b].wrapping_mul(machine.reg[c]),
        Div(Pointers { a, b, c }) => machine.reg[a] = machine.reg[b].wrapping_div(machine.reg[c]),
        Nand(Pointers { a, b, c }) => machine.reg[a] = !(machine.reg[b] & machine.reg[c]),
        Halt(_) => {
            return None;
        }
        Allocate(Pointers { b, c, .. }) => {
            machine.reg[b] = machine.insert_stack(machine.reg[c] as usize);
        }
        Abandon(Pointers { c, .. }) => {
            machine.available.push(machine.reg[c] as usize);
        }
        Out(Pointers { c, .. }) => {
            machine.outbox.send(machine.reg[c]).expect("Output channel closed")
        }
        In(Pointers { c, .. }) => {
            if let Some(b) = read_byte(&mut machine) {
                machine.reg[c] = b;
            } else {
                machine.fin -= 1;
            }
        },
        Load(Pointers { b, c, .. }) => {
            if machine.reg[b] > 0 {
                machine.stacks[0] = machine
                    .stacks
                    .get(machine.reg[b] as usize)
                    .expect(NULL_STACK_ERR)
                    .clone();
            }
            machine.fin = machine.reg[c] as usize;
        }
        Ortho(OrthoPointers { a, value }) => machine.reg[a] = value,
    };
    Some(machine)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction() {
        let target = Instruction::Add(Pointers { a: 7, b: 6, c: 0 });
        let other: Instruction = 0b0011_0000_0000_0000_0000_0001_1111_0000.into();
        assert_eq!(target, other);
    }
}
