use std::fmt::Debug;

use crate::symbols;

const C_INST: u16 = 0b1110_0000_0000_0000;

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
enum Comp {
    Zero = 0b101010,
    One = 0b111111,
    NegOne = 0b111010,
    D = 0b001100,
    A = 0b110000,
    NotD = 0b001101,
    NotA = 0b110001,
    NegD = 0b001111,
    NegA = 0b110011,
    DPlusOne = 0b011111,
    APlusOne = 0b110111,
    DMinusOne = 0b001110,
    AMinusOne = 0b110010,
    DPlusA = 0b000010,
    DMinusA = 0b010011,
    AMinusD = 0b000111,
    DAndA = 0b000000,
    DOrA = 0b010101,
}

const DEST_M: u16 = 0b001;
const DEST_D: u16 = 0b010;
const DEST_A: u16 = 0b100;

const JUMP_GT: u16 = 0b001;
const JUMP_EQ: u16 = 0b010;
const JUMP_LT: u16 = 0b100;

pub trait Instruction: Debug {
    fn to_u16(&self) -> u16;
}

#[derive(Debug)]
pub struct AInstruction {
    addr: u16,
}

impl AInstruction {
    pub fn new(value: &str, symbols: &mut symbols::Symbols) -> Self {
        let address = match value.parse::<u16>() {
            Ok(address) => address,
            Err(_) => {
                // If it isn't an int, it's a symbol!
                let value = String::from(value);
                match symbols.table.get(&value) {
                    Some(address) => *address,
                    None => {
                        // Symbol isn't in table yet; add it at the next
                        // available address
                        let symbol_addr = symbols.next_addr;
                        symbols.table.insert(value, symbol_addr);
                        symbols.next_addr += 1;

                        symbol_addr
                    },
                }
            },
        };

        Self {
            addr: address,
        }
    }
}

impl Instruction for AInstruction {
    fn to_u16(&self) -> u16 {
        self.addr
    }
}

#[derive(Debug)]
pub struct CInstruction {
    a: bool,
    comp: Comp,
    dest: u16,
    jump: u16,
}

impl CInstruction {
    pub fn new(dest: Option<&str>, comp: &str, jump: Option<&str>) -> Self {
        let dest = match dest {
            Some(dest) => {
                let mut dest_value = 0b000;

                for dest_char in dest.chars() {
                    dest_value |= match dest_char {
                        'A' => DEST_A,
                        'M' => DEST_M,
                        'D' => DEST_D,
                        _ => panic!("unrecognized destination {}", dest_char),
                    };
                }

                dest_value
            },
            None => 0b000,
        };

        // If this is one of the computations that accesses memory, we replace
        // M with A for convenience in matching below
        let mut comp = String::from(comp);
        let a = comp.contains('M');
        if a {
            comp = comp.replace("M", "A");
        }

        let comp = match &comp[..] {
            "0" => Comp::Zero,
            "1" => Comp::One,
            "-1" => Comp::NegOne,
            "D" => Comp::D,
            "A" => Comp::A,
            "!D" => Comp::NotD,
            "!A" => Comp::NotA,
            "-D" => Comp::NegD,
            "-A" => Comp::NegA,
            "D+1" => Comp::DPlusOne,
            "A+1" => Comp::APlusOne,
            "D-1" => Comp::DMinusOne,
            "A-1" => Comp::AMinusOne,
            "D+A" => Comp::DPlusA,
            "D-A" => Comp::DMinusA,
            "A-D" => Comp::AMinusD,
            "D&A" => Comp::DAndA,
            "D|A" => Comp::DOrA,
            &_ => panic!("unrecognized computation {}", comp),
        };

        let jump = match jump {
            Some("JGT") => JUMP_GT,
            Some("JEQ") => JUMP_EQ,
            Some("JGE") => JUMP_GT | JUMP_EQ,
            Some("JLT") => JUMP_LT,
            Some("JNE") => JUMP_GT | JUMP_LT,
            Some("JLE") => JUMP_EQ | JUMP_LT,
            Some("JMP") => JUMP_GT | JUMP_EQ | JUMP_LT,
            Some(unknown) => panic!("unrecognized jump {}", unknown),
            None => 0b000,
        };

        Self {
            a,
            comp,
            dest,
            jump,
        }
    }
}

impl Instruction for CInstruction {
    fn to_u16(&self) -> u16 {
        let mut value = C_INST;

        value |= (self.a as u16) << 12;
        value |= (self.comp as u16) << 6;
        value |= self.dest << 3;
        value |= self.jump;

        value
    }
}
