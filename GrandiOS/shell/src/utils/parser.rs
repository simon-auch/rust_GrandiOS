use core::result::Result;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::linked_list::LinkedList;
use alloc::string::{String,ToString};
use alloc::slice::SliceConcatExt;

#[derive(PartialEq,Debug,Clone)]
pub enum Argument {
    Nothing, Bool(bool), Int(isize), Str(String), List(Vec<Argument>),
    Operator(String), Method(String), Application(Vec<Argument>),
}

impl ToString for Argument {
    fn to_string(&self) -> String {
        match self {
            &Argument::Bool(b) => format!("{}",b).to_string(),
            &Argument::Int(i) => format!("{}",i).to_string(),
            &Argument::Str(ref s) => format!("\"{}\"", s).clone(),
            &Argument::Method(ref s) | &Argument::Operator(ref s) => s.clone(),
            &Argument::Application(ref l) => ["(".to_string(),l.iter().map(|a|a.to_string()).collect::<Vec<String>>().join(" "),")".to_string()].concat(),
            &Argument::List(ref l) => ["[".to_string(),l.iter().map(|a|a.to_string()).collect::<Vec<String>>().join(","),"]".to_string()].concat(),
            &Argument::Nothing => "".to_string(),
        }
    }
}

impl Argument {
    pub fn is_something(&self) -> bool {
        match self { &Argument::Nothing => false, _ => true }
    }
    pub fn is_list(&self) -> bool {
        match self {
            &Argument::List(_) => true,
            _ => false
        }
    }
    pub fn is_str(&self) -> bool {
        match self {
            &Argument::Str(_) => true,
            _ => false
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            &Argument::Bool(_) => true,
            _ => false
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            &Argument::Int(_) => true,
            _ => false
        }
    }
    pub fn is_method(&self) -> bool {
        match self {
            &Argument::Method(_) => true,
            _ => false
        }
    }
    pub fn is_operator(&self) -> bool {
        match self {
            &Argument::Operator(_) => true,
            _ => false
        }
    }
    pub fn is_application(&self) -> bool {
        match self {
            &Argument::Application(_) => true,
            _ => false
        }
    }
    pub fn get_list(&self) -> Vec<Argument> {
        match self {
            &Argument::List(ref s) => s.clone(),
            _ => vec![]
        }
    }
    pub fn get_str(&self) -> Option<String> {
        match self {
            &Argument::Str(ref s) => Some(s.clone()),
            _ => None
        }
    }
    pub fn get_bool(&self) -> Option<bool> {
        match self {
            &Argument::Bool(b) => Some(b),
            _ => None
        }
    }
    pub fn get_int(&self) -> Option<isize> {
        match self {
            &Argument::Int(i) => Some(i),
            _ => None
        }
    }
    pub fn get_method_name(&self) -> Option<String> {
        match self {
            &Argument::Method(ref s) => Some(s.clone()),
            _ => None
        }
    }
    pub fn get_operator(&self) -> Option<String> {
        match self {
            &Argument::Operator(ref s) => Some(s.clone()),
            _ => None
        }
    }
    pub fn get_application(&self) -> Vec<Argument> {
        match self {
            &Argument::Application(ref s) => s.clone(),
            _ => vec![]
        }
    }
}

