use core::fmt::Write;
use utils::parser::Argument;
use utils::vt;
use alloc::string::String;
use core::str;
use alloc::slice::SliceConcatExt;
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
    let mut tcbs = vec!(
        TCBTestData::new("I".to_string(),1024),
        TCBTestData::new("find".to_string(),1024),
        TCBTestData::new("your".to_string(),1024),
        TCBTestData::new("lack".to_string(),1024),
        TCBTestData::new("of".to_string(),1024),
        TCBTestData::new("faith".to_string(),8),
        TCBTestData::new("disturbing".to_string(),1024),
        TCBTestData::new(".".to_string(),1024),
        TCBTestData::new("- Darth Vader".to_string(),1024),
        TCBTestData::new("ultra langer total bekloppter nicht viel sinnmachender name der hoffentlich lang genug ist um verkürzt zu werden".to_string(),24)
        );
        tcbs[0].memory.push(4);
        for i in 0..1024 {
            tcbs[6].memory.push((i%256)as u8);
        }
    let mut htop_data = HtopData{
        selected_row:0,
        selected_column:0,
        color_: vt::Color{ct: vt::ColorType::Background, cc: vt::ColorCode::Bit8(019)},
        color_selected: vt::Color{ct: vt::ColorType::Background, cc: vt::ColorCode::Bit8(033)},
        num_of_static_rows: 2,
    };
    let mut c;
    print!("{}",&vt::CursorControl::Hide);
    let mut num_of_dynamic_rows;
    loop {
        num_of_dynamic_rows = draw(&htop_data, &tcbs); //draw htop (all)
        print!("{}{}", '\r', &vt::CursorControl::Up{count:(tcbs.len() + htop_data.num_of_static_rows + num_of_dynamic_rows) as u32});
        c = read!();
        if c == 27 { // Escape
            read!(); // [
            c=read!(); // Up/Down/etc
        } else if c == 11 {
            kill_selected(&htop_data, &mut tcbs);
        }
        match vt::parse_input(str::from_utf8(&[c]).unwrap()) {
            vt::Input::Up => {
                if htop_data.selected_row > 0 {
                    htop_data.selected_row -= 1;
                }
            },
            vt::Input::Down => {
                if htop_data.selected_row < tcbs.len()-1 {
                    htop_data.selected_row += 1;
                }
            },
            _ => {}
        }
        if c == 4 { break; } //4 = ^d = end of transmission
    }
    print!("{}",&vt::CursorControl::Show);
    print!("{}{}", '\r', &vt::CursorControl::Down{count:(tcbs.len() + htop_data.num_of_static_rows + num_of_dynamic_rows) as u32});
    Ok(vec![])
}

fn draw(htop_data: &HtopData, tcbs: &Vec<TCBTestData>) -> usize {
    // cpu time  unfegähr so?  00[e] 00[e]
    // table header colors
    let c_i = if htop_data.selected_column == 0 {&htop_data.color_selected} else {&htop_data.color_};
    let c_p = if htop_data.selected_column == 1 {&htop_data.color_selected} else {&htop_data.color_};
    let c_s = if htop_data.selected_column == 2 {&htop_data.color_selected} else {&htop_data.color_};
    let c_t = if htop_data.selected_column == 3 {&htop_data.color_selected} else {&htop_data.color_};
    let c_m = if htop_data.selected_column == 4 {&htop_data.color_selected} else {&htop_data.color_};
    let c_n = if htop_data.selected_column == 5 {&htop_data.color_selected} else {&htop_data.color_};
    let headersize = 34; // table header size without "Name"-column
    let termsize = vt::get_size();
    let mut spaces = termsize.0 as usize - (headersize + 4);
    //print table header
    println!("{}{:^10}{}{:3}{}{:^7}{}{:>8}{}{:>5}{} {:<name_width$}{}",
             c_i, "ID", c_p, "PRI", c_s, "State", c_t, "CPU_Time", c_m, "Mem", c_n, "Name", &vt::CB_STANDARD,
             name_width=(4+spaces)); // named arguments
    for (i,tcb) in tcbs.iter().enumerate() {
        // row colors
        let cb = if i == htop_data.selected_row { &htop_data.color_selected } else { &vt::CB_STANDARD };
        let cf = if i == htop_data.selected_row { &vt::CF_BLACK } else { &vt::CF_STANDARD };
        spaces = termsize.0 as usize - headersize;
        // TODO: namen scrollen ( <-> )
        // shorten name if needed
        let mut name = if tcb.name.len()>=spaces && spaces > 2 { [&tcb.name[0..(usize::min(tcb.name.len(),spaces)-2)], ".."].join("") } else { tcb.name[0..usize::min(tcb.name.len(),spaces)].to_string() };

        // print row/tcb
        let mem = get_memory(tcb);
        println!("{}{}{:10}{:3}{:^7}{:>8}{:>4}{:1} {:<name_width$}{}{}",
               cf, cb, tcb.id, tcb.priority,"TODO", tcb.cpu_time, mem.0, mem.1, name, &vt::CB_STANDARD, &vt::CF_STANDARD,
               name_width=(spaces)); // named arguments
    }
    let num_of_dynamic_rows=(u32::min(termsize.1,256) as usize)-(tcbs.len()+htop_data.num_of_static_rows+1);
    for i in 0..num_of_dynamic_rows {
        println!("{:termsize$}","",termsize=(termsize.0 as usize));
    }
    // print controls menu
    println!("^k{0}{1}Kill{2}{3}^d{0}{1}Quit{2}{3}",
             &htop_data.color_selected, &vt::CF_BLACK, &vt::CB_STANDARD, &vt::CF_STANDARD);
    num_of_dynamic_rows
}

fn kill_selected(htop_data: &HtopData, tcbs: &mut Vec<TCBTestData>) {
    //TODO
    if tcbs.len() > htop_data.selected_row {
        tcbs.remove(htop_data.selected_row);
    }
}

fn get_memory(tcb: &TCBTestData) -> (usize,char) {
    //TODO
    let mut mem = tcb.memory.len()*8;
    let mut e = 'b';

    if mem/1024 >= 1 {
        mem /= 1024;
        e = 'M';
    }

    (mem, e)
}
