use core::fmt::Write;
use utils::parser::Argument;
use core::result::Result;
use alloc::string::String;
use alloc::vec::Vec;

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    let mut c = read!();
    while c != 4 { //4 = ^d = end of transmission
        if args.len() == 0 {
            print!("{}", c as char);
        } else {
            print!("{} ", c);
        }
        c = read!();
    }
    Ok(vec![])
}
