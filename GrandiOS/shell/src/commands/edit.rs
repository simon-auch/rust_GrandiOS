use core::fmt::Write;
use utils::parser::Argument;
use utils::vt;
use core::str;
use core::result::Result;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use core::ptr::{write_volatile, read_volatile};

pub fn move_col(pos: usize, dest: usize) {
    let offset = [0,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0];
    if dest < pos {
        for i in dest..pos {
            print!("{}", &vt::CursorControl::Left{count: offset[i+1]+1});
        }
    } else {
        for i in pos..dest {
            print!("{}", &vt::CursorControl::Right{count: offset[i+1]+1});
        }
    }
}

fn print_line(i: usize) {
    let line = unsafe { read_volatile(i as *mut [u8; 16]) };
    print!("{:08x}   {:02x}{:02x} {:02x}{:02x}  {:02x}{:02x} {:02x}{:02x}  {:02x}{:02x} {:02x}{:02x}  {:02x}{:02x} {:02x}{:02x}   ",
             i, line[0], line[1], line[2], line[3], line[4], line[5], line[6],
             line[7], line[8], line[9], line[10], line[11], line[12], line[13],
             line[14], line[15]);
    for i in 0..(line.len()) {
        print!("{}", if line[i] < 32 || line[i] >= 127 {
            48 } else { line[i] } as char);
    }
    println!("");
}

pub fn exec(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    ::eval_args(&mut args, 0);
    if args.len() < 1 {
        return Err("Start address and an optional length needed!".to_string());
    }
    if !args[0].is_int() && !args[0].is_method() {
        return Err("Invalid argument for start address".to_string());
    }
    if args.len() > 1 && !args[1].is_int() {
        return Err("Invalid argument for length".to_string());
    }
    let width = 16;
    let start = if args[0].is_int() {
        args[0].get_int().unwrap()
    } else {
        (::get_function(args[0].clone()).unwrap()) as usize
    };
    let length = if args.len() > 1 { args[1].get_int().unwrap() } else { width*8 };
    if length <= 0 {
        return Err("Invalid length!".to_string());
    }
    let mut pos = 0;
    let mut linepos = 0;
    let mut line: [u8; 16] = [0; 16];
    let lines = length/width+(if length%width==0 { 0 } else { 1 });
    for i in 0..lines { print_line(start+i*width); }
    print!("{}{}[11C{}[{}A", '\r', 27 as char, 27 as char, lines);
    let mut escape = false;
    let mut sequence = vec!();
    let mut c = read!();
    while c != 4 { //4 = ^d = end of transmission
        if !escape && ((48..58).contains(c) || (65..71).contains(c) || (97..103).contains(c)) {
            let mut v: u8 = c - 48;
            if v > 9 { v = v - 7; }
            if v > 9 { v = v - 32; }
            let b: u8 = unsafe { read_volatile((start+linepos*width+pos/2) as *mut u8) };
            if pos % 2 == 0 {
                v = v<<4 + (b & 0x0f);
            } else {
                v = v + (b & 0xf0);
            }
            unsafe {
                write_volatile((start+linepos*width+pos/2) as *mut u8, v);
            }
            print!("{}7\r", 27 as char);
            print_line(start+linepos*width);
            print!("{}8", 27 as char);
        }
        if c == 27 { escape = true; }
        if c != 91 && c >= 32 && escape { sequence.push(c); }
        if escape && ((65..69).contains(c) || c == 126) {
            escape = false;
            match vt::parse_input(str::from_utf8(&sequence[..]).unwrap()) {
                vt::Input::Home => {
                    move_col(pos, 0);
                    pos = 0;
                },
                vt::Input::End => {
                    move_col(pos, 4*8-1);
                    pos = 4*8-1;
                },
                vt::Input::Left => {
                    if pos > 0 {
                        move_col(pos, pos-1);
                        pos -= 1;
                    }
                },
                vt::Input::Right => {
                    if pos < 4*8-1 {
                        move_col(pos, pos+1);
                        pos += 1;
                    }
                },
                vt::Input::Up => {
                    if linepos > 0 {
                        print!("{}", &vt::CursorControl::Up{count: 1});
                        linepos -= 1;
                    }
                },
                vt::Input::Down => {
                    if linepos < lines-1 {
                        print!("{}", &vt::CursorControl::Down{count: 1});
                        linepos += 1;
                    }
                },
                _ => {}
            }
            sequence = vec![];
        }
        c = read!();
    }
    print!("{}", "\n".repeat(lines-linepos));
    Ok(vec![])
}
