use core::fmt::Write;
use utils::parser::Argument;
use utils::vt;
use alloc::string::String;
use core::str;
use alloc::string::ToString;
use alloc::vec::Vec;

pub struct TCBTestData {
    pub id: u32,
    pub name: String,
    pub cpu_time: u32,
    pub priority: u32,
    pub memory: Vec<u8>,
}
static mut NEXT_ID: u32 = 0;
impl TCBTestData {
    pub fn new(name: String, memory_size: usize) -> Self {
        let id;
        unsafe{
            NEXT_ID+=1;
            id=NEXT_ID;
        }
        let memory = Vec::with_capacity(memory_size);
        TCBTestData {
            id: id,
            name: name,
            cpu_time: 0,
            priority: 0,
            memory: memory,
        }
    }
}

struct HtopData {
    selected_row: usize,
    selected_column: usize,
    color_selected: vt::Color,
    color_: vt::Color,
    num_of_static_rows: usize,
}

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    //create test data
    let tcbs = vec!(
        TCBTestData::new("I".to_string(),1024),
        TCBTestData::new("find".to_string(),1024),
        TCBTestData::new("your".to_string(),1024),
        TCBTestData::new("lack".to_string(),1024),
        TCBTestData::new("of".to_string(),1024),
        TCBTestData::new("faith".to_string(),8),
        TCBTestData::new("disturbing".to_string(),1024),
        TCBTestData::new(".".to_string(),1024),
        TCBTestData::new("- Darth Vader".to_string(),1024)
        );
    let mut htop_data = HtopData{
        selected_row:0,
        selected_column:0,
        color_: vt::Color{ct: vt::ColorType::Background, cc: vt::ColorCode::Bit8(019)},
        color_selected: vt::Color{ct: vt::ColorType::Background, cc: vt::ColorCode::Bit8(033)},
        num_of_static_rows: 2,
    };
    let mut c;
    print!("{}",&vt::CursorControl::Hide);
    loop {
        draw(&htop_data, &tcbs); //draw htop (all)
        print!("{}{}", '\r', &vt::CursorControl::Up{count:(tcbs.len() + htop_data.num_of_static_rows) as u32});
        c=read!();
        match vt::parse_input(str::from_utf8(&[c]).unwrap()) {
            vt::Input::Up => {
                if htop_data.selected_row > 0 {
                    htop_data.selected_row -= 1;
                }
            },
            vt::Input::Down => {
                print!("{}",c);
                if htop_data.selected_row < tcbs.len()-1 {
                    htop_data.selected_row += 1;
                }
            },
            _ => {
                    print!("{}",c);}
        }
        if c == 4 { break; } //4 = ^d = end of transmission
    }
    print!("{}",&vt::CursorControl::Show);
    print!("{}{}", '\r', &vt::CursorControl::Down{count:(tcbs.len() + htop_data.num_of_static_rows) as u32});
    Ok(vec![])
}

fn draw(htop_data: &HtopData, tcbs: &Vec<TCBTestData>) {
    let c_c = if htop_data.selected_column == 0 {&htop_data.color_selected} else {&htop_data.color_};
    let c_p = if htop_data.selected_column == 1 {&htop_data.color_selected} else {&htop_data.color_};
    let c_s = if htop_data.selected_column == 2 {&htop_data.color_selected} else {&htop_data.color_};
    let c_t = if htop_data.selected_column == 3 {&htop_data.color_selected} else {&htop_data.color_};
    let c_m = if htop_data.selected_column == 4 {&htop_data.color_selected} else {&htop_data.color_};
    let c_n = if htop_data.selected_column == 5 {&htop_data.color_selected} else {&htop_data.color_};
    let headersize = "  ID  Priority State CPU Time Mem Name".len();
    let termsize = vt::get_size();
    let spaces = " ".repeat(termsize.0 as usize - headersize);
    println!("{}  ID  {}Priority {}State {}CPU Time {}Mem {}Name{}{}",
             c_c, c_p, c_s, c_t, c_m, c_n, spaces, &vt::CB_STANDARD);
    for (i,tcb) in tcbs.iter().enumerate() {
        let cb = if i == htop_data.selected_row { &htop_data.color_selected } else { &vt::CB_STANDARD };
        let cf = if i == htop_data.selected_row { &vt::CF_BLACK } else { &vt::CF_STANDARD };
        println!("{}{} {:5}{:8} {:5} {:8} {:3} {}{}{}", cf, cb, tcb.id, tcb.priority, "TODO", tcb.cpu_time, tcb.memory.len(), tcb.name, &vt::CB_STANDARD, &vt::CF_STANDARD);
    }
    println!("^d{}{}Quit{}{}",&htop_data.color_selected, &vt::CF_BLACK, &vt::CB_STANDARD, &vt::CF_STANDARD);
}
