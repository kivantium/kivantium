use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::env;

struct State {
    address: u32,
    register: [u32; 32],
    dmem: Vec<u32>,
    imem: Vec<u32>,
}

impl State {
    fn init(instructions: Vec<u32>) -> State {
        State {
            address: 0,
            register: [0; 32],
            dmem: vec![0; 1024],
            imem: instructions,
        }
    }
    fn show_register(&self) {
        for (i, r) in self.register.iter().enumerate() {
            println!("reg{:02}: {:032b}", i, r);
        }
    }

    fn step(&mut self) -> Result<u32, u32>{
        let addr = (self.address / 4) as usize;
        if addr >= self.imem.len() {
            return Err(0);
        }
        let instruction = self.imem[addr];
        let opcode = instruction & 0x7f;
        match opcode {
            0b0000011 => self.exec_load(instruction),
            0b0010011 => self.exec_op_imm(instruction),
            0b0100011 => self.exec_store(instruction),
            0b0110011 => self.exec_op(instruction),
            0b0110111 => self.exec_lui(instruction),
            0b1100011 => self.exec_branch(instruction),
            0b1100111 => self.exec_jalr(instruction),
            0b1101111 => self.exec_jal(instruction),
            _         => panic!("Unsupported instruction"),
        }
        Ok(0)
    }

    fn exec_load(&mut self, instruction: u32) {
        self.address += 4;
    }
    fn exec_op_imm(&mut self, instruction: u32) {
        let funct3 = (instruction & 0x7000) >> 12;
        let rd =     (instruction & 0xf80) >> 7;
        let rs1 =    (instruction & 0xf8000) >> 15;
        let imm =    (instruction & 0xfff00000) >> 20;
        let shamt =  (instruction & 0x700000) >> 20;
        let funct7 = (instruction & 0xfe000000) >> 25;

        // sign extention
        let imm = imm | if instruction >> 31 == 1 {0xfffff000} else {0};
        let rs1 = self.register[rs1 as usize];

        self.register[rd as usize] = match funct3 {
            0b000 => rs1 + imm,
            0b010 => panic!("SLTI is not implemented"),
            0b011 => panic!("SLTIU is not implemented"),
            0b100 => rs1 ^ imm,
            0b110 => rs1 | imm,
            0b111 => rs1 & imm,
            0b001 => panic!("SLLI is not implemented"),
            0b101 => panic!("SRLI/SRLI is not implemented"),
            _     => panic!("Unknown OP_IMM"),
        };
        self.address += 4;
    }

    fn exec_store(&mut self, instruction: u32) {
        panic!("STORE is not implemented");
    }
    fn exec_op(&mut self, instruction: u32) {
        panic!("OP is not implemented");
    }
    fn exec_lui(&mut self, instruction: u32) {
        panic!("LUI is not implemented");
    }
    fn exec_branch(&mut self, instruction: u32) {
        panic!("BRANCH is not implemented");
    }
    fn exec_jalr(&mut self, instruction: u32) {
        panic!("JALR is not implemented");
    }
    fn exec_jal(&mut self, instruction: u32) {
        panic!("JAL is not implemented");
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        panic!("Usage: {} <filename>", args[0]);
    } 

    let f = match File::open(&args[1]) {
        Ok(file) => BufReader::new(file),
        Err(err) => panic!("File open error: {:?}", err),
    };

    let mut instructions = Vec::new();
    for line in f.lines() {
        let l = line.unwrap();
        let instr = u32::from_str_radix(&l, 2).unwrap();
        instructions.push(instr);
    }
    let mut state = State::init(instructions);
    loop {
        match state.step() {
            Ok(_) => (),
            Err(e) => {
                if e == 0 {
                    std::process::exit(0);
                }
            }
        }
    }
}
