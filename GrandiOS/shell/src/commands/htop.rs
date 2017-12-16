use core::fmt::Write;
use utils::parser::Argument;
use utils::vt;
use alloc::string::String;
use core::str;
use alloc::slice::SliceConcatExt;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;
use swi::TCBStatistics;

struct HtopData {
    selected_row: usize,
    selected_column: usize,
    color_selected: vt::Color,
    color_: vt::Color,
    num_of_static_rows: usize,
}

pub fn exec(args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
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
    let mut tcbs = tcbs_statistics!();
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
    Ok(VecDeque::new())
}

fn draw(htop_data: &HtopData, tcbs: &Vec<TCBStatistics>) -> usize {
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

fn kill_selected(htop_data: &HtopData, tcbs: &mut Vec<TCBStatistics>) {
    //TODO
    if tcbs.len() > htop_data.selected_row {
        tcbs.remove(htop_data.selected_row);
    }
}

fn get_memory(tcb: &TCBStatistics) -> (u32,char) {
    //TODO
    let mut mem = tcb.memory_allocated;
    let mut e = 'B';

    if mem/1024 >= 1 {
        mem /= 1024;
        e = 'K';
    }
    if mem/1024 >= 1 {
        mem /= 1024;
        e = 'M';
    }
    if mem/1024 >= 1 {
        mem /= 1024;
        e = 'G';
    }
    if mem/1024 >= 1 {
        mem /= 1024;
        e = 'T';
    }

    (mem, e)
}
