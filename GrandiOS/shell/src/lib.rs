#![no_std]
#![feature(alloc,global_allocator)]
#![feature(range_contains)]
#![feature(slice_concat_ext)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
#[macro_use]
extern crate aids;
init!();

mod utils{
    pub mod parser;
    pub mod vt {
        use core::fmt::Write;
        extern crate vt as vt_lib;
        pub use self::vt_lib::*;
        //These two function currently cannot be implemented in vt, since they require a read makro (and we dont have input streams, or any kind of abstraction for this).
        pub fn get_position() -> (u32, u32) {
            print!("\x1B[6n");
            //we expect a response in the form <Esc>[h;wR
            let mut h: u32 = 0;
            let mut w: u32 = 0;
            let _ = read!(); //Escape
            let _ = read!(); //[
            let mut c = read!();
            while c != 59 { //read to ;
                h = h*10 + (c as u32) - 48;
                c = read!();
            }
            c = read!();
            while c != 82 { //read to R
                w = w*10 + (c as u32) - 48;
                c = read!();
            }
            (w, h)
        }
        
        pub fn get_size() -> (u32, u32) {
            print!("\x1B7");
            print!("\x1B[999:999H");
            let res = get_position();
            print!("\x1B8");
            res
        }
    }
}
mod commands{
    //pub mod logo;
    pub mod cat;
    //pub mod test;
    pub mod edit;
    pub mod cowsay;
    pub mod math;
    pub mod higher;
}

use utils::parser::*;
use utils::vt;
use commands::*;
use core::str;
use core::result::Result;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::vec_deque::VecDeque;
use alloc::linked_list::LinkedList;

static mut IT: Argument = Argument::Nothing;
pub fn get_it(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    unsafe { args.insert(0, IT.clone()); }
    Ok(args)
}

macro_rules! command {
	( $t:tt, $o:expr, $c:tt, $m:tt ) => {
        (Argument::$t($o.to_string()), $m::$c as fn(Vec<Argument>) -> Result<Vec<Argument>, String>)
	};
	//( $o:tt, $c:tt, $m:tt ) => { command!(Operator, $o, $c, $m) };
	//( $c:tt, $m:tt ) => { command!(Method, $c, $m, $c) };
	//( $m:tt ) => { command!(Method, $m, $m, exec) };
}

static mut COMMANDS: Option<Vec<(Argument, fn(Vec<Argument>) -> Result<Vec<Argument>,String>)>> = None;

pub fn run() {
    println!("Welcome to pfush - the perfect functional shell");
    println!("type help for command list");
    unsafe {
        COMMANDS = Some(vec![
            command!(Method, "it", get_it, self),
            //command!(Method, "logo", exec, logo),
            //command!(Method, "test", exec, test),
            command!(Method, "edit", exec, edit),
            command!(Method, "cowsay", exec, cowsay),
            command!(Method, "cat", exec, cat),
            command!(Method, "map", map, higher),
            command!(Method, "foldl", foldl, higher),
            command!(Operator, ".", dot, higher),
            command!(Operator, "+", plus, math),
            command!(Operator, "-", minus, math),
            command!(Operator, "*", times, math),
            command!(Operator, "/", div, math),
        ]);
    }
    let mut history = VecDeque::new();
    loop {
        let mut raw_input = read_command("> ", &mut history);
        history.push_back(raw_input.clone());
        match parse(&mut raw_input, 0) {
            Err((s,p)) => { println!("{}^\n{}", "-".repeat(p+1), s); continue; },
            Ok(mut v) => { 
                match apply(&mut v.0[0]) {
                    Some(arg) => {
                        unsafe { IT = arg.clone(); }
                        if arg.is_something() {
                            println!("{}", arg.to_string());
                        }
                    }
                    None => {}
                }
            }
        }
    }
}

pub fn get_function(command: Argument) -> Option<fn(Vec<Argument>) -> Result<Vec<Argument>,String>> {
    unsafe {
        for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
            if command == *c {
                return Some(m);
            }
        }
    }
    None
}

pub fn unpack_args(args: &mut Vec<Argument>, len: usize) {
    for i in 0..(if len > 0 && len <= args.len() { len } else { args.len() }) {
        while args[i].is_application() && args[i].get_application().len() == 1 {
            args[i] = args[i].get_application()[0].clone();
        }
    }
}

pub fn eval_args(args: &mut Vec<Argument>, len: usize) {
    for i in 0..(if len > 0 && len <= args.len() { len } else { args.len() }) {
        while args[i].is_application() {
            match apply(&mut args[i]) {
                Some(s) => { 
                    if args[i] == s { break; }
                    args[i] = s;
                } , None => { return; }
            };
        }
    }
}

