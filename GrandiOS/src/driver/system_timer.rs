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
    pits: bool,   //did the period interval timer overflow?
    wdovf: bool,  //did the watchdog overflow?
    rttinc: bool, //did the real time timer increment?
    alms: bool,   //something with alarms
}

impl STController {
    //Marked unsafe because is only safe assuming the base_adress is correct
    pub unsafe fn new(base_address: u32) -> Self{
        STController{
            st: base_address as *mut STMemoryMap,
            pits:   false,
            wdovf:  false,
            rttinc: false,
            alms:   false,
        }
    }
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
    fn check_timers(&mut self) {
        let reg = unsafe{read_volatile(&mut (*(self.st)).sr)};
        self.pits   = self.pits   | ((reg & (1<<0)) > 0);
        self.wdovf  = self.wdovf  | ((reg & (1<<1)) > 0);
        self.rttinc = self.rttinc | ((reg & (1<<2)) > 0);
        self.alms   = self.alms   | ((reg & (1<<3)) > 0);
    }
    pub fn interrupt_pits(&mut self) -> bool {
        self.check_timers();
        let ret = self.pits;
        self.pits = false;
        return ret;
    }
}
