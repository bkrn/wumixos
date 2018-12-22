use std::collections::HashMap;
use std::io::{self, Read, BufRead, BufReader, Write};
use std::process::exit;
use std::{thread, time};
use std::fs::{File};

type Stacks = HashMap<u32, Vec<u32>>;

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

struct Machine {
    fin: usize,
    reg: [u32; 8],
    stacks: Stacks,
    buffer: Vec<u8>,
    session: Vec<u8>,
}

impl Machine {
    fn advance(&mut self) -> Instruction {
        let instruction = self.instruction();
        self.fin = self.fin + 1;
        instruction
    }

    fn instruction(&self) -> Instruction {
        self.stacks
            .get(&0)
            .map(|scroll| scroll[self.fin])
            .expect("Finger or stack invalid")
            .into()
    }

    fn insert_stack(&mut self, stack: Vec<u32>) -> u32 {
        let mut key = 1;
        while self.stacks.contains_key(&key) {
            key = key + 1;
        }
        self.stacks.insert(key, stack);
        key
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
        .map(|(i, b)| (*b as u32) << ((3 - i) * 8))
        .sum()
}

fn get_scroll() -> Vec<u32> {
    let mut scroll: Vec<u32> = Vec::new();
    let mut stdin = io::stdin();
    let mut word: [u8; 4] = [0; 4];
    while let Ok(_) = stdin.read_exact(&mut word) {
        scroll.push(as_u32(word));
    }
    scroll
}

fn main() {
    let mut stacks: HashMap<u32, Vec<u32>> = HashMap::new();
    stacks.insert(0, get_scroll());
    let mut machine = Machine {
        fin: 0,
        reg: [0; 8],
        stacks: stacks,
        buffer: Vec::new(),
        session: Vec::new(),
    };
    loop {
        machine = spin(machine);
    }
}

static NULL_STACK_ERR: &'static str = "Attempted operation on unallocated stack";

type Tty = BufReader<File>;

fn open_tty() -> Tty {
    let f = File::open("/dev/tty")
        .expect("Could Not Open TTY");
    BufReader::new(f)
}

fn read_byte(machine: &mut Machine) -> u32 {
    if machine.buffer.len() == 0 {
        open_tty().read_until(10, &mut machine.buffer);
        machine.buffer.reverse();
    }
    machine.buffer.pop().expect("No input read") as u32
}

fn spin(mut machine: Machine) -> Machine {
    match machine.advance() {
        Move(Pointers { a, b, c }) => {
            machine.reg[a] = if machine.reg[c] > 0 {
                machine.reg[b]
            } else {
                machine.reg[a]
            }
        }
        Index(Pointers { a, b, c }) => {
            machine.reg[a] =
                machine.stacks.get(&machine.reg[b]).expect(NULL_STACK_ERR)[machine.reg[c] as usize]
        }
        Amend(Pointers { a, b, c }) => {
            machine
                .stacks
                .get_mut(&machine.reg[a])
                .expect(NULL_STACK_ERR)[machine.reg[b] as usize] = machine.reg[c]
        }
        Add(Pointers { a, b, c }) => machine.reg[a] = machine.reg[b].wrapping_add(machine.reg[c]),
        Mul(Pointers { a, b, c }) => machine.reg[a] = machine.reg[b].wrapping_mul(machine.reg[c]),
        Div(Pointers { a, b, c }) => machine.reg[a] = machine.reg[b].wrapping_div(machine.reg[c]),
        Nand(Pointers { a, b, c }) => machine.reg[a] = !(machine.reg[b] & machine.reg[c]),
        Halt(_) => {
            let splitter = b"UM program follows colon:";
            let mut compare: Vec<u8> = Vec::new();
            let mut session: Vec<u8> = Vec::new();
            let mut program: Vec<u8> = Vec::new();
            let mut split = false;
            for b in machine.session {
                if !split {
                    if compare.len() == splitter.len() {
                        compare.remove(0);
                    }
                    compare.push(b);
                }
                if split {
                    program.push(b);
                } else {
                    session.push(b);
                }
                split = compare == splitter;
            }

            if program.len() > 0 {
                let mut pf = File::create("program.um").expect("Could not open program");
                pf.write_all(&program).expect("Could not write program");
            }
            let mut sf = File::create("session.log").expect("Could not open log");
            sf.write_all(&session).expect("Could not write log");
            
            exit(0)
        },
        Allocate(Pointers { b, c, .. }) => {
            machine.reg[b] = machine.insert_stack(vec![0u32; machine.reg[c] as usize]);
        }
        Abandon(Pointers { c, .. }) => {
            machine.stacks.remove(&machine.reg[c]);
        }
        Out(Pointers { c, .. }) => {
            machine.session.push(machine.reg[c] as u8);
            print!("{}", machine.reg[c] as u8 as char)
        },
        In(Pointers { c, .. }) => machine.reg[c] = read_byte(&mut machine),
        Load(Pointers { b, c, .. }) => {
            if machine.reg[b] > 0 {
                machine.stacks.insert(
                    0,
                    machine
                        .stacks
                        .get(&machine.reg[b])
                        .expect(NULL_STACK_ERR)
                        .clone(),
                );
            }
            machine.fin = machine.reg[c] as usize;
        }
        Ortho(OrthoPointers { a, value }) => machine.reg[a] = value,
    };
    machine
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction() {
        let target = Instruction::Add(Pointers { a: 7, b: 6, c: 0 });
        let other: Instruction = 0b00110000000000000000000111110000.into();
        assert_eq!(target, other);
    }
}
