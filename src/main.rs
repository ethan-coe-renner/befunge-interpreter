use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{error::Error, fmt, io::Read};

const PROGHEIGHT: usize = 25;
const PROGWIDTH: usize = 80;

#[derive(Debug)]
enum InterpreterError {
    PCOutOfBounds,
    EmptyStack,
    NoInput,
    InvalidInstruction(char),
}

impl Error for InterpreterError {}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PCOutOfBounds => {
                write!(f, "Program pointer out of bounds")
            }
            Self::EmptyStack => {
                write!(f, "Read from empty stack")
            }
            Self::NoInput => {
                write!(f, "No valid input given")
            }
            Self::InvalidInstruction(chr) => {
                write!(f, "Invalid instruction: '{chr}'")
            }
        }
    }
}

type Program = [[u8; PROGWIDTH]; PROGHEIGHT];

trait Pointable<T> {
    fn get(&self, pointer: &Pointer) -> T;
    fn put(&mut self, pointer: &Pointer, value: T);
}

impl Pointable<u8> for Program {
    fn get(&self, pointer: &Pointer) -> u8 {
        self[pointer.x][pointer.y]
    }
    fn put(&mut self, pointer: &Pointer, value: u8) {
        self[pointer.x][pointer.y] = value
    }
}

struct State {
    stack: Vec<u8>,
    program: Program,
    program_pointer: Pointer,
    string_mode: bool,
    inertia: Direction,
}

impl State {
    fn stack_arithmetic(&mut self, op: fn(u8, u8) -> u8) -> Result<(), InterpreterError> {
        let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
        let b = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
        self.stack.push(op(a, b));
        Ok(())
    }
    fn move_pointer(&mut self) -> Result<(), InterpreterError> {
        self.program_pointer.travel(&self.inertia)
    }
    fn update_state(&mut self) -> Result<(), InterpreterError> {
        if self.string_mode {
            self.stack.push(self.program.get(&self.program_pointer))
        } else {
            let instruction = Instruction::from(self.program.get(&self.program_pointer));
            match instruction {
                Instruction::Addition => self.stack_arithmetic(u8::wrapping_add)?,
                Instruction::Subtraction => self.stack_arithmetic(u8::wrapping_sub)?,
                Instruction::Multiplication => self.stack_arithmetic(u8::wrapping_mul)?,
                Instruction::Division => self.stack_arithmetic(u8::wrapping_div)?,
                Instruction::Modulo => self.stack_arithmetic(u8::wrapping_rem)?,
                Instruction::Not => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.stack.push(if a == 0 { 1 } else { 0 })
                }
                Instruction::GreaterThan => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    let b = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.stack.push(if b > a { 1 } else { 0 })
                }
                Instruction::PCRight => self.inertia = Direction::Right,
                Instruction::PCLeft => self.inertia = Direction::Left,
                Instruction::PCDown => self.inertia = Direction::Down,
                Instruction::PCUp => self.inertia = Direction::Up,
                Instruction::PCRandom => self.inertia = rand::random(),
                Instruction::HorizIf => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.inertia = if a == 0 {
                        Direction::Right
                    } else {
                        Direction::Left
                    }
                }
                Instruction::VertIf => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.inertia = if a == 0 {
                        Direction::Down
                    } else {
                        Direction::Up
                    }
                }
                Instruction::StrModeToggle => self.string_mode = !self.string_mode,
                Instruction::DupTop => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.stack.push(a);
                    self.stack.push(a);
                }
                Instruction::SwapTop => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    let b = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.stack.push(a);
                    self.stack.push(b);
                }
                Instruction::Discard => {
                    self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                }
                Instruction::OutputInt => {
                    let a = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    print!("{a}")
                }
                Instruction::OutputChar => {
                    let a: char = self.stack.pop().ok_or(InterpreterError::EmptyStack)? as char;
                    print!("{a}")
                }
                Instruction::Bridge => {
                    self.move_pointer();
                }
                Instruction::Get => {
                    let x = self.stack.pop().ok_or(InterpreterError::EmptyStack)? as usize;
                    let y = self.stack.pop().ok_or(InterpreterError::EmptyStack)? as usize;
                    self.stack.push(if x > PROGWIDTH || y > PROGHEIGHT {
                        0
                    } else {
                        self.program.get(&Pointer { x, y })
                    })
                }
                Instruction::Put => {
                    let x = self.stack.pop().ok_or(InterpreterError::EmptyStack)? as usize;
                    let y = self.stack.pop().ok_or(InterpreterError::EmptyStack)? as usize;
                    let v = self.stack.pop().ok_or(InterpreterError::EmptyStack)?;
                    self.program.put(&Pointer { x, y }, v);
                }
                Instruction::InputInt => {
                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_err() {
                        return Err(InterpreterError::NoInput);
                    }
                    let num: u8;
                    match input.trim().parse() {
                        Ok(value) => num = value,
                        Err(_) => return Err(InterpreterError::NoInput),
                    }
                    self.stack.push(num);
                }
                Instruction::InputChar => {
                    let input: Option<u8> = std::io::stdin()
                        .bytes()
                        .next()
                        .and_then(|result| result.ok());
                    match input {
                        Some(value) => self.stack.push(value),
                        None => return Err(InterpreterError::NoInput),
                    }
                }
                Instruction::End => {
                    // TODO: figure out end trigger
                }
                Instruction::Num(num) => self.stack.push(num),
                Instruction::Char(chr) => {
                    return Err(InterpreterError::InvalidInstruction(chr));
                }
            }
        }
        self.program_pointer.travel(&self.inertia)
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0..=3) {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Right,
            _ => Direction::Left,
        }
    }
}

struct Pointer {
    x: usize,
    y: usize,
}

impl Pointer {
    fn travel(&mut self, direction: &Direction) -> Result<(), InterpreterError> {
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

impl From<u8> for Instruction {
    fn from(item: u8) -> Self {
        match item as char {
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
