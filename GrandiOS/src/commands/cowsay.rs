use driver::serial::*;
use utils::parser::Argument;
use utils::shell::*;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() == 0 { return Err("Arguments expected".to_string()); }
    print!("{}", "\n".repeat(8));
    print!("{}[7{}< ", 27 as char, EscapeSequence::Up.to_string());
    for arg in args {
        print!("{} ", arg.to_string());
    }
    print!(">{0}[{1}{0}[2{2}", 27 as char, EscapeSequence::Up.to_string(), EscapeSequence::Left.to_string());
    let (mut x, _) = get_position();
    while x > 1 {
        print!("_{0}[{1}{0}[2{2}-{0}[2{1}{0}[2{3}", 27 as char, EscapeSequence::Left.to_string(), EscapeSequence::Down.to_string(), EscapeSequence::Up.to_string());
        x -= 1;
    }
    print!("{}[3{}", 27 as char, EscapeSequence::Down.to_string());
    print!("  \\  ^__^\r{}[{}", 27 as char, EscapeSequence::Down.to_string());
    print!("   \\ (oo)\\_______\r{}[{}", 27 as char, EscapeSequence::Down.to_string());
    print!("     (__)\\       )\\/\\\r{}[{}", 27 as char, EscapeSequence::Down.to_string());
    print!("         ||----w |\r{}[{}", 27 as char, EscapeSequence::Down.to_string());
    print!("         ||     ||\r");
    Ok(vec![])
}
