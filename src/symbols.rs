use std::collections::HashMap;
use std::fmt::Write;

pub struct Symbols {
    pub table: HashMap<String, u16>,
    pub next_addr: u16,
}

impl Symbols {
    pub fn new() -> Self {
        let mut table: HashMap<String, u16> = HashMap::new();

        // Initialize symbols table with predefined symbols
        table.insert("SP".to_string(), 0x0);
        table.insert("LCL".to_string(), 0x1);
        table.insert("ARG".to_string(), 0x2);
        table.insert("THIS".to_string(), 0x3);
        table.insert("THAT".to_string(), 0x4);

        for addr in 0x0..0x10 {
            let mut symbol = String::new();
            write!(&mut symbol, "R{}", addr).expect("error writing symbol string");
            table.insert(symbol, addr);
        }

        table.insert("SCREEN".to_string(), 0x4000);
        table.insert("KBD".to_string(), 0x6000);

        Self {
            table,
            next_addr: 0x10,
        }
    }
}
