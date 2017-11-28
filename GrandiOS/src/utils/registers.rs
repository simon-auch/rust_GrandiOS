//must be inlined because of obvious reasons...
#[inline(always)]
pub fn get_lr() -> u32{
    let lr: u32;
    unsafe{asm!("mov r0, r14":"={r0}"(lr):::"volatile")}
    lr
}

//ein paar hilfreiche werte um einen wert für das cpsr register zu bauen
pub const CPSR_MODE_MASK      : u32 = 0x1F;
pub const CPSR_MODE_USER      : u32 = 0x10;
pub const CPSR_MODE_FIQ       : u32 = 0x11;
pub const CPSR_MODE_IRQ       : u32 = 0x12;
pub const CPSR_MODE_SVC       : u32 = 0x13;
pub const CPSR_MODE_ABORT     : u32 = 0x17;
pub const CPSR_MODE_UNDEFINED : u32 = 0x1B;
pub const CPSR_MODE_SYS       : u32 = 0x1F;

pub const CPSR_INSTRUCTION_MASK    : u32 = 0x1 << 5;
pub const CPSR_INSTRUCTION_ARM     : u32 = 0x0 << 5;
pub const CPSR_INSTRUCTION_THUMB   : u32 = 0x1 << 5;

pub const CPSR_IRQ : u32 = 0x1 << 7;
pub const CPSR_FIQ : u32 = 0x1 << 6;
