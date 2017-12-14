//Dokumentation: S.276

use core::ptr::{read_volatile, write_volatile};
use driver::serial::*;

pub const PMC_BASE_ADDRESS : u32 = 0xFFFFFC00;

static mut POWER_MANAGEMENT_CONTROLLER: Option<PowerManagementController> = None;

pub unsafe fn init(){
    POWER_MANAGEMENT_CONTROLLER = Some(PowerManagementController::new(PMC_BASE_ADDRESS));
}

pub unsafe fn get_power_management_controller() -> &'static mut PowerManagementController {
    match POWER_MANAGEMENT_CONTROLLER {
        Some(ref mut pmc) => &mut *pmc,
        None => panic!(),
    }
}

#[repr(C)]
struct PowerManagementMemoryMap{
    pmc_scer: u32,	//system clock enable register
    pmc_scdr: u32,	//system clock disable register
    pmc_scsr: u32,	//system clock status register
    reserved_0: [u8; 0x10-0x0C],
    pmc_pcer: u32,	//peripheral clock enable register
    pmc_pcdr: u32,	//peripheral clock disable register
    pmc_pcsr: u32,	//peripheral clock status register
    reserved_1: [u8; 0x20-0x1C],
    ckgr_mor: u32,	//main oscillator register
    ckgr_mcfr: u32,	//main clock frequency register
    ckgr_pllar: u32,	//PLL A register
    ckgr_pllbr: u32,	//PLL B register
    pmc_mckr: u32,	//master clock register
    reserved_2: [u8; 0x40-0x34],
    pmc_pck0: u32,	//programmable clock 0 register
    pmc_pck1: u32,	//programmable clock 1 register
    pmc_pck2: u32,	//programmable clock 2 register
    pmc_pck3: u32,	//programmable clock 3 register
    reserved_3: [u8; 0x60-0x50],
    pmc_ier: u32,	//interrupt enable register
    pmc_idr: u32,	//interrupt disable register
    pmc_sr: u32,	//status register
    pmc_imr: u32,	//interrupt mask register
}

pub struct PowerManagementController{
	pmc: *mut PowerManagementMemoryMap,
}

impl PowerManagementController {
    //Marked unsafe because is only safe assuming the base_adress is correct
    pub unsafe fn new(base_adress: u32) -> Self{
        PowerManagementController{
            pmc: base_adress as *mut PowerManagementMemoryMap,
        }
    }
    pub fn sc_enable_pck(&mut self) {
        unsafe{ write_volatile(&mut (*(self.pmc)).pmc_scer, 1); }
    }
    pub fn sc_disable_pck(&mut self) {
        unsafe{ write_volatile(&mut (*(self.pmc)).pmc_scdr, 1); }
    }
    pub fn sc_get_pck(&mut self) -> bool {
        unsafe{ (read_volatile(&mut (*(self.pmc)).pmc_scsr)&1)==1 }
    }
    pub fn mc_select_slow_clock(&mut self) {
        unsafe{
            let register = &mut (*(self.pmc)).pmc_mckr;
            write_volatile(register, read_volatile(register) & 0xFFFFFFFC0 | 0x0);
        }
    }
    pub fn mc_select_main_clock(&mut self) {
        unsafe{
            let register = &mut (*(self.pmc)).pmc_mckr;
            write_volatile(register, read_volatile(register) & 0xFFFFFFFC0 | 0x1);
        }
    }
    pub fn mc_select_pll_a(&mut self) {
        unsafe{
            let register = &mut (*(self.pmc)).pmc_mckr;
            write_volatile(register, read_volatile(register) & 0xFFFFFFFC0 | 0x2);
        }
    }
    pub fn mc_select_pll_b(&mut self) {
        unsafe{
            let register = &mut (*(self.pmc)).pmc_mckr;
            write_volatile(register, read_volatile(register) & 0xFFFFFFFC0 | 0x3);
        }
    }
    pub fn sc_get_raw(&mut self) -> u32 {
        unsafe{ read_volatile(&mut (*(self.pmc)).pmc_scsr) }
    }
    pub fn interrupt_enable(&mut self) {
        unsafe{ write_volatile(&mut (*(self.pmc)).pmc_ier, 1<<3); }
    }
    pub fn interrupt_disable(&mut self) {
        unsafe{ write_volatile(&mut (*(self.pmc)).pmc_idr, 1<<3); }
    }
    pub fn interrupt_get(&mut self) -> bool {
        unsafe{ (read_volatile(&mut (*(self.pmc)).pmc_imr)&(1<<3))==(1<<3) }
    }
    pub fn interrupt_get_raw(&mut self) -> u32 {
        unsafe{ read_volatile(&mut (*(self.pmc)).pmc_imr) }
    }
}
