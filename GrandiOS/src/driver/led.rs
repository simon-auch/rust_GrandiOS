use core::ptr::write_volatile;

pub const PIO_B: u32 = 0xfffff600;
pub const PIO_C: u32 = 0xfffff800;

pub const PIO_LED_YELLOW: PIO_PIN = PIO_PIN{ base_adress: PIO_B, pin: 1 << 27 };
pub const PIO_LED_RED:    PIO_PIN = PIO_PIN{ base_adress: PIO_C, pin: 1 <<  0 };
pub const PIO_LED_GREEN:  PIO_PIN = PIO_PIN{ base_adress: PIO_C, pin: 1 <<  1 };

pub struct PIO_PIN{
	base_adress: u32,
	pin: u32,
}

#[repr(C)]
struct HW_PIO{
	per: u32,
	un0: [u32; 3],
	oer: u32,
	un1: [u32; 7],
	sodr: u32,
}

pub struct PIO{
	hw_pio: *mut HW_PIO,
	pin: u32,
}


impl PIO {
	//Marked unsafe because self.on is only safe assuming the base_adress and pin are correct
	pub unsafe fn new(pio_pin: PIO_PIN) -> Self{
		PIO{
			hw_pio: pio_pin.base_adress as *mut HW_PIO,
			pin: pio_pin.pin,
		}
	}
	pub fn on(&mut self){
		unsafe {
		//Initialisieren
		write_volatile(&mut (*(self.hw_pio)).per, self.pin);
		write_volatile(&mut (*(self.hw_pio)).oer, self.pin);
		//Anschalten
		write_volatile(&mut (*(self.hw_pio)).sodr, self.pin);
		}
	}
}
