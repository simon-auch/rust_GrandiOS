use core::fmt::Write;
use utils::vt;
use utils::parser::Argument;
use core::result::Result;
use alloc::string::String;
use alloc::vec_deque::VecDeque;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    println!("{}Red on Black {}White on Black {}{}Red on Green {}{}White on Black{}{}{} Standard", &vt::CF_RED, &vt::CF_WHITE, &vt::CF_RED, &vt::CB_GREEN, &vt::CF_WHITE, &vt::CB_BLACK, &vt::ATT_RESET, &vt::CF_STANDARD, &vt::CB_STANDARD);
    println!("\x1B[38;2;255;0;0m 24-Bit Color? {}", &vt::CF_STANDARD);
    println!("8-Bit Color Table:");
    for i in 0..16{
        for j in 0..16{
            print!("{}{:03} ", &vt::Color{ct: vt::ColorType::Background, cc: vt::ColorCode::Bit8(i*16+j)}, (i*16+j))
        }
        println!("{}", &vt::CB_STANDARD);
    }
    Ok(VecDeque::new())
}
