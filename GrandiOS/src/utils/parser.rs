use core::result::Result;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::linked_list::LinkedList;
use alloc::string::{String,ToString};
use alloc::slice::SliceConcatExt;

#[derive(PartialEq,Debug,Clone)]
pub enum Argument {
    Nothing, Int(usize), Str(String), List(Vec<Argument>),
    Operator(String), Method(String), Application(Vec<Argument>),
}

impl ToString for Argument {
    fn to_string(&self) -> String {
        match self {
            &Argument::Int(i) => format!("{}",i).to_string(),
            &Argument::Str(ref s) => format!("\"{}\"", s).clone(),
            &Argument::Method(ref s) => s.clone(),
            _ => format!("{:?}", self).to_string()
            //&Argument::List(l) => ["[",l.iter().map(|a|a.to_string()).collect().join(","),"]"].concat().to_string()
        }
    }
}

impl Argument {
    pub fn is_str(&self) -> bool {
        match self {
            &Argument::Str(_) => true,
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
    pub fn get_str(&self) -> Option<String> {
        match self {
            &Argument::Str(ref s) => Some(s.clone()),
            _ => None
        }
    }
    pub fn get_int(&self) -> Option<usize> {
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
    pub fn get_application(&self) -> Vec<Argument> {
        match self {
            &Argument::Application(ref s) => s.clone(),
            _ => vec![]
        }
    }
}

pub fn parse(s: &mut LinkedList<u8>, start: usize) -> Result<Vec<Argument>,(String, usize)> {
    let mut res = vec![];
    let mut akk = vec![];
    let mut pos = start;
    let mut i = 0;
    let mut sign = 1;
    let mut base = 10;
    /* We currently have the following modes:
     * 0 - default, nothing to do
     * 10 - integer
     * 20 - string
     * 30 - function
     * 40 - operator
     * 50 - subexpression
     */
    let mut mode = 0;
    let mut oldmode = 0;
    let mut base = 10;
    while !s.is_empty() && mode != 55 {
        let c = s.pop_front().unwrap();
        pos += 1;
        if mode != 20 && c == 40 { // (
            mode = 50;
        }
        if mode != 20 && c == 41 { // )
            mode = 55;
        }
        if mode == 30 && !((65..91).contains(c) || (97..123).contains(c) || c == 95) {
            mode = 0;
        }
        if mode == 40 && !((33..38).contains(c) || (42..48).contains(c) || (58..65).contains(c)) {
            mode = 0;
        }
        if mode == 10 && !((48..58).contains(c) || c == 120 || c == 98 || c == 111) {
            mode = 0;
        }
        if mode == 0 && ((48..58).contains(c) || c == 45) { //number ahead
            mode = 10;
        }
        if mode == 0 && ((65..91).contains(c) || (97..123).contains(c) || c == 95) { //letter -> function
            mode = 30;
        }
        if mode == 0 && ((33..38).contains(c) || (42..48).contains(c) || (58..65).contains(c)) { //operator
            mode = 40;
        }
        if mode != 20 && c == 34 { //string
            mode = 20;
            continue;
        }
        if oldmode != mode {
            match oldmode {
                10 => {
                    res.push(Argument::Int(i));
                    i = 0;
                    base = 10;
                },
                30 => {
                    res.push(Argument::Method(String::from_utf8(akk).unwrap()));
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
                if  c == 120 || c == 98 || c == 111 { //we found x/b/o
                    if i != 0 {
                        return Err(("Cannot switch bases".to_string(), pos));
                    }
                    base = match c {
                        120 => 16, 98 => 2, 111 => 8, _ => 0
                    };
                } else {
                    let mut v = c as usize;
                    match v {
                        48...57 | 65...90 | 97...122 => {
                            v -= 48;
                            if base == 16 && v > 9 { v = v - 7; }
                            if base == 16 && v > 9 { v = v - 32; }
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
            _ => {}
        }
        if mode == 50 {
            match parse(s, pos) {
                Err(x) => { return Err(x); },
                Ok(mut e) => { res.append(&mut e); }
            }
            mode = 0;
        }
        oldmode = mode;
    }
    if mode == 55 && start == 0 {
        return Err(("Unbalanced parantheses!".to_string(), pos));
    }
    Ok(vec![Argument::Application(res)])
}
