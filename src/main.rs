use std::error::Error;
use std::fmt;

const PROGHEIGHT: usize = 25;
const PROGWIDTH: usize = 80;

#[derive(Debug)]
enum InterpreterError {
    PCOutOfBounds,
}

impl Error for InterpreterError {}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PCOutOfBounds => {
                write!(f, "Program counter out of bounds")
            }
        }
    }
}

struct State {
    stack: Vec<u8>,
    program: [[u8; PROGWIDTH]; PROGHEIGHT],
    program_counter: Counter,
    string_mode: bool,
    inertia: Direction,
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Counter {
    x: usize,
    y: usize,
}

impl Counter {
    fn travel(&mut self, direction: Direction) -> Result<(), InterpreterError> {
        match direction {
            Direction::Right => {
                if self.x < PROGWIDTH {
                    self.x += 1;
                    Ok(())
                } else {
                    Err(InterpreterError::PCOutOfBounds)
                }
            }
            Direction::Left => {
                if self.x > 0 {
                    self.x -= 1;
                    Ok(())
                } else {
                    Err(InterpreterError::PCOutOfBounds)
                }
            }
            Direction::Down => {
                if self.y < PROGHEIGHT {
                    self.y += 1;
                    Ok(())
                } else {
                    Err(InterpreterError::PCOutOfBounds)
                }
            }
            Direction::Up => {
                if self.y > 0 {
                    self.y -= 1;
                    Ok(())
                } else {
                    Err(InterpreterError::PCOutOfBounds)
                }
            }
        }
    }
}

enum Instruction {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulo,
    Not,
    GreaterThan,
    PCRight,
    PCLeft,
    PCUp,
    PCDown,
    PCRandom,
    HorizIf,
    VertIf,
    StrModeToggle,
    DupTop,
    SwapTop,
    Discard,
    OutputInt,
    OutputChar,
    Bridge,
    Get,
    Put,
    InputInt,
    InputChar,
    End,
    Num(u8),
    Char(char),
}

impl From<char> for Instruction {
    fn from(item: char) -> Self {
        match item {
            '+' => Self::Addition,
            '-' => Self::Subtraction,
            '*' => Self::Multiplication,
            '/' => Self::Division,
            '%' => Self::Modulo,
            '!' => Self::Not,
            '`' => Self::GreaterThan,
            '>' => Self::PCRight,
            '<' => Self::PCLeft,
            '^' => Self::PCUp,
            'v' => Self::PCDown,
            '?' => Self::PCRandom,
            '_' => Self::HorizIf,
            '|' => Self::VertIf,
            '"' => Self::StrModeToggle,
            ':' => Self::DupTop,
            '\\' => Self::SwapTop,
            '$' => Self::Discard,
            '.' => Self::OutputInt,
            ',' => Self::OutputChar,
            '#' => Self::Bridge,
            'g' => Self::Get,
            'p' => Self::Put,
            '&' => Self::InputInt,
            '~' => Self::InputChar,
            '@' => Self::End,
            // both unwraps valid because is_numeric passes
            num if num.is_numeric() => Self::Num(num.to_digit(10).unwrap().try_into().unwrap()),
            chr => Self::Char(chr),
        }
    }
}

fn main() {
    println!("Hello, world!");
}
