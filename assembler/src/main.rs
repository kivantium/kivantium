use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::env;
use std::collections::HashMap;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        panic!("Usage: {} <filename>", args[0]);
    } 

    let f = match File::open(&args[1]) {
        Ok(file) => BufReader::new(file),
        Err(err) => panic!("File open error: {:?}", err),
    };

    let mut symbols = HashMap::new();
    let mut address = 0;

    for line in f.lines() {
        let l = line.unwrap();
        let mut opcode = l.split_whitespace().nth(0).unwrap().to_string();
        if opcode.chars().rev().nth(0).unwrap() == ':' {
            opcode.pop();
            println!("{}, {}", opcode, address);
            symbols.insert(opcode, address);
        } else {
            parse_line(&l, &symbols);
        }
        address += 4;
    }
}

fn parse_line(line: &str, symbols: &HashMap<String, u32>) {
    let opcode = line.split_whitespace().nth(0).unwrap();
    match opcode {
        "add" | "sub" | "sll" | "slt" | "sltu" | "xor" 
              | "srl" | "sra" | "or" | "and" 
            => parse_r_type(&line),

        "addi" | "slti" | "sltiu" | "xori" | "ori" | "andi"
               | "lb" | "lh" | "lw" | "lbu" | "lhu"
            => parse_i_type(&line),

        "sb" | "sh" | "sw"
            => parse_s_type(&line),

        "beq" | "bne" | "blt" | "bge" | "bltu" | "bgeu"
            => parse_b_type(&line, &symbols),

        "jal"  => parse_jal(&line, &symbols),
        "jalr" => parse_jalr(&line, &symbols),
        "lui" => parse_lui(&line),

        _ => panic!("Unknown opcode!"),
    }
}

fn parse_r_type(line: &str) {
    let opcode = line.split_whitespace().nth(0).unwrap();
    let mut rd = line.split_whitespace().nth(1).unwrap().to_string();
    let mut rs1 = line.split_whitespace().nth(2).unwrap().to_string();
    let mut rs2 = line.split_whitespace().nth(3).unwrap().to_string();
    rd.pop(); rs1.pop(); rs2.pop();
    let (opcode, funct3, funct7) = match opcode {
        "add"  => (0b0110011, 0b000, 0b0000000),
        "sub"  => (0b0110011, 0b000, 0b0100000),
        "sll"  => (0b0110011, 0b001, 0b0000000),
        "slt"  => (0b0110011, 0b010, 0b0000000),
        "sltu" => (0b0110011, 0b011, 0b0000000),
        "xor"  => (0b0110011, 0b100, 0b0000000),
        "srl"  => (0b0110011, 0b101, 0b0000000),
        "sra"  => (0b0110011, 0b101, 0b0100000),
        "or"   => (0b0110011, 0b110, 0b0000000),
        "and"  => (0b0110011, 0b111, 0b0000000),
        _      => panic!("Unknown I type opcode!"),
    };
    println!("{:07b}{:05b}{:05b}{:03b}{:05b}{:07b}", 
             funct7, register(&rs2), register(&rs1), funct3, register(&rd), opcode);
}

fn parse_i_type(line: &str) {
    let opcode = line.split_whitespace().nth(0).unwrap();
    let mut rd = line.split_whitespace().nth(1).unwrap().to_string();
    let mut rs1 = line.split_whitespace().nth(2).unwrap().to_string();
    let imm = line.split_whitespace().nth(3).unwrap().to_string().parse::<u32>().unwrap();
    rd.pop(); rs1.pop();
    let (opcode, funct3, a) = match opcode {
        "addi"  => (0b0010011, 0b000, 0),
        "slti"  => (0b0010011, 0b010, 0),
        "sltiu" => (0b0010011, 0b011, 0),
        "xori"  => (0b0010011, 0b100, 0),
        "ori"   => (0b0010011, 0b110, 0),
        "andi"  => (0b0010011, 0b111, 0),
        "slli"  => (0b0010011, 0b001, 0),
        "srli"  => (0b0010011, 0b101, 0),
        "srai"  => (0b0010011, 0b101, 1),
        "lb"    => (0b0000011, 0b000, 0),
        "lh"    => (0b0000011, 0b001, 0),
        "lw"    => (0b0000011, 0b010, 0),
        "lbu"   => (0b0000011, 0b100, 0),
        "lhu"   => (0b0000011, 0b101, 0),
        _       => panic!("Unknown I type opcode!"),
    };
    if a == 0 {
        println!("{:012b}{:05b}{:03b}{:05b}{:07b}", 
                 imm, register(&rs1), funct3, register(&rd), opcode);
    } else {
        println!("0100000{:05b}{:05b}{:03b}{:05b}{:07b}", 
                 imm, register(&rs1), funct3, register(&rd), opcode);
    }
}

