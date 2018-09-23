extern crate argparse;

use argparse::{ArgumentParser, Store};
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::string::String;

struct Args {
    src: String,
    opt_level: u32,
    debug_level: u32,
}

#[derive(Debug)]
enum Code {
    // Base
    Get,
    Put,
    Add(i32),
    Move(isize),
    Loop(Box<AST>),
    // Extension
    Assign(u8),
    Mull((isize, i32)),
}

impl Clone for Code {
    fn clone(&self) -> Self {
        match self {
            Code::Get => Code::Get,
            Code::Put => Code::Put,
            Code::Add(i) => Code::Add(*i),
            Code::Move(i) => Code::Move(*i),
            Code::Assign(i) => Code::Assign(*i),
            Code::Loop(ref src) => {
                let dst = src.clone();
                Code::Loop(dst)
            }
            Code::Mull(ref src) => {
                let dst = src.clone();
                Code::Mull(dst)
            }
        }
    }
}

#[derive(Debug)]
struct AST {
    codes: Vec<Code>,
}

impl Clone for AST {
    fn clone(&self) -> Self {
        let mut dst = AST::new();
        for code in &self.codes {
            dst.codes.push(code.clone());
        }
        dst
    }
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

    fn _optimize_merge(src: &AST) -> AST {
        let mut dst = AST::new();
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
                    let mut loop_dst = AST::_optimize_merge(loop_src);
                    dst.codes.push(Code::Loop(Box::new(loop_dst)));
                }
                else_code => {
                    if let Some(other) = ctx {
                        dst.codes.push(other);
                        ctx = None;
                    }
                    dst.codes.push(else_code.clone());
                }
            }
        }
        if let Some(some) = ctx {
            dst.codes.push(some);
        }
        dst
    }

    fn _optimize_simple_loop(src: &AST) -> (AST, bool) {
        let mut is_simple = true;
        let mut point = 0;
        let mut map = HashMap::new();
        for c in &src.codes {
            match c {
                Code::Add(i) => {
                    let count = map.entry(point).or_insert(0);
                    *count += *i;
                }
                Code::Move(i) => {
                    point += *i;
                }
                _ => {
                    is_simple = false;
                    break;
                }
            }
        }
        let mut dst = AST::new();
        if is_simple && point == 0 {
            let origin = map.get(&0).unwrap_or(&2);
            if map.len() == 1 && origin % 2 != 0 {
                dst.codes.push(Code::Assign(0));
                return (dst, true);
            }
            if *origin == -1 {
                for (key, value) in &map {
                    if *key == 0 || *value == 0 {
                        continue;
                    }
                    dst.codes.push(Code::Mull((*key, *value)));
                }
                dst.codes.push(Code::Assign(0));
                return (dst, true);
            }
        }
        for c in &src.codes {
            match c {
                Code::Loop(ref loop_src) => {
                    let (loop_dst, unnest) = AST::_optimize_simple_loop(loop_src);
                    if unnest {
                        dst.codes.extend(loop_dst.codes);
                    } else {
                        dst.codes.push(Code::Loop(Box::new(loop_dst)));
                    }
                }
                else_code => {
                    dst.codes.push(else_code.clone());
                }
            }
        }
        (dst, false)
    }

    fn optimize(&self, opt_level: u32) -> AST {
        match opt_level {
            0 => self.clone(),
            1 => AST::_optimize_merge(&self),
            _ => {
                let o1 = AST::_optimize_merge(&self);
                let (o2, _) = AST::_optimize_simple_loop(&o1);
                o2
            }
        }
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
            point: 4096,
        }
    }

    fn put_char(&self) {
        print!("{}", char::from(self.mem[self.point]));
        stdout().flush().unwrap();
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
                Code::Assign(i) => self.mem[self.point] = *i,
                Code::Mull((d, mul)) => {
                    let pos;
                    if *d > 0 {
                        pos = self.point + (*d as usize);
                    } else {
                        assert!(self.point >= ((-d) as usize));
                        pos = self.point - ((-d) as usize);
                    }
                    let a = self.mem[pos];
                    let b = a.wrapping_add((self.mem[self.point] as i32 * mul) as u8);
                    self.mem[pos] = b;
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

    let optimized = ast.optimize(args.opt_level);
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
        opt_level: 1,
        debug_level: 0,
    };
    {
        let mut p = ArgumentParser::new();
        p.set_description("brainfuck interpreter");
        p.refer(&mut args.src)
            .add_argument("src", Store, "brainfuck source file");
        p.refer(&mut args.opt_level)
            .add_option(&["--optimize", "-O"], Store, "optimize level");
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
