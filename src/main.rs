extern crate argparse;

use argparse::{ArgumentParser, Store};
use std::fs::File;
use std::io::Read;
use std::string::String;

struct Args {
    src: String,
    debug_level: u32,
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

    fn parse(&mut self, src: &[char]) -> Result<usize, String> {
        self._parse(src, 0)
    }

    fn _optimize_merge(src: &AST, dst: &mut AST) {
        let mut ctx = None;
        for c in &src.codes {
            match c {
                Code::Add(i) => match ctx {
                    Some(Code::Add(j)) => ctx = Some(Code::Add(i + j)),
                    Some(some) => {
                        dst.codes.push(some);
                        ctx = Some(Code::Add(*i));
                    }
                    None => ctx = Some(Code::Add(*i)),
                },
                Code::Move(i) => match ctx {
                    Some(Code::Move(j)) => ctx = Some(Code::Move(i + j)),
                    Some(some) => {
                        dst.codes.push(some);
                        ctx = Some(Code::Move(*i));
                    }
                    None => ctx = Some(Code::Move(*i)),
                },
                Code::Loop(ref loop_src) => {
                    if let Some(other) = ctx {
                        dst.codes.push(other);
                        ctx = None;
                    }
                    let mut loop_dst = AST::new();
                    AST::_optimize_merge(loop_src, &mut loop_dst);
                    dst.codes.push(Code::Loop(Box::new(loop_dst)));
                }
                Code::Get => {
                    if let Some(other) = ctx {
                        dst.codes.push(other);
                        ctx = None;
                    }
                    dst.codes.push(Code::Get);
                }
                Code::Put => {
                    if let Some(other) = ctx {
                        dst.codes.push(other);
                        ctx = None;
                    }
                    dst.codes.push(Code::Put);
                }
            }
        }
        if let Some(some) = ctx {
            dst.codes.push(some);
        }
    }

    fn optimize(&self) -> AST {
        let mut opt = AST::new();
        AST::_optimize_merge(&self, &mut opt);
        opt
    }
}

struct Tape {
    mem: Vec<u8>,
    point: usize,
}

impl Tape {
    fn new() -> Tape {
        Tape {
            mem: vec![0; 2_i32.pow(20) as usize],
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
    if args.debug_level >= 3 {
        println!("{:?}", src);
    }
    let mut ast = AST::new();
    ast.parse(&src)?;
    if args.debug_level >= 2 {
        println!("{:?}", ast);
    }

    let optimized = ast.optimize();
    if args.debug_level >= 1 {
        println!("{:?}", optimized);
    }

    let mut tape = Tape::new();
    tape.evaluate(&optimized);

    Ok(())
}

fn parse_args() -> Args {
    let mut args = Args {
        src: "-".to_string(),
        debug_level: 0,
    };
    {
        let mut p = ArgumentParser::new();
        p.set_description("brainfuck interpreter");
        p.refer(&mut args.src)
            .add_argument("src", Store, "brainfuck source file");
        p.refer(&mut args.debug_level)
            .add_option(&["--debug", "-d"], Store, "debug level");
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