fn parse_s_type(line: &str) {
    let opcode = line.split_whitespace().nth(0).unwrap();
    let mut rs1 = line.split_whitespace().nth(1).unwrap().to_string();
    let mut rs2 = line.split_whitespace().nth(2).unwrap().to_string();
    let imm = line.split_whitespace().nth(3).unwrap().to_string().parse::<u32>().unwrap();
    rs1.pop(); rs2.pop();
    let (opcode, funct3) = match opcode {
        "sb" => (0b0100011, 0b000),
        "sh" => (0b0100011, 0b001),
        "sw" => (0b0100011, 0b010),
        _    => panic!("Unknown I type opcode!"),
    };
    println!("{:07b}{:05b}{:05b}{:03b}{:05b}{:07b}", 
             imm&0xffe, register(&rs2), register(&rs1), funct3, imm&0x01f, opcode);
}

fn parse_b_type(line: &str, symbols: &HashMap<String, u32>) {
    let opcode = line.split_whitespace().nth(0).unwrap();
    let mut rs1 = line.split_whitespace().nth(1).unwrap().to_string();
    let mut rs2 = line.split_whitespace().nth(2).unwrap().to_string();
    let label = line.split_whitespace().nth(3).unwrap().to_string();
    rs1.pop(); rs2.pop();
    let (opcode, funct3) = match opcode {
        "beq"  => (0b1100011, 0b000),
        "bne"  => (0b1100011, 0b001),
        "blt"  => (0b1100011, 0b100),
        "bge"  => (0b1100011, 0b101),
        "bltu" => (0b1100011, 0b110),
        "bgeu" => (0b1100011, 0b111),
        _      => panic!("Unknown B type opcode!"),
    };
    let addr = symbols.get(&label).unwrap();
    println!("{:01b}{:06b}{:05b}{:05b}{:03b}{:04b}{:01b}{:07b}", 
             addr&0x1000, addr&0x07e0, register(&rs2), register(&rs1),
             funct3, addr&0x001e, addr&0x0800, opcode);
}

fn parse_jal(line: &str, symbols: &HashMap<String, u32>) {
    let mut rd = line.split_whitespace().nth(1).unwrap().to_string();
    let label = line.split_whitespace().nth(2).unwrap().to_string();
    rd.pop();
    let addr = symbols.get(&label).unwrap();

    println!("{:01b}{:010b}{:01b}{:08b}{:05b}1101111", 
             addr&0x100000, addr&0x07fe, addr&0x0800, addr&0xff000, register(&rd));
}

fn parse_jalr(line: &str, symbols: &HashMap<String, u32>) {
    let mut rd = line.split_whitespace().nth(1).unwrap().to_string();
    let mut rs1 = line.split_whitespace().nth(2).unwrap().to_string();
    let label = line.split_whitespace().nth(3).unwrap().to_string();
    rd.pop(); rs1.pop();
    let addr = symbols.get(&label).unwrap();

    println!("{:012b}{:05b}000{:05b}1100111", 
             addr&0xfff, register(&rs1), register(&rd));
}


fn parse_lui(line: &str) {
    let mut rd = line.split_whitespace().nth(1).unwrap().to_string();
    let imm = line.split_whitespace().nth(3).unwrap().to_string().parse::<u32>().unwrap();
    rd.pop();

    println!("{:020b}{:05b}0110111", 
             imm, register(&rd));
}

fn register(reg: &String) -> u32 {
    match reg.as_str() {
        "x0"  | "zero"       => 0,
        "x1"  | "ra"         => 1,
        "x2"  | "sp"         => 2,
        "x3"  | "gp"         => 3,
        "x4"  | "tp"         => 4,
        "x5"  | "t0"         => 5,
        "x6"  | "t1"         => 6,
        "x7"  | "t2"         => 7,
        "x8"  | "s0" | "fp"  => 8,
        "x9"  | "s1"         => 9,
        "x10" | "a0"         => 10,
        "x11" | "a1"         => 11,
        "x12" | "a2"         => 12,
        "x13" | "a3"         => 13,
        "x14" | "a4"         => 14,
        "x15" | "a5"         => 15,
        "x16" | "a6"         => 16,
        "x17" | "a7"         => 17,
        "x18" | "s2"         => 18,
        "x19" | "s3"         => 19,
        "x20" | "s4"         => 20,
        "x21" | "s5"         => 21,
        "x22" | "s6"         => 22,
        "x23" | "s7"         => 23,
        "x24" | "s8"         => 24,
        "x25" | "s9"         => 25,
        "x26" | "s10"        => 26,
        "x27" | "s11"        => 27,
        "x28" | "t3"         => 28,
        "x29" | "t4"         => 29,
        "x30" | "t5"         => 30,
        "x31" | "t6"         => 31,
        _                    => panic!("Unknown register name {}", reg),
    }
}
