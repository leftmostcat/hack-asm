use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::Vec;

use regex::Regex;

mod instr;
mod symbols;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);

    let file = match File::open(&path) {
        Err(reason) => panic!("could not open {}: {}", path.display(), reason),
        Ok(file) => file,
    };

    let buf_reader = BufReader::new(file);

    // First pass; add labels to the symbols table and build a list of assembly
    // instructions
    let mut symbols = symbols::Symbols::new();
    let mut commands: Vec<String> = Vec::new();

    // Labels are constructed in assembly as `(<value>)`, where <value> is
    // a symbol representing the instruction immediately following the label
    let label_regex = Regex::new(r"^\((.[^\)]*)\)$").unwrap();

    for line in buf_reader.lines() {
        let line = match line {
            Err(reason) => panic!("could not read from {}: {}", path.display(), reason),
            Ok(line) => line,
        };

        let line = String::from(line.trim());
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(label_match) = label_regex.captures(&line[..]) {
            symbols.table.insert(String::from(label_match.get(1).unwrap().as_str()), commands.len() as u16);

            continue;
        }

        commands.push(line);
    }

    // Second pass; process a-instruction and c-instruction assembly into
    // structured data
    let mut instructions: Vec<Box<dyn instr::Instruction>> = Vec::new();

    // A-instructions are constructed in assembly as `@<value>`, where <value>
    // is either a 15-bit unsigned integer or a symbol representing a memory
    // address
    let a_inst_regex = Regex::new(r"^@(.+)$").unwrap();

    /* C-instructions are constructed in assembly as `<dest>=<comp>;<jump>`,
     * where the following hold true:
     *
     * * <dest> may contain "A", "M", and "D", in that order and in any
     *   combination; if empty, it is omitted along with the following `=`
     * * <comp> is the intended computation; for a full list, see the resources
     *   linked below
     * * <jump> is one of JGT, JEQ, JGE, JLT, JNE, JLE, or JMP, or empty; if
     *   empty, it is omitted along with the preceding ';'
     *
     * For a full specification, refer to §4.2.3 of The Elements of Computing
     * Systems, available at present from https://www.nand2tetris.org/project04
     * under the "Resources" heading */
    let c_inst_regex = Regex::new(r"^(?:(?P<dest>A?M?D?)=)?(?P<comp>[01AMD!+&|-]{1,3})(?:;(?P<jump>J[EGLMN][EPQT]))?").unwrap();

    for command in commands {
        if let Some(a_inst_match) = a_inst_regex.captures(&command[..]) {
            let inst = instr::AInstruction::new(a_inst_match.get(1).unwrap().as_str(), &mut symbols);
            instructions.push(Box::new(inst));

            continue;
        }

        if let Some(c_inst_match) = c_inst_regex.captures(&command[..]) {
            let dest = match c_inst_match.name("dest") {
                Some(dest) => Some(dest.as_str()),
                None => None,
            };

            let jump = match c_inst_match.name("jump") {
                Some(jump) => Some(jump.as_str()),
                None => None,
            };

            let inst = instr::CInstruction::new(dest, c_inst_match.name("comp").unwrap().as_str(), jump);
            instructions.push(Box::new(inst));
        }
    }

    for inst in instructions {
        println!("{:016b}", inst.to_u16());
    }
}
