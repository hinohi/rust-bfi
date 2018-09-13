extern crate argparse;

use argparse::{ArgumentParser, Store};
use std::fs::File;
use std::io::Read;
use std::string::String;

struct Args {
    src: String,
}

#[derive(Debug)]
enum Code {
    Get,
    Put,
    Add(i32),
    Move(isize),
    Loop(Box<AST>),
}

#[derive(Debug)]
struct AST {
    codes: Vec<Code>,
}

impl AST {
    fn new() -> AST {
        AST { codes: vec![] }
    }
    fn _parse(&mut self, src: &[char], stack: u32) -> Result<usize, String> {
        let mut i = 0;
        while i < src.len() {
            match src[i] {
                '+' => self.codes.push(Code::Add(1)),
                '-' => self.codes.push(Code::Add(-1)),
                '>' => self.codes.push(Code::Move(1)),
                '<' => self.codes.push(Code::Move(-1)),
                ',' => self.codes.push(Code::Get),
                '.' => self.codes.push(Code::Put),
                '[' => {
                    let mut lp = AST::new();
                    i += lp._parse(&src[i + 1..], stack + 1)?;
                    self.codes.push(Code::Loop(Box::new(lp)));
                }
                ']' => {
                    if stack > 0 {
                        return Ok(i + 1);
                    } else {
                        return Err("開き括弧が足りない".to_string());
                    }
                }
                _ => (),
            }
            i += 1;
        }
        if stack > 0 {
            Err("閉じ括弧が足りない".to_string())
        } else {
            Ok(i)
        }
    }

    fn parse(&mut self, src: &[char])-> Result<usize, String> {
        self._parse(src, 0)
    }
}

struct Tape {
    mem: Vec<u8>,
    point: usize,
}

impl Tape {
    fn new() -> Tape {
        Tape {
            mem: vec![0; 32],
            point: 0,
        }
    }

    fn put_char(&self) {
        print!("{}", char::from(self.mem[self.point]))
    }

    fn get_char(&mut self) {
        self.mem[self.point] = 0;
    }

    fn evaluate(&mut self, ast: &AST) {
        for code in &ast.codes {
            match code {
                Code::Add(i) => self.mem[self.point] = self.mem[self.point].wrapping_add(*i as u8),
                Code::Move(i) => {
                    if *i > 0 {
                        self.point = self.point.wrapping_add(*i as usize);
                    } else {
                        self.point = self.point.wrapping_add(*i as usize);
                    }
                }
                Code::Put => self.put_char(),
                Code::Get => self.get_char(),
                Code::Loop(ref loop_asp) => {
                    while self.mem[self.point] > 0 {
                        self.evaluate(&loop_asp);
                    }
                }
            }
            // println!("{:?} {}", self.mem, self.point);
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let mut reader = match File::open(args.src) {
        Ok(f) => f,
        Err(e) => return Err(e.to_string()),
    };
    let mut buf = String::new();
    match reader.read_to_string(&mut buf) {
        Err(e) => return Err(e.to_string()),
        Ok(_) => (),
    };

    let src: Vec<char> = buf.chars().collect();
    println!("{:?}", src);

    let mut ast = AST::new();
    ast.parse(&src)?;
    println!("{:?}", ast);

    let mut tape = Tape::new();
    tape.evaluate(&ast);

    Ok(())
}

fn parse_args() -> Args {
    let mut args = Args {
        src: "-".to_string(),
    };
    {
        let mut p = ArgumentParser::new();
        p.set_description("brainfuck interpreter");
        p.refer(&mut args.src)
            .add_argument("src", Store, "brainfuck source file");
        p.parse_args_or_exit();
    }
    args
}

fn main() {
    let args = parse_args();
    match run(args) {
        Err(e) => {
            println!("{:?}", e);
        }
        _ => (),
    }
}
