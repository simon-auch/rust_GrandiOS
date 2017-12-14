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
            print!("{}", &vt_lib::DeviceStatus::QueryCursorPosition);
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
        pub fn set_position(p: (u32, u32)) { print!("{}", &vt_lib::CursorControl::Home{row: p.1, col: p.0}); }
        pub fn save_pos() { print!("{}", &vt_lib::CursorControl::SavePos); }
        pub fn restore_pos() { print!("{}", &vt_lib::CursorControl::LoadPos); }
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
    pub mod exit;
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
    index: usize,
    start: Vec<u8>,
    found: Vec<String>,
    pressed: bool,
    tab: bool,
    clear: bool,
    removed: usize,
}
impl TabState {
    pub fn new() -> TabState {
        TabState{
            index: 0, start: vec![], found: vec![],
            pressed: false, tab: false, clear: false,
            removed: 0
        }
    }
    pub fn clear(&mut self) {
        self.index = 0; self.start = vec![]; self.found = vec![];
        self.pressed = false; self.tab = false; self.clear = false;
        self.removed = 0;
    }
    pub fn get_completion(&self) -> Option<String> {
        if self.found.is_empty() { return None; }
        Some(self.found[self.index][self.start.len()..].to_string())
    }
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
                if v.1 != "it" { set_var(v.1, &v.0[0]); } else {
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
    unsafe {
        if TABSTATE.as_ref().unwrap().tab {
            let len = TABSTATE.as_ref().unwrap().start.len();
            if TABSTATE.as_ref().unwrap().found.is_empty() {
                for &(ref cmd, m) in COMMANDS.as_ref().unwrap().iter() {
                    if !cmd.is_method() { continue; }
                    let s = cmd.get_method_name().unwrap();
                    let mut starts = true;
                    for (i, c) in s.clone().as_bytes().iter().enumerate() {
                        if i == len { break; }
                        if *c != TABSTATE.as_ref().unwrap().start[i] {
                            starts = false;
                            break;
                        }
                    }
                    if starts {
                        TABSTATE.as_mut().unwrap().found.push(s.clone());
                    }
                }
                for &(ref s, ref m) in VARS.as_ref().unwrap().iter() {
                    let mut starts = true;
                    for (i, c) in s.clone().as_bytes().iter().enumerate() {
                        if i == len { break; }
                        if *c != TABSTATE.as_ref().unwrap().start[i] {
                            starts = false;
                            break;
                        }
                    }
                    if starts {
                        TABSTATE.as_mut().unwrap().found.push(s.clone());
                    }
                }
            }
            if !TABSTATE.as_ref().unwrap().found.is_empty() {
                let clear = TABSTATE.as_ref().unwrap().clear;
                TABSTATE.as_mut().unwrap().index %= TABSTATE.as_ref().unwrap().found.len(); 
                let p = TABSTATE.as_ref().unwrap().index;
                let opos = vt::get_position();
                let width = vt::get_size().0 as usize;
                let mut x = 0;
                print!("\r");
                if TABSTATE.as_ref().unwrap().pressed { print!("{}", &vt::CursorControl::Down{count: 1}); } else { print!("\n"); }
                print!("{}", &vt::DisplayAttribute::Reset);
                for (i, s) in TABSTATE.as_ref().unwrap().found.iter().enumerate() {
                    if i == p && !clear { print!("{}", &vt::DisplayAttribute::Reverse); }
                    if clear {
                        print!("{}", " ".repeat(s.len()));
                    } else {
                        print!("{}", s);
                    }
                    x += s.len()+1;
                    if i == p && !clear { print!("{}", &vt::DisplayAttribute::Reset); }
                    print!(" ");
                }
                let cpos = vt::get_position();
                print!("{}", &vt::CursorControl::Up{count: 1+(x/width) as u32});
                move_to(cpos.0 as usize, opos.0 as usize);
            }
        }
    }
    print!("\r{}{}\r{}", prompt(), " ".repeat(s), prompt());
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

fn complete(ln: &mut LinkedList<u8>, other: String) {
    for c in other.as_bytes().iter() { ln.push_back(*c); }
}
fn remove_word(ln: &mut LinkedList<u8>) {
    match ln.back() {
        None => { return; }, Some(c) => { if *c == 32 { return; } }
    }
    unsafe { TABSTATE.as_mut().unwrap().removed += 1; }
    ln.pop_back(); remove_word(ln);
}

pub fn read_command(history: &mut VecDeque<LinkedList<u8>>) -> LinkedList<u8> {
    print!("{}", prompt());
    let mut ln = LinkedList::new();
    let mut pos = 0;
    let mut escape = false;
    let mut sequence = vec!();
    let mut histpos = history.len();
    let mut stringpos = 0;
    unsafe {
        if TABSTATE.is_none() { TABSTATE = Some(TabState::new()); }
    }
    loop {
        let c = read!();
        match c {
            10 | 13 => { //newline
                unsafe {
                    if TABSTATE.as_ref().unwrap().pressed {
                        TABSTATE.as_mut().unwrap().clear = true;
                        TABSTATE.as_mut().unwrap().tab = true;
                        print_split_command(&mut ln, stringpos, false, |ln|{;});
                    }
                }
                println!("");
                ln.push_back(32); //make life easier for the parser
                unsafe { TABSTATE = None; }
                return ln;
            },
            12 => { // ^L
            },
            9 => { //tab
                unsafe {
                    if TABSTATE.as_ref().unwrap().start.is_empty() && !TABSTATE.as_ref().unwrap().pressed {
                        let mut tw = vec![];
                        for (i, b) in ln.iter().enumerate() {
                            if i == stringpos { break; }
                            if *b == 32 { tw.clear(); continue; }
                            tw.push(*b);
                        }
                        TABSTATE.as_mut().unwrap().start = tw;
                    }
                    TABSTATE.as_mut().unwrap().tab = true;
                    print_split_command(&mut ln, stringpos, false, |ln|{;});
                    match TABSTATE.as_ref().unwrap().get_completion() {
                        None => {}, Some(s) => {
                            let inc = s.len();
                            if TABSTATE.as_mut().unwrap().pressed {
                                let start = String::from_utf8(TABSTATE.as_ref().unwrap().start.clone()).unwrap();
                                print_split_command(&mut ln, stringpos, false, |ln|{remove_word(ln);complete(ln,start.clone())});
                                //print!("{}{}{}START:{}{}{}", &vt::CursorControl::SavePos, &vt::CursorControl::Up{count: 4+TABSTATE.as_ref().unwrap().index as u32}, &vt::CF_RED, start, &vt::CF_STANDARD, &vt::CursorControl::LoadPos);
                                stringpos -= TABSTATE.as_ref().unwrap().removed - start.len();
                                TABSTATE.as_mut().unwrap().removed  = 0;
                            }
                            TABSTATE.as_mut().unwrap().pressed = true;
                            TABSTATE.as_mut().unwrap().tab = false;
                            print_split_command(&mut ln, stringpos, false, |ln|complete(ln,s.clone()));
                            stringpos += inc;
                        }
                    }
                    TABSTATE.as_mut().unwrap().pressed = true;
                    TABSTATE.as_mut().unwrap().tab = false;
                    TABSTATE.as_mut().unwrap().index += 1;
                }
            },
            127 => { //backspace
                unsafe {
                    if TABSTATE.as_ref().unwrap().pressed {
                        TABSTATE.as_mut().unwrap().clear = true;
                        TABSTATE.as_mut().unwrap().tab = true;
                        print_split_command(&mut ln, stringpos, false, |ln|{;});
                    }
                    TABSTATE.as_mut().unwrap().clear();
                }
                if stringpos > 0 {
                    print_split_command(&mut ln, stringpos, false, |ln: &mut LinkedList<u8>| {ln.pop_back();});
                    stringpos -= 1;
                }
            },
            27 => { //escape
                escape = true;
            },
            _ => {
                unsafe {
                    if TABSTATE.as_ref().unwrap().pressed {
                        TABSTATE.as_mut().unwrap().clear = true;
                        TABSTATE.as_mut().unwrap().tab = true;
                        print_split_command(&mut ln, stringpos, false, |ln|{;});
                    }
                    TABSTATE.as_mut().unwrap().clear();
                }
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
