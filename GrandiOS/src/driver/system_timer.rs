//Dokumentation: S.305

use core::ptr::{read_volatile, write_volatile};

pub const ST_BASE_ADDRESS : u32 = 0xFFFF_FD00;

static mut SYSTEM_TIMER: Option<STController> = None;

pub unsafe fn init(){
    SYSTEM_TIMER = Some(STController::new(ST_BASE_ADDRESS));
}

pub unsafe fn get_system_timer() -> &'static mut STController {
    match SYSTEM_TIMER {
        Some(ref mut st) => &mut *st,
        None => panic!(),
    }
}

#[repr(C)]
struct STMemoryMap{
    cr: u32,	//control regiset
    pimr: u32,	//period interval mode register
    wdmr: u32,	//watchdog mode register
    rtmr: u32,	//real-time mode register
    sr: u32,    //status register
    ier: u32,   //interrupt enable register
    idr: u32,   //interrupt disable register
    imr: u32,   //interrupt mask register
    rtar: u32,  //real-time alarm register
    crtr: u32,  //current real-time register
}

pub struct STController{
    st: *mut STMemoryMap,
    crtr_count: u32, //counter for he real-time timer with bigger capacity
    crtr_last: u32,
}

impl STController {
    //Marked unsafe because is only safe assuming the base_adress is correct
    pub unsafe fn new(base_address: u32) -> Self{
        let mut st = STController{
            st: base_address as *mut STMemoryMap,
            crtr_count: 0,
            crtr_last: 0,
        };
        st.set_rtpres(0x1001); //sets the real time clock prescaler to 0x1000 ticks = 125ms
        return st;
    }
    //pits stuff
    pub fn set_piv(&mut self, val: u16) { //0x8000 entspricht mit default slowclock einstellungen einer sekunde.
        unsafe{ write_volatile(&mut (*(self.st)).pimr, val as u32); }
    }
    pub fn interrupt_enable_pits(&mut self) {
        unsafe{ write_volatile(&mut (*(self.st)).ier, 1<<0); }
    }
    pub fn interrupt_disable_pits(&mut self) {
        unsafe{ write_volatile(&mut (*(self.st)).idr, 1<<0); }
    }
    pub fn interrupt_get_pits(&mut self) -> bool {
        unsafe{ (read_volatile(&mut (*(self.st)).imr)&(1<<0))==(1<<0) }
    }
    //real time suff
    pub fn set_rtpres(&mut self, val: u16){
        unsafe{ write_volatile(&mut (*(self.st)).rtmr, val as u32); }
    }
    pub fn get_current_time(&mut self) -> u32 { //returns the current time in ticks, defaults to 125ms
        let crtr_current = unsafe{read_volatile(&mut (*(self.st)).crtr)};
        if crtr_current < self.crtr_last {
            //the counter wrapped around
            self.crtr_count += crtr_current;
            self.crtr_count += (0xFFFFF - self.crtr_last);
        } else {
            self.crtr_count += (crtr_current - self.crtr_last);
        }
        self.crtr_last = crtr_current;
        return crtr_current;
    }
    //general stuff
    pub fn check_timers(&mut self) -> (bool, bool, bool, bool){
        let reg = unsafe{read_volatile(&mut (*(self.st)).sr)};
        let pits   = (reg & (1<<0)) > 0;
        let wdovf  = (reg & (1<<1)) > 0;
        let rttinc = (reg & (1<<2)) > 0;
        let alms   = (reg & (1<<3)) > 0;
        return (pits, wdovf, rttinc, alms)
    }
}
