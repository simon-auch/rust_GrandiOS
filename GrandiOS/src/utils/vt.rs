//See: http://www.termsys.demon.co.uk/vtansi.htm
//This file only supports sending Control Sequences, not parsing received ones.
//Sections that are missing:
//Scrolling, Tab Control, Erasing Text, Printing, Define Key


//"\x1B" is the Escape character.

use core::fmt;

//Some public constants that are often used
pub const CF_BLACK:   Color = Color { ct: ColorType::Foreground, cc: ColorCode::Black};
pub const CF_RED:     Color = Color { ct: ColorType::Foreground, cc: ColorCode::Red};
pub const CF_GREEN:   Color = Color { ct: ColorType::Foreground, cc: ColorCode::Green};
pub const CF_YELLOW:  Color = Color { ct: ColorType::Foreground, cc: ColorCode::Yellow};
pub const CF_BLUE:    Color = Color { ct: ColorType::Foreground, cc: ColorCode::Blue};
pub const CF_MAGENTA: Color = Color { ct: ColorType::Foreground, cc: ColorCode::Magenta};
pub const CF_CYAN:    Color = Color { ct: ColorType::Foreground, cc: ColorCode::Cyan};
pub const CF_WHITE:   Color = Color { ct: ColorType::Foreground, cc: ColorCode::White};

pub const CB_BLACK:   Color = Color { ct: ColorType::Background, cc: ColorCode::Black};
pub const CB_RED:     Color = Color { ct: ColorType::Background, cc: ColorCode::Red};
pub const CB_GREEN:   Color = Color { ct: ColorType::Background, cc: ColorCode::Green};
pub const CB_YELLOW:  Color = Color { ct: ColorType::Background, cc: ColorCode::Yellow};
pub const CB_BLUE:    Color = Color { ct: ColorType::Background, cc: ColorCode::Blue};
pub const CB_MAGENTA: Color = Color { ct: ColorType::Background, cc: ColorCode::Magenta};
pub const CB_CYAN:    Color = Color { ct: ColorType::Background, cc: ColorCode::Cyan};
pub const CB_WHITE:   Color = Color { ct: ColorType::Background, cc: ColorCode::White};


pub enum DeviceStatus{
    QueryDeviceCode,
    QueryDeviceStatus,
    QueryCursorPosition,
}

pub enum TerminalSetup {
    ResetDevice,
    EnableLineWrap,
    DisableLineWrap,
}

pub enum Fonts {
    FontSetG0,
    FontSetG1,
}

pub enum CursorControl {
    Home {row: u32, col: u32},
    Up {count: u32},
    Down {count: u32},
    Forward {count: u32},
    Backward {count: u32},
    Position {row: u32, col: u32},
    SavePos,
    LoadPos,
    SavePosAndAtt,
    LoadPosAndAtt,
}

pub enum DisplayAttributes{
    Reset,
    Bright,
    Dim,
    Underscore,
    Blink,
    Reverse,
    Hidden,
}

pub struct Color{
    pub ct: ColorType,
    pub cc: ColorCode,
}

pub enum ColorType{
    Foreground,
    Background,
}

pub enum ColorCode{
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let offset = match self.ct{
            ColorType::Foreground => 30,
            ColorType::Background => 40,
        };
        write!(f, "\x1B[{}m", offset + match self.cc{
            ColorCode::Black   => 0,
            ColorCode::Red     => 1,
            ColorCode::Green   => 2,
            ColorCode::Yellow  => 3,
            ColorCode::Blue    => 4,
            ColorCode::Magenta => 5,
            ColorCode::Cyan    => 6,
            ColorCode::White   => 7,
        })
    }
}

impl fmt::Display for DisplayAttributes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B{}", match self{
            &DisplayAttributes::Reset      => 0,
            &DisplayAttributes::Bright     => 1,
            &DisplayAttributes::Dim        => 2,
            &DisplayAttributes::Underscore => 4,
            &DisplayAttributes::Blink      => 5,
            &DisplayAttributes::Reverse    => 7,
            &DisplayAttributes::Hidden     => 8,
        })
    }
}

impl fmt::Display for CursorControl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self{
            &CursorControl::Home {row, col}     => write!(f, "\x1B[{};{}H", row, col),
            &CursorControl::Up {count}          => write!(f, "\x1B[{}A", count),
            &CursorControl::Down {count}        => write!(f, "\x1B[{}B", count),
            &CursorControl::Forward {count}     => write!(f, "\x1B[{}C", count),
            &CursorControl::Backward {count}    => write!(f, "\x1B[{}D", count),
            &CursorControl::Position {row, col} => write!(f, "\x1B[{};{}f", row, col),
            &CursorControl::SavePos             => write!(f, "\x1B[s"),
            &CursorControl::LoadPos             => write!(f, "\x1B[u"),
            &CursorControl::SavePosAndAtt       => write!(f, "\x1B7"),
            &CursorControl::LoadPosAndAtt       => write!(f, "\x1B8"),
        }
    }
}

impl fmt::Display for Fonts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &Fonts::FontSetG0 => "\x1B(",
            &Fonts::FontSetG1 => "\x1B)",
        })
    }
}

impl fmt::Display for TerminalSetup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &TerminalSetup::ResetDevice => "\x1Bc",
            &TerminalSetup::EnableLineWrap => "\x1B[7h",
            &TerminalSetup::DisableLineWrap => "\x1B[7l",
        })
    }
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &DeviceStatus::QueryDeviceCode => "\x1B[c",
            &DeviceStatus::QueryDeviceStatus => "\x1B[5n",
            &DeviceStatus::QueryCursorPosition => "\x1B[6n",
        })
    }
}

