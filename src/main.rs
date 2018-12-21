
use std::collections::HashMap;
use std::io::{self, Read};
use std::process::exit;

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
            value: ((i << 7) >> 7),
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
    Ortho(OrthoPointers)
}

use self::Instruction::*;

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
            _ => panic!("Bad Instruction")
        }
    }
}

fn as_u32(word: [u8; 4]) -> u32 {
    word.iter().enumerate().map(|(i, b)| (*b as u32) << ((3-i)*8)).sum()
}

fn main() -> io::Result<()> {

    let mut registers: [u32; 8] = [0; 8];
    let mut scroll: Vec<u32> = Vec::new();
    let mut stacks: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut finger: usize = 0;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut word: [u8; 4] = [0; 4];
    while let Ok(_) = stdin.read_exact(&mut word) {
        scroll.push(as_u32(word));
    };
    stacks.insert(0, scroll);
    loop {
        let t = cycle(finger, registers, stacks);
        finger = t.0;
        registers = t.1;
        stacks = t.2;
    }
}

static NULL_STACK_ERR: &'static str = "Attempted operation on unallocated stack";

fn insert_stack(stacks: &mut HashMap<u32, Vec<u32>>, stack: Vec<u32>) -> u32 {
    let mut key = 0;
    while stacks.contains_key(&key) {
        key = key + 1; 
    }
    stacks.insert(key, stack);
    key
}

fn cycle(mut finger: usize, mut reg: [u32; 8], mut stacks: Stacks) -> (usize, [u32; 8], Stacks) {
    let instruction: Instruction = stacks.get(&0)
        .and_then(|scroll| scroll.get(finger).cloned())
        .map(|v| v.into())
        .expect("Finger or stack invalid");
    finger = finger + 1;
    operate(finger, reg, stacks, instruction)
}

fn operate(mut finger: usize, mut reg: [u32; 8], mut stacks: Stacks, instruction: Instruction) ->  (usize, [u32; 8], Stacks) {
    match instruction {
        Move(Pointers{a, b, c}) => reg[a] = if reg[c] > 0 {reg[b]} else {reg[a]},
        Index(Pointers{a, b, c}) => reg[a] = stacks.get(&reg[b]).expect(NULL_STACK_ERR)[reg[c] as usize],
        Amend(Pointers{a, b, c}) => stacks.get_mut(&reg[a]).expect(NULL_STACK_ERR).insert(reg[b] as usize, reg[c]),
        Add(Pointers{a, b, c}) => reg[a] = reg[b].wrapping_add(reg[c]),
        Mul(Pointers{a, b, c}) => reg[a] = reg[b].wrapping_mul(reg[c]),
        Div(Pointers{a, b, c}) => reg[a] = reg[b] / reg[c],
        Nand(Pointers{a, b, c}) => reg[a] = !(reg[b] & reg[c]),
        Halt(_) => exit(0),
        Allocate(Pointers{b, c, ..}) => reg[b]= insert_stack(&mut stacks, vec![0u32; reg[c] as usize]),
        Abandon(Pointers{c, ..}) => {stacks.remove(&reg[c]);},
        Out(Pointers{c, ..}) => print!("{}", reg[c] as u8 as char),
        In(Pointers{c, ..}) => reg[c] = io::stdin().take(1).bytes().next().expect("No Input").expect("Invalid Input") as u32,
        Load(Pointers{b, c, ..}) => {
            stacks.insert(0, stacks.get(&reg[b]).expect(NULL_STACK_ERR).clone());
            finger = reg[c] as usize;
        },
        Ortho(OrthoPointers{a, value}) => reg[a] = value, 
    };
    (finger, reg, stacks)
}




#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_instruction() {
        let target = Instruction::Add(Pointers{
            a: 7,
            b: 6,
            c: 0

        });
        let other: Instruction = 0b00110000000000000000000111110000.into();
        assert_eq!(target, other);
        
    }
}