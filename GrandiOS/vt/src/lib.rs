//See: http://www.termsys.demon.co.uk/vtansi.htm
//This file only supports sending Control Sequences, not parsing received ones.
//Sections that are missing:
//Scrolling, Tab Control, Erasing Text, Printing, Define Key

#![no_std]

//"\x1B" is the Escape character.

use core::fmt;

//Some public constants that are often used
pub const CF_BLACK:    Color = Color { ct: ColorType::Foreground, cc: ColorCode::Black};
pub const CF_RED:      Color = Color { ct: ColorType::Foreground, cc: ColorCode::Red};
pub const CF_GREEN:    Color = Color { ct: ColorType::Foreground, cc: ColorCode::Green};
pub const CF_YELLOW:   Color = Color { ct: ColorType::Foreground, cc: ColorCode::Yellow};
pub const CF_BLUE:     Color = Color { ct: ColorType::Foreground, cc: ColorCode::Blue};
pub const CF_MAGENTA:  Color = Color { ct: ColorType::Foreground, cc: ColorCode::Magenta};
pub const CF_CYAN:     Color = Color { ct: ColorType::Foreground, cc: ColorCode::Cyan};
pub const CF_WHITE:    Color = Color { ct: ColorType::Foreground, cc: ColorCode::White};
pub const CF_STANDARD: Color = Color { ct: ColorType::Foreground, cc: ColorCode::Standard};

pub const CB_BLACK:    Color = Color { ct: ColorType::Background, cc: ColorCode::Black};
pub const CB_RED:      Color = Color { ct: ColorType::Background, cc: ColorCode::Red};
pub const CB_GREEN:    Color = Color { ct: ColorType::Background, cc: ColorCode::Green};
pub const CB_YELLOW:   Color = Color { ct: ColorType::Background, cc: ColorCode::Yellow};
pub const CB_BLUE:     Color = Color { ct: ColorType::Background, cc: ColorCode::Blue};
pub const CB_MAGENTA:  Color = Color { ct: ColorType::Background, cc: ColorCode::Magenta};
pub const CB_CYAN:     Color = Color { ct: ColorType::Background, cc: ColorCode::Cyan};
pub const CB_WHITE:    Color = Color { ct: ColorType::Background, cc: ColorCode::White};
pub const CB_STANDARD: Color = Color { ct: ColorType::Background, cc: ColorCode::Standard};

pub const ATT_RESET     : DisplayAttribute = DisplayAttribute::Reset;
pub const ATT_BRIGHT    : DisplayAttribute = DisplayAttribute::Bright;
pub const ATT_DIM       : DisplayAttribute = DisplayAttribute::Dim;
pub const ATT_UNDERSCORE: DisplayAttribute = DisplayAttribute::Underscore;
pub const ATT_BLINK     : DisplayAttribute = DisplayAttribute::Blink;
pub const ATT_REVERSE   : DisplayAttribute = DisplayAttribute::Reverse;
pub const ATT_HIDDEN    : DisplayAttribute = DisplayAttribute::Hidden;

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

pub enum Font {
    FontSetG0,
    FontSetG1,
}

pub enum CursorControl {
    Home {row: u32, col: u32},
    Up {count: u32},
    Down {count: u32},
    Right {count: u32},
    Left {count: u32},
    Position {row: u32, col: u32},
    SavePos,
    LoadPos,
    SavePosAndAtt,
    LoadPosAndAtt,
    Hide,
    Show,
}

#[derive(Clone)]
pub enum Input {
    Unknown,
    Left,
    Right,
    Up,
    Down,
    Delete,
    Home,
    End,
    PgUp,
    PgDn,
}

pub enum DisplayAttribute{
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
    Bit8(u8),
    Standard,
}

impl Input {
    pub fn as_str(&self) -> &str {
        match self {
            &Input::Left => "D",
            &Input::Right => "C",
            &Input::Up => "A",
            &Input::Down => "B",
            &Input::Delete => "3~",
            &Input::Home => "1~",
            &Input::End => "4~",
            &Input::PgUp => "5~",
            &Input::PgDn => "6~",
            _ => "",
        }
    }
}

pub fn parse_input(s: &str) -> Input {
    for haystick in [Input::Left, Input::Right, Input::Up, Input::Down, Input::Delete, Input::Home, Input::End, Input::PgUp, Input::PgDn].into_iter() {
        if haystick.as_str() == s { return haystick.clone(); }
    }
    Input::Unknown
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let offset = match self.ct{
            ColorType::Foreground => 30,
            ColorType::Background => 40,
        };
        match self.cc{
            ColorCode::Black    => write!(f, "\x1B[{}m", offset + 0),
            ColorCode::Red      => write!(f, "\x1B[{}m", offset + 1),
            ColorCode::Green    => write!(f, "\x1B[{}m", offset + 2),
            ColorCode::Yellow   => write!(f, "\x1B[{}m", offset + 3),
            ColorCode::Blue     => write!(f, "\x1B[{}m", offset + 4),
            ColorCode::Magenta  => write!(f, "\x1B[{}m", offset + 5),
            ColorCode::Cyan     => write!(f, "\x1B[{}m", offset + 6),
            ColorCode::White    => write!(f, "\x1B[{}m", offset + 7),
            ColorCode::Standard => write!(f, "\x1B[{}m", offset + 9),
            ColorCode::Bit8(c)  => write!(f, "\x1B[{};5;{}m", offset + 8,  c),
        }
    }
}

impl fmt::Display for DisplayAttribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1B[{}m", match self{
            &DisplayAttribute::Reset      => 0,
            &DisplayAttribute::Bright     => 1,
            &DisplayAttribute::Dim        => 2,
            &DisplayAttribute::Underscore => 4,
            &DisplayAttribute::Blink      => 5,
            &DisplayAttribute::Reverse    => 7,
            &DisplayAttribute::Hidden     => 8,
        })
    }
}

impl fmt::Display for CursorControl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self{
            &CursorControl::Home {row, col}     => write!(f, "\x1B[{};{}H", row, col),
            &CursorControl::Up {count}          => write!(f, "\x1B[{}A", count),
            &CursorControl::Down {count}        => write!(f, "\x1B[{}B", count),
            &CursorControl::Right {count}       => write!(f, "\x1B[{}C", count),
            &CursorControl::Left {count}        => write!(f, "\x1B[{}D", count),
            &CursorControl::Position {row, col} => write!(f, "\x1B[{};{}f", row, col),
            &CursorControl::SavePos             => write!(f, "\x1B[s"),
            &CursorControl::LoadPos             => write!(f, "\x1B[u"),
            &CursorControl::SavePosAndAtt       => write!(f, "\x1B7"),
            &CursorControl::LoadPosAndAtt       => write!(f, "\x1B8"),
            &CursorControl::Hide                => write!(f, "\x1B[?25l"),
            &CursorControl::Show                => write!(f, "\x1B[?25h"),
        }
    }
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            &Font::FontSetG0 => "\x1B(",
            &Font::FontSetG1 => "\x1B)",
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

