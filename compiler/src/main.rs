use std::io::prelude::*;
use std::fs::File;

use std::env;
use std::boxed::Box;

#[derive(Debug, Clone)]
enum LexItem {
    Paren(char),
    Op(char),
    Id(String),
    Num(i64),
    Semicolon,
    INT(),
}

enum AST {
    AstNum(i64),
    AstFunc(String, Box<AST>),
}

fn main() {
    // argument check
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: {} <filename>", args[0]);
    }

    // file open
    let mut f = match File::open(&args[1]) {
        Ok(file) => file,
        Err(err) => panic!("File open error: {:?}", err),
    };

    // read whole source into a string
    let mut src = String::new();
    f.read_to_string(&mut src).expect("File read error");

    let lex_result = lex(&src).unwrap();
    println!("Result of lexical analysis:");
    for token in &lex_result {
        println!("{:?}", token);
    }
    let ast = parse(&lex_result).unwrap();
    println!("\nGenerated AST:");
    for a in &ast {
        print_ast(&a);
    }
}

fn print_ast(ast: &AST) {
    match ast {
        &AST::AstNum(n) => {
            println!("AstNum({})", n);
        }
        &AST::AstFunc(ref s, ref a) => {
            println!("AstFunc({}", s);
            print_ast(&a);
            println!(")");
        }
    }
}

fn lex(input: &String) -> Result<Vec<LexItem>, String> {
    let mut result = Vec::new();
    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        if c.is_whitespace() {
            it.next();
        } else if c.is_numeric() {
            it.next();
            let n = get_number(c, &mut it);
            result.push(LexItem::Num(n));
        } else if c.is_alphabetic() {
            it.next();
            let id = get_id(c, &mut it);
            match id.as_ref() {
                "int" => result.push(LexItem::INT()),
                _     => result.push(LexItem::Id(id)),
            }
        } else {
            match c {
                '+' => {
                    it.next();
                    result.push(LexItem::Op(c));
                }
                '(' | ')' | '{' | '}' => {
                    it.next();
                    result.push(LexItem::Paren(c));
                }
                ';' => {
                    it.next();
                    result.push(LexItem::Semicolon);
                }
                _ => {
                    return Err(format!("Unexpected character {}", c));
                }
            }
        }
    }
    Ok(result)
}

fn get_number<T: Iterator<Item = char>>(c: char, iter: &mut std::iter::Peekable<T>) -> i64 {
    let mut number = c.to_string().parse::<i64>().expect("c is not a digit");
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<i64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    number
}

fn get_id<T: Iterator<Item = char>>(c: char, iter: &mut std::iter::Peekable<T>) -> String {
    let mut id = c.to_string();
    while let Some(c) = iter.peek().map(|c| c.clone()) { // TODO: avoid clone()
        if c.is_alphabetic() {
            id.push_str(&c.to_string());
            iter.next();
        } else {
            break;
        }
    }
    id
}

fn parse(tokens: &Vec<LexItem>) -> Result<Vec<AST>, String> {
    let mut result = Vec::new();
    let mut pos = 0;
    loop {
        if pos == tokens.len() {
            return Ok(result);
        }
        let t = tokens.get(pos);
        match t {
            Some(&LexItem::INT()) => {
                let (ast, next_pos) = parse_function(tokens, pos+1).unwrap();
                result.push(ast);
                pos = next_pos;
            }
            _ => {
                return Err(format!("Expected INT, found {:?}", t));
            }
        }
    }
}

fn parse_function(tokens: &Vec<LexItem>, pos: usize) -> Result<(AST, usize), String> {
    let t = tokens.get(pos);
    match t {
        Some(&LexItem::Id(ref id)) => {
            match tokens.get(pos+1) {
                Some(&LexItem::Paren(p1)) => {
                    if p1 == '{' {
                        let (ast, next_pos) = parse_expr(&tokens, pos+2).unwrap();
                        match tokens.get(next_pos) {
                            Some(&LexItem::Paren(p2)) => {
                                if p2 == '}' {
                                    Ok((AST::AstFunc(id.to_string(), Box::new(ast)), next_pos+1))
                                } else {
                                    Err(format!("Open parenthesis did not closed A"))
                                }
                            }
                            _ => {
                                Err(format!("Open parenthesis did not closed B"))
                            }
                        }
                    } else {
                        Err(format!("Function definition must start with {{"))
                    }
                }
                _ => {
                    Err(format!("Function definition must start with {{"))
                }
            }
        }
        _ => {
            Err(format!("Invalid function definition"))
        }
    }
}

fn parse_expr(tokens: &Vec<LexItem>, pos: usize) -> Result<(AST, usize), String> {
    let (ast, next_pos) = parse_num(&tokens, pos).unwrap();
    match tokens.get(next_pos) {
        Some(&LexItem::Semicolon) => {
            Ok((ast, next_pos+1))
        }
        _ => {
            Err(format!("EXPR must be separated by a SEMICOLON"))
        }
    }
}

fn parse_num(tokens: &Vec<LexItem>, pos: usize) -> Result<(AST, usize), String> {
    let t = tokens.get(pos);
    match t {
        Some(&LexItem::Num(n)) => {
            Ok((AST::AstNum(n), pos+1))
        }
        _ => {
            Err(format!("Error in parse_num"))
        }
    }
}
