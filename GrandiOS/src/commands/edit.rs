use driver::serial::*;
use utils::parser::Argument;
use utils::shell::*;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use core::ptr::{write_volatile, read_volatile};

pub fn move_col(pos: usize, dest: usize) {
    let offset = [0,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0];
    if dest < pos {
        for i in dest..pos {
            print!("{}[{}{}", 27 as char, offset[i+1]+1, EscapeSequence::Left.to_string());
        }
    } else {
        for i in pos..dest {
            print!("{}[{}{}", 27 as char, offset[i+1]+1, EscapeSequence::Right.to_string());
        }
    }
}

fn print_line(i: usize) {
    let line = unsafe { read_volatile(i as *mut [u8; 16]) };
    println!("{:08x}   {:02x}{:02x} {:02x}{:02x}  {:02x}{:02x} {:02x}{:02x}  {:02x}{:02x} {:02x}{:02x}  {:02x}{:02x} {:02x}{:02x}   {}",
             i, line[0], line[1], line[2], line[3], line[4], line[5], line[6],
             line[7], line[8], line[9], line[10], line[11], line[12], line[13],
             line[14], line[15], get_string(line.clone()));
}

pub fn exec(args: Vec<Argument>) {
    if args.len() < 2 {
        println!("Address of start and end needed!");
        return;
    }
    let start = args[0].get_int().expect("Start is not a number");
    let end = args[1].get_int().expect("End is not a number");
    if end <= start {
        println!("Invalid range!");
    }
    let width = 16;
    let mut pos = 0;
    let mut linepos = 0;
    let mut line: [u8; 16] = [0; 16];
    let lines = (end-start)/width+(if (end-start)%width==0 { 0 } else { 1 });
    for i in 0..lines { print_line(start+i*width); }
    print!("{}{}[11C{}[{}A", '\r', 27 as char, 27 as char, lines);
    let mut escape = false;
    let mut sequence = vec!();
    let mut c = read!();
    while c != 4 { //4 = ^d = end of transmission
        if !escape && ((48..58).contains(c) || (65..71).contains(c) || (97..103).contains(c)) {
            let mut v: u8 = c - 48;
            if v > 9 { v = v - 39; }
            if v < 0 { v = v + 32; }
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
            match parse_escape(sequence) {
                EscapeSequence::Home => {
                    move_col(pos, 0);
                    pos = 0;
                },
                EscapeSequence::End => {
                    move_col(pos, 4*8-1);
                    pos = 4*8-1;
                },
                EscapeSequence::Left => {
                    if pos > 0 {
                        move_col(pos, pos-1);
                        pos -= 1;
                    }
                },
                EscapeSequence::Right => {
                    if pos < 4*8-1 {
                        move_col(pos, pos+1);
                        pos += 1;
                    }
                },
                EscapeSequence::Up => {
                    if linepos > 0 {
                        print!("{}[{}", 27 as char, EscapeSequence::Up.to_string());
                        linepos -= 1;
                    }
                },
                EscapeSequence::Down => {
                    if linepos < lines-1 {
                        print!("{}[{}", 27 as char, EscapeSequence::Down.to_string());
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
}

fn get_string(mut line: [u8; 16]) -> String {
    for i in 0..(line.len()) {
        if line[i] < 32 || line[i] >= 127 {
            line[i] = 46;
        }
    }
    String::from_utf8_lossy(&line).into_owned()
}
