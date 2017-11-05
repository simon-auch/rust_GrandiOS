use utils::spinlock;
use core::ptr::{write_volatile, read_volatile};
use core::fmt;
pub use core::fmt::Write;

const DUMM_BASE_ADRESS : u32 = 0xFFFFF200;

//lots of consts for all the register bits
const CR_RSTRX: u32 = 1 << 2;
const CR_RSTTX: u32 = 1 << 3;
const CR_RXEN:  u32 = 1 << 4;
const CR_RXDIS: u32 = 1 << 5;
const CR_TXEN:  u32 = 1 << 6;
const CR_TXDIS: u32 = 1 << 7;
const CR_RSTSTA:u32 = 1 << 8;

const SR_RXRDY:   u32 = 1 << 0;
const SR_TXRDY:   u32 = 1 << 1;
const SR_ENDRX:   u32 = 1 << 3;
const SR_ENDTX:   u32 = 1 << 4;
const SR_OVRE:    u32 = 1 << 5;
const SR_FRAME:   u32 = 1 << 6;
const SR_PARE:    u32 = 1 << 7;
const SR_TXEMPTY: u32 = 1 << 9;
const SR_TXBUFE:  u32 = 1 << 11;
const SR_RXBUFF:  u32 = 1 << 12;
const SR_COMMTX:  u32 = 1 << 30;
const SR_COMMRX:  u32 = 1 << 31;

#[repr(C)]
struct DebugUnitMemoryMap{
	cr:  u32,	//control register
	mr:  u32,	//mode register
	ier: u32,	//interrupt enable reister
	idr: u32,	//interrupt disable register
	imr: u32,	//interrupt mask register
	sr:  u32,	//status register
	rhr: u8,	//receive holding register
	reserved_0: [u8; 3],
	thr: u8,	//transmit holding register
	reserved_1: [u8; 3],
	brgr:u32,	//baud rate generator
	reserved_2: [u32; 3],
	cidr:u32,	//chip id register
	exid:u32,	//chip id extension register
}

pub struct DebugUnit{
	dumm: *mut DebugUnitMemoryMap,
}

unsafe impl Send for DebugUnit { }

impl DebugUnit {
	//Marked unsafe because self.on is only safe assuming the base_adress and pin are correct
	pub const unsafe fn new(base_adress: u32) -> Self{
		DebugUnit{
			dumm: base_adress as *mut DebugUnitMemoryMap,
		}
	}
	fn transmitter_enable(&mut self) {
		unsafe{
		write_volatile(&mut (*(self.dumm)).cr, CR_TXEN);
		}
	}
	fn transmitter_disable(&mut self) {
		unsafe{
		write_volatile(&mut (*(self.dumm)).cr, CR_TXDIS);
		}
	}
	fn transmitter_reset(&mut self) {
		unsafe{
		write_volatile(&mut (*(self.dumm)).cr, CR_RSTTX);
		}
	}
    pub fn read(&mut self) -> u8 {
        unsafe{
        while (read_volatile(&mut (*(self.dumm)).sr) & (SR_RXRDY)) == 0 {}
        read_volatile(&mut (*(self.dumm)).rhr)
        }
    }
}

impl fmt::Write for DebugUnit{
	fn write_char(&mut self, c: char) -> fmt::Result {
		unsafe{
		//make sure the last character has been written or moved to the shift register
		while (read_volatile(&mut (*(self.dumm)).sr) & (SR_TXRDY)) == 0 {}
		//write new character
		write_volatile(&mut (*(self.dumm)).thr, c as u8);
		}
		Ok(())
	}
	fn write_str(&mut self, s: &str) -> fmt::Result {
		for c in s.chars(){
			self.write_char(c).unwrap();
		}
		Ok(())
	}
	fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
		fmt::write(self, args)
	}
}

pub static DEBUG_UNIT : spinlock::Spinlock<DebugUnit> = spinlock::Spinlock::new(unsafe { DebugUnit::new(DUMM_BASE_ADRESS) });

#[allow(unused_macros)]
macro_rules! read {
    () => {{
        let mut debug_unit = DEBUG_UNIT.lock();
        debug_unit.read()
    }};
}
#[allow(unused_macros)]
macro_rules! print {
	( $x:expr ) => {{
		let mut debug_unit = DEBUG_UNIT.lock();
		write!(*debug_unit, $x).unwrap();
	}};
	( $x:expr, $( $y:expr ),* ) => {{
		let mut debug_unit = DEBUG_UNIT.lock();
		write!(*debug_unit, $x, $($y),*).unwrap();
	}};
}
#[allow(unused_macros)]
macro_rules! println {
	( $x:expr ) => {{
		let mut debug_unit = DEBUG_UNIT.lock();
		write!(*debug_unit, $x).unwrap();
		write!(*debug_unit, "\n").unwrap();
	}};
	( $x:expr, $( $y:expr ),* ) => {{
		let mut debug_unit = DEBUG_UNIT.lock();
		write!(*debug_unit, $x, $($y),*).unwrap();
		write!(*debug_unit, "\n").unwrap();
	}};
}
