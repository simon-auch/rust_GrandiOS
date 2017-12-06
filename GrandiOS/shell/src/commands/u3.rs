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
        for i in 1..200 {
            print!("{} {}", c as char, vt::CursorControl::Left{count:1});
            if i%w==w/2 {
                for j in 1..100 {} //do nothing, hopefully
            }
            if i%w==0 { print!("\r"); }
        }
        c = read!();
    }
    Ok(vec![])
}