pub fn parse(s: &mut LinkedList<u8>, start: usize) -> Result<(Vec<Argument>, String, usize), (String, usize)> {
    let mut res = vec![];
    let mut akk = vec![];
    let mut pos = start;
    let mut i = 0;
    let mut sign = 1;
    let mut base = 10;
    let mut name = "it".to_string();
    //conditions for mode switching in the same order as the modes
    let cond: Vec<Box<Fn(u8) -> bool>> = vec![
        Box::new(|c| (48..58).contains(c) || (65..71).contains(c) || (97..103).contains(c)),
        Box::new(|c| c == 34), Box::new(|c| (65..91).contains(c) || (97..123).contains(c) || c == 95),
        Box::new(|c| (33..38).contains(c) || (42..48).contains(c) || (58..65).contains(c) || c == 92),

    ];
    /* We currently have the following modes:
     * 0 - default, nothing to do
     * 10 - integer
     * 20 - string
     * 30 - function
     * 40 - operator
     * 50 - subexpression
     * 60 - lists
     */
    let mut mode = 0;
    let mut oldmode = 0;
    while !s.is_empty() && mode != 55 {
        let c = s.pop_front().unwrap();
        pos += 1;
        // ( and )
        if mode != 20 && c == 40 { mode = 50; }
        if mode != 20 && c == 41 { mode = 55; }
        if mode != 20 && c == 91 { mode = 60; }
        if mode != 20 && c == 93 { mode = 65; }
        if mode != 20 && c == 44 { mode = 61; }
        if mode == 30 && !(cond[2](c) || (48..58).contains(c)) { mode = 0; }
        if mode == 40 && !cond[3](c) { mode = 0; }
        if mode == 10 && !(cond[0](c) || c == 120 || c == 98 || c == 111) {mode = 0; }
        if mode == 0 && cond[2](c) { mode = 30; }
        if mode == 0 && cond[3](c) { mode = 40; }
        if mode == 0 && cond[0](c) { mode = 10; }
        if mode != 20 && cond[1](c) { mode = 20; continue; }
        if oldmode != mode {
            match oldmode {
                10 => {
                    res.push(Argument::Int(i));
                    i = 0;
                    base = 10;
                },
                30 => {
                    let w = String::from_utf8(akk).unwrap();
                    if w == "let".to_string() {
                        name = "".to_string();
                    } else if name == "".to_string() {
                        name = w;
                        loop {
                            pos += 1;
                            match s.pop_front() {
                                Some(c) => if c == 61 { break; },
                                None => {return Err(("Missing =".to_string(), pos));}
                            }
                        }
                    } else {
                        res.push(Argument::Method(w));
                    }
                    akk = vec![];
                },
                40 => {
                    res.push(Argument::Operator(String::from_utf8(akk).unwrap()));
                    akk = vec![];
                },
                _ => {}
            }
        }
        match mode {
            10 => {
                if  c == 120 || c == 98 && base != 16 || c == 111 { //we found x/b/o
                    if i != 0 {
                        return Err(("Cannot switch bases".to_string(), pos));
                    }
                    base = match c {
                        120 => 16, 98 => 2, 111 => 8, _ => 0
                    };
                } else {
                    let mut v = c as isize;
                    match v {
                        48...57 | 65...90 | 97...122 => {
                            v -= 48;
                            if base == 16 && v > 9 { v = v - 7; }
                            if base == 16 && v > 15 { v = v - 32; }
                            if v >= base {
                                return Err(("Invalid digit".to_string(), pos));
                            }
                            i = i*base+v;
                        }, _ => {}
                    }
                }
            },
            20 => {
                if c == 34 {
                    res.push(Argument::Str(String::from_utf8(akk).unwrap()));
                    akk = vec![];
                    mode = 0;
                } else {
                    akk.push(c);
                }
            },
            30 | 40 => {
                akk.push(c);
            },
            55 => {
                if start != 0 {
                    return Ok((vec![precedence(res)], name, pos));
                } else {
                    return Err(("Unbalanced parantheses!".to_string(), pos));
                }
            },
            61 => { mode = 0; },
            65 => {
                if start != 0 {
                    return Ok((vec![Argument::List(res)], name, pos));
                } else {
                    return Err(("Unbalanced brackets!".to_string(), pos));
                }
            },
            _ => {}
        }
        if mode == 50 || mode == 60 {
            match parse(s, pos) {
                Err(x) => { return Err(x); },
                Ok((mut e, _, p)) => { res.append(&mut e); pos = p; }
            }
            mode = 0;
        }
        oldmode = mode;
    }
    if start != 0 {
        return Err(("Unbalanced parantheses or brackets!".to_string(), pos));
    }
    Ok((vec![precedence(res)], name, pos))
}

fn precedence(args: Vec<Argument>) -> Argument {
    let mut res = vec![];
    let mut akk: Vec<Argument> = vec![];
    // evaluate methods first
    for arg in args {
        if arg.is_operator() {
            res.push(match akk.len() {
                0 => Argument::Nothing,
                _ => Argument::Application(akk)
            });
            res.push(arg);
            akk = vec![];
        } else {
            akk.push(arg);
        }
    }
    if !akk.is_empty() {
        res.push(Argument::Application(akk));
    }
    opprec(res)
}

fn opprec(args: Vec<Argument>) -> Argument {
    if args.is_empty() { return Argument::Nothing; }
    if args.len() > 4 {
        if ["+".to_string(), "-".to_string(), "\\".to_string()].contains(&args[1].get_operator().unwrap()) {
            Argument::Application(vec![args[0].clone(),args[1].clone(),opprec(args[2..].to_vec())])
        } else {
            Argument::Application(vec![Argument::Application(args[0..3].to_vec()),args[3].clone(),opprec(args[4..].to_vec())])
        }
    } else {
        Argument::Application(args)
    }
}
