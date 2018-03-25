use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Integer(i64),
    Id(String),
    Operator(char),
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Semicolon,
    INT,
}

fn lexer(input: &String) -> Result<Vec<Token>, String> {
    let mut tokens: Vec<Token> = vec![];
    let mut char_iter = input.chars().peekable();
    while let Some(c) = char_iter.next() {
        match c {
            c if c.is_whitespace() => continue,
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '{' => tokens.push(Token::LeftBracket),
            '}' => tokens.push(Token::RightBracket),
            ';' => tokens.push(Token::Semicolon),
            '+' | '-' | '*' | '/' => {
                tokens.push(Token::Operator(c));
            }
            '0'...'9' => {
                let mut value = String::new();
                value.push(c);
                while let Some(&'0'...'9') = char_iter.peek() {
                    value.push(char_iter.next().unwrap());
                }
                tokens.push(Token::Integer(value.parse::<i64>().unwrap()));
            }
            'a'...'z' | 'A'...'Z' | '_' => {
                let mut value = String::new();
                value.push(c);
                while let Some(&c) = char_iter.peek() {
                    match c {
                        c if c.is_alphanumeric() || c == '_' => {
                            value.push(char_iter.next().unwrap());
                        }
                        _ => {
                            break;
                        }
                    }
                }
                match value.as_ref() {
                    "int" => tokens.push(Token::INT),
                    _ => tokens.push(Token::Id(value)),
                }
            }
            _ => return Err(format!("Unknown character: {}", c)),
        }
    }
    Ok(tokens)
}

#[derive(Debug,PartialEq,Clone)]
pub enum PrimType {
    Int,
}
#[derive(Debug,PartialEq,Clone)]
pub enum Node {
    Program { body: Vec<Node> },
    FuncDef { name: String, return_type: PrimType, body: Vec<Node> },
    FuncCall { name: String, args: Vec<Node> },
    Expr {value: Vec<Node> },
    IntConst{n: i64},
}

pub fn parser(tokens: Vec<Token>) -> Result<Node, String> {
    fn func_def(token: Token, token_iter: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        match token {
            Token::INT => {
                if let Some(token) = token_iter.next() {
                    match token {
                        Token::Id(name) => {
                            let token1 = token_iter.next().unwrap();
                            let token2 = token_iter.next().unwrap();
                            if token1 == Token::LeftParen && token2 == Token::RightParen {
                                let mut body = Vec::new();
                                if let Some(Token::LeftBracket) = token_iter.next() {
                                    while match token_iter.peek() {
                                        Some(&Token::RightBracket) | None 
                                            => false,
                                        _ => true,
                                        } {
                                        match walk(token_iter.next().unwrap(), token_iter) {
                                            Ok(nodes) => body.push(nodes),
                                            Err(value) => {return Err(value);},
                                        }
                                    }
                                    // skip Token::RightBracket
                                    token_iter.next().unwrap();
                                    Ok(Node::FuncDef {
                                        name: name,
                                        return_type: PrimType::Int,
                                        body: body
                                    })
                                } else {
                                    return Err(format!("Left bracket must follow function name"));
                                }
                            } else {
                                return Err(format!("Function definition must have empty argument"));
                            }
                        }
                        _ => {
                            return Err(format!("Id must follow the return type in FuncDef"));
                        }
                    }
                } else {
                    return Err(format!("No token after type name in FuncDef."));
                }
            }
            _ => {
                return Err(format!("Function definition must start with type specifier"));
            }
        }
    }

    fn walk(token: Token, token_iter: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        match token {
            Token::Integer(value) => Ok(Node::IntConst{n: value}),
            Token::Id(name) => { // function call
                let func_name = name.clone();
                if let Some(Token::LeftParen) = token_iter.next() {
                    let mut args = Vec::new();
                    while match token_iter.peek() {
                        Some(&Token::RightParen) | None => false,
                            _ => true,
                    } {
                        match walk(token_iter.next().unwrap(), token_iter) {
                            Ok(nodes) => args.push(nodes),
                            Err(value) => return Err(value),
                        }
                    }
                    token_iter.next().unwrap();
                    if let Some(&Token::Semicolon) = token_iter.peek() {
                        token_iter.next().unwrap();
                    } else {
                        return Err(format!("Semicolon must follow after function call {:?}, name", name));
                    }
                    Ok(Node::FuncCall {
                        name: func_name,
                        args: args,
                    })
                } else {
                    return Err(format!("Left parenthesis must follow {:?}", name));
                }
            }
            _ => {
                return Err(format!("This token should not come here"));
            }
        }
    }

    let mut body: Vec<Node> = vec![];
    let mut token_iter = tokens.into_iter().peekable();
    while let Some(token) = token_iter.next() {
        match func_def(token, &mut token_iter) {
            Ok(nodes) => body.push(nodes),
            Err(value) => return Err(value),
        }
    }
    Ok(Node::Program{body: body})
}

/*#[derive(Debug,PartialEq,Clone)]
pub enum Node {
    Program { body: Vec<Node> },
    FuncDef { name: String, return_type: PrimType, body: Vec<Node> },
    FuncCall { name: String, args: Vec<Node> },
    Expr {value: Vec<Node> },
    IntConst(i64),
}*/

fn generate_code(program: Node) {
    match program {
        Node::Program{body} => {
            for ast in body {
                translate_ast(ast);
            }
        }
        _ => {
            panic!("The root of the AST must be program.");
        }
    }
}

fn translate_ast(ast: Node) {
    match ast {
        Node::FuncDef{name, return_type, body} => {
            println!("{}:", name);
            for a in body {
                translate_ast(a);
            }
            if name == "main" {
                assert_eq!(return_type, PrimType::Int);
                println!("exit");
            } else {
                println!("jal $ra");
            }
        }
        Node::FuncCall{name, args} => {
            for arg in args {
                translate_ast(arg);
            }
            if name == "print_int" {
                println!("print_int $a0");
            } else {
                println!("jalr {}", name);
            }
        }
        Node::Expr{value} => {
            for v in value {
                translate_ast(v);
            }
        }
        Node::IntConst{n} => {
            println!("addi $a0, $zero, {}", n);
        }
        _ => {
            panic!("Unknown node: {:?}", ast);
        }
    }
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

    let lex_result = lexer(&src).unwrap();
    let parse_result = parser(lex_result);
    generate_code(parse_result.unwrap());
}
