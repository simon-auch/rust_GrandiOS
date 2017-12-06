use core::fmt::Write;
use utils::vt;
use utils::parser::Argument;
use core::result::Result;
use alloc::string::String;
use alloc::vec::Vec;

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    let mut c = read!();
    let (w, _) = vt::get_size();
    println!("");
    while c != 4 { //4 = ^d = end of transmission
        for i in 0..200 {
            print!("{}", c as char);
            if i%w==0 { print!("\r"); }
            print!(" {}", vt::CursorControl::Left{count:1});
            for j in 1..(if i%w==w/2 {10} else {1} * 1000) {} //do nothing, hopefully
            if i%w==0 { print!("\r"); }
        }
        c = read!();
    }
    Ok(vec![])
}
