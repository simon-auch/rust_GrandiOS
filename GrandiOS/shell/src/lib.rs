#![no_std]
#![feature(asm)]
#![feature(lang_items)]
#![feature(alloc,global_allocator)]
#![feature(range_contains)]
#![feature(slice_concat_ext)]
#![allow(unused_variables)]
#![allow(unused_unsafe)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![feature(compiler_builtins_lib)]
#[macro_use]
extern crate aids;
init!();

#[macro_use]
mod utils{
    #[macro_use]
    pub mod evaluate;
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
        pub fn save_pos() { print!("\x1B7"); }
        pub fn restore_pos() { print!("\x1B8"); }
        pub fn get_size() -> (u32, u32) {
            save_pos();
            print!("\x1B[999;999H");
            let res = get_position();
            restore_pos();
            res
        }
    }
}

use utils::evaluate::*;

mod commands{
    pub mod logo;
    pub mod cat;
    pub mod htop;
    pub mod colors;
    pub mod edit;
    pub mod cowsay;
    pub mod math;
    pub mod bool;
    pub mod higher;
    pub mod list;
    pub mod u3;
}

extern crate rlibc;
extern crate compiler_builtins;
use utils::parser::*;
use utils::vt;
use commands::*;
use core::str;
use core::result::Result;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::vec_deque::VecDeque;
use alloc::linked_list::LinkedList;

struct TabState {
    index: i32,
    start: LinkedList<u8>,
}

static mut TABSTATE: Option<TabState> = None;

#[no_mangle]
pub extern fn _start() {
    println!("Welcome to pfush - the perfect functional shell");
    println!("type help for command list");
    populate_commands();
    let mut history = VecDeque::new();
    loop {
        print!("{}", &vt::CursorControl::Show); // make cursor visible
        unsafe { LOCALVARS = None; }
        let mut raw_input = read_command(&mut history);
        if history.is_empty() || *history.back().unwrap() != raw_input {
            history.push_back(raw_input.clone());
        }
        match parse(&mut raw_input, 0) {
            Err((s,p)) => { println!("{}^\n{}", "-".repeat(p+1), s); continue; },
            Ok(mut v) => { 
                match apply(&mut v.0[0]) {
                    Some(arg) => {
                        set_var(v.1, &arg);
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

fn prompt() -> String {
    format!("{1} {2} {3} {0}> ", &vt::CB_STANDARD,
            if get_led!(0) { &vt::CB_RED } else { &vt::CB_STANDARD },
            if get_led!(1) { &vt::CB_YELLOW } else { &vt::CB_STANDARD },
            if get_led!(2) { &vt::CB_GREEN } else { &vt::CB_STANDARD }
    ).to_string()
}

pub fn move_to(pos: usize, dest: usize) {
    if dest < pos {
        print!("{}", &vt::CursorControl::Left{count: (pos-dest) as u32});
    } else {
        print!("{}", &vt::CursorControl::Right{count: (dest-pos) as u32});
    }
}

fn clear_prompt(s: usize) {
    print!("\r{}{}\r{}", prompt(), " ".repeat(s), prompt());
    unsafe {
        if TABSTATE.is_some() {
            unsafe {
                for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
                    //if c starts with ln, do stuff
                }
            }
        }
    }
}

fn print_command(ln: &LinkedList<u8>) {
    print!("{}", ln.iter().map(|x| *x as char).collect::<String>());
}

fn print_split_command<F>(ln: &mut LinkedList<u8>, stringpos: usize, left: bool, f: F) where F: Fn(&mut LinkedList<u8>) {
    clear_prompt(ln.len());
    let mut others = ln.split_off(stringpos);
    f(ln);
    print_command(&ln);
    print!("{}", &vt::CursorControl::SavePos);
    print_command(&others);
    ln.append(&mut others);
    print!("{}", &vt::CursorControl::LoadPos);
    if left { print!("{}", &vt::CursorControl::Left{count: 1}); }
}

pub fn read_command(history: &mut VecDeque<LinkedList<u8>>) -> LinkedList<u8> {
    print!("{}", prompt());
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
                unsafe { TABSTATE = None; }
                return ln;
            },
            12 => { // ^L
            },
            9 => { //tab
                if ln.contains(&32) { continue; } //we currently only complete first words
                unsafe {
                    if TABSTATE.is_none() {
                        TABSTATE = Some(TabState{index: -1, start: ln.clone()});
                    }
                    TABSTATE.as_mut().unwrap().index += 1;
                }
                print_split_command(&mut ln, stringpos, false, |ln|{;});
            },
            127 => { //backspace
                if stringpos > 0 {
                    print_split_command(&mut ln, stringpos, false, |ln: &mut LinkedList<u8>| {ln.pop_back();});
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
                        print_split_command(&mut ln, stringpos, true, |ln: &mut LinkedList<u8>| {ln.push_back(c);});
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
                                print_split_command(&mut ln, stringpos+1, true, |ln: &mut LinkedList<u8>| {ln.pop_back();});
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
                                clear_prompt(ln.len());
                                histpos -= 1;
                                ln = history[histpos].clone();
                                ln.pop_back();
                                print_command(&ln);
                                stringpos = ln.len();
                            }
                        },
                        vt::Input::Down => {
                            clear_prompt(ln.len());
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