pub fn apply(app: &mut Argument) -> Option<Argument> {
    if !app.is_application() {
        println!("Unexpected call of apply without Application");
        return None;
    }
    let mut args = app.get_application();
    unpack_args(&mut args, 2);
    if args.len() == 1 && args[0].is_application() { return apply(&mut args[0]); }
    if args.len() == 1 && !args[0].is_method() { return Some(args[0].clone()); }
    if args.is_empty() { return None; }
    if args[0].is_application() {
        args = {
            let mut t = args.remove(0).get_application();
            t.append(&mut args);
            t
        };
    }
    let mut command = if args.len() > 1 && args[1].is_operator() {
        args.remove(1)
    } else {
        args.remove(0)
    };
    if command == Argument::Method("help".to_string()) {
        if args.is_empty() {
            unsafe {
                for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
                    if c.is_method() {
                        print!("{} ", c.to_string());
                    }
                }
            }
            println!("");
            return None;
        } else {
            command = args[0].clone();
            args[0] = Argument::Method("help".to_string());
        }
    }
    unsafe {
        for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
            if command == *c {
                match m(args) {
                    Err(msg) => { println!("Error: {}", msg); return None; },
                    Ok(res) => {
                        if res.len() == 1 { return Some(res[0].clone()); }
                        return Some(if res.is_empty() { Argument::Nothing } else { Argument::Application(res) });
                    }
                }
            }
        }
    }
    println!("Unknown command: {}", command.to_string());
    return None;
}

pub fn move_to(pos: usize, dest: usize) {
    if dest < pos {
        print!("{}", &vt::CursorControl::Left{count: (pos-dest) as u32});
    } else {
        print!("{}", &vt::CursorControl::Right{count: (dest-pos) as u32});
    }
}

fn clear_prompt(prompt: &str, s: usize) {
    print!("\r{}{}\r{}", prompt, " ".repeat(s), prompt);
}

fn print_command(ln: &LinkedList<u8>) {
    print!("{}", ln.iter().map(|x| *x as char).collect::<String>());
}

fn print_split_command<F>(ln: &mut LinkedList<u8>, prompt: &str, stringpos: usize, left: bool, f: F) where F: Fn(&mut LinkedList<u8>) {
    clear_prompt(prompt, ln.len());
    let mut others = ln.split_off(stringpos);
    f(ln);
    print_command(&ln);
    print!("{}", &vt::CursorControl::SavePos);
    print_command(&others);
    ln.append(&mut others);
    print!("{}", &vt::CursorControl::LoadPos);
    if left { print!("{}", &vt::CursorControl::Left{count: 1}); }
}

pub fn read_command(prompt: &str, history: &mut VecDeque<LinkedList<u8>>) -> LinkedList<u8> {
    print!("{}", prompt);
    let mut ln = LinkedList::new();
    let mut pos = 0;
    let mut escape = false;
    let mut sequence = vec!();
    let mut histpos = history.len();
    let mut stringpos = 0;
    loop {
        let c = read!();
        match c {
            10 | 13 => { //newline
                println!("");
                ln.push_back(32); //make life easier for the parser
                return ln;
            },
            12 => { // ^L
            },
            9 => { //tab
                if !(ln.contains(&32)) { //we did not have a space yet
                    unsafe {
                        for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
                            //if c starts with ln, do stuff
                        }
                    }
                }
            },
            127 => { //backspace
                if stringpos > 0 {
                    print_split_command(&mut ln, prompt, stringpos, false, |ln: &mut LinkedList<u8>| {ln.pop_back();});
                    stringpos -= 1;
                }
            },
            27 => { //escape
                escape = true;
            },
            _ => {
                if escape {
                    if c != 91 { sequence.push(c); }
                } else {
                    if stringpos == ln.len() {
                        ln.push_back(c);
                    } else {
                        print_split_command(&mut ln, prompt, stringpos, true, |ln: &mut LinkedList<u8>| {ln.push_back(c);});
                    }
                    stringpos += 1;
                }
                if !escape {
                    print!("{}", c as char);
                }
                if escape && ((65..69).contains(c) || c == 126) {
                    escape = false;
                    match vt::parse_input(str::from_utf8(&sequence[..]).unwrap()) {
                        vt::Input::Home => {
                            move_to(stringpos, 0);
                            stringpos = 0;
                        },
                        vt::Input::End => {
                            move_to(stringpos, ln.len());
                            stringpos = ln.len();
                        },
                        vt::Input::Delete => {
                            if stringpos < ln.len() {
                                print_split_command(&mut ln, prompt, stringpos+1, true, |ln: &mut LinkedList<u8>| {ln.pop_back();});
                            }
                        },
                        vt::Input::Left => {
                            if stringpos > 0 {
                                stringpos -= 1;
                                print!("{}", &vt::CursorControl::Left{count: 1});
                            }
                        },
                        vt::Input::Right => {
                            if stringpos < ln.len() {
                                stringpos += 1;
                                print!("{}", &vt::CursorControl::Right{count: 1});
                            }
                        },
                        vt::Input::Up => {
                            if histpos > 0 {
                                clear_prompt(prompt, ln.len());
                                histpos -= 1;
                                ln = history[histpos].clone();
                                ln.pop_back();
                                print_command(&ln);
                                stringpos = ln.len();
                            }
                        },
                        vt::Input::Down => {
                            clear_prompt(prompt, ln.len());
                            histpos += if histpos == history.len() { 0 } else { 1 };
                            if histpos == history.len() {
                                ln = LinkedList::new();
                            } else {
                                ln = history[histpos].clone();
                                ln.pop_back();
                                print_command(&ln);
                            }
                            stringpos = ln.len();
                        },
                        _ => {}
                    }
                    sequence = vec![];
                }
            }
        }
    }
}
