use driver::serial::*;
use utils::parser::Argument;
use alloc::vec::Vec;

pub fn exec(args: Vec<Argument>) {
    let mut c = read!();
    while c != 4 { //4 = ^d = end of transmission
        if args.len() == 0 {
            print!("{}", c as char);
        } else {
            print!("{} ", c);
        }
        c = read!();
    }
    println!("");
}
