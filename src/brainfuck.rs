// Brainfuck Symbols:
// `>` Move the pointer to the left
// `<` Move the pointer to the right
// `+` Increment the memory at the pointer
// `-` Decrement the memory at the pointer
// `.` Print the ASCII-character at the current cell to std-out
// `,` Take the character at std-in and store it within the curren cell
// `[` Jump past the matching `]` if the cell at the pointer is 0
// `]` Jump back to the matching `[` if the character at the pointer is nonzero

use std::io::{Read, Write};

use nom::{self, Parser, branch, bytes, multi};

const MEM_SIZE: usize = 32_768;

// 32 Kilobyte Memory
type Memory = [u8; MEM_SIZE];

#[derive(Debug)]
pub struct Executor {
    instruction_pointer: usize,
    memory_pointer: usize,
    memory: Memory,
    instructions: Box<[Instruction]>,
}

impl Executor {
    pub fn from_bytes(source: &[u8]) -> Self {
        let (_, mut instructions) = multi::many0(Instruction::parse)
            .parse_complete(source)
            .expect("unexpected symbol");

        Self::decorate_functions(&mut instructions);

        Self {
            instruction_pointer: 0,
            memory_pointer: 0,
            memory: [0; MEM_SIZE],
            instructions: instructions.into_boxed_slice(),
        }
    }

    pub fn run(&mut self) -> Result<(), ExecutorError> {
        let mut output = Vec::with_capacity(128);

        while self.instruction_pointer < self.instructions.len() {
            match self.instructions[self.instruction_pointer] {
                Instruction::Left(number) => {
                    self.memory_pointer = (self.memory_pointer.wrapping_sub(number)) % (MEM_SIZE - 1)
                }
                Instruction::Right(number) => {
                    self.memory_pointer = (self.memory_pointer.wrapping_add(number)) % (MEM_SIZE - 1)
                }
                Instruction::Add(number) => {
                    self.memory[self.memory_pointer] =
                        self.memory[self.memory_pointer].wrapping_add(number)
                }
                Instruction::Sub(number) => {
                    self.memory[self.memory_pointer] =
                        self.memory[self.memory_pointer].wrapping_sub(number)
                }
                Instruction::LeftBracket(src_index) => {
                    if self.memory[self.memory_pointer] == 0 {
                        self.instruction_pointer = src_index + 1
                    }
                }
                Instruction::RightBracket(src_index) => {
                    if self.memory[self.memory_pointer] != 0 {
                        self.instruction_pointer = src_index
                    }
                }
                Instruction::Output => {
                    // println!("OUTPUT: {}", self.memory[self.memory_pointer] as char);
                    // handle.write_all(&self.memory[self.memory_pointer..self.memory_pointer])?;
                    output.push(self.memory[self.memory_pointer]);
                }
                Instruction::Input => {
                    std::io::stdin()
                        .read_exact(&mut self.memory[self.memory_pointer..self.memory_pointer])?;
                }
            };
            self.instruction_pointer += 1;
        }

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&output[0..output.len() - 1])?;
        handle.flush()?;
        Ok(())
    }

    fn decorate_functions(instructions: &mut [Instruction]) {
        let mut bracket_index_stack = vec![];
        for i in 0..instructions.len() - 1 {
            match instructions[i] {
                Instruction::LeftBracket(_) => bracket_index_stack.push(i),
                Instruction::RightBracket(_) => {
                    let previous_bracket = bracket_index_stack.pop().unwrap();
                    // dbg!(&previous_bracket);
                    instructions[previous_bracket] = Instruction::LeftBracket(i);
                    instructions[i] = Instruction::RightBracket(previous_bracket);
                }
                _ => (),
            }
        }
        // let mut i = 0;
        //   let mut bracket_index_stack = vec![];
        //   while i < prog_ops.len() {
        //       match prog_ops[i] {
        //           Ops::LBrack(_) => {
        //               bracket_index_stack.push(i);
        //           }
        //           Ops::RBrack(_) => {
        //               let open_bracket = bracket_index_stack.pop().unwrap();
        //               // Safe to mutate prog_ops because we're not borrowing it as immutable
        //               prog_ops[open_bracket] = Ops::LBrack(i);
        //               prog_ops[i] = Ops::RBrack(open_bracket);
        //           }
        //           _ => (),
        //       }
        //       i += 1;
        //   }
    }
}

#[derive(Debug)]
pub enum ExecutorError {
    IoError(std::io::Error),
    // OverFlowError
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::IoError(e) => write!(f, "IO Error occured: {e}"),
            // ParseError::OverFlowError => write!(f, "Increasing Cell too much, would cause overflow"),
        }
    }
}

impl From<std::io::Error> for ExecutorError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Left(usize),
    Right(usize),
    Add(u8),
    Sub(u8),
    Output,
    Input,
    LeftBracket(usize),
    RightBracket(usize),
    // Block(Vec<Instructions>),
}

impl Instruction {
    pub fn parse(input: &[u8]) -> nom::IResult<&[u8], Self> {
        branch::alt((
            Self::parse_add,
            Self::parse_sub,
            Self::parse_left,
            Self::parse_right,
            Self::parse_l_brk,
            Self::parse_r_brk,
            Self::parse_out,
            Self::parse_in,
        ))
        .parse(input)
    }
}

impl<'source> Instruction {
    fn parse_add(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::map(multi::many1_count(bytes::complete::tag("+")), |v| {
            Self::Add(v.try_into().unwrap_or(255))
        })
        .parse(source)
    }

    fn parse_sub(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::map(multi::many1_count(bytes::complete::tag("-")), |v| {
            Self::Sub(v.try_into().unwrap_or(255))
        })
        .parse(source)
    }

    fn parse_left(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::map(multi::many1_count(bytes::complete::tag("<")), |v| {
            Self::Left(v.try_into().unwrap_or(255))
        })
        .parse(source)
    }

    fn parse_right(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::map(multi::many1_count(bytes::complete::tag(">")), |v| {
            Self::Right(v.try_into().unwrap_or(255))
        })
        .parse(source)
    }

    fn parse_l_brk(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::value(
            Instruction::LeftBracket(usize::max_value()),
            bytes::complete::tag("["),
        )
        .parse(source)
    }

    fn parse_r_brk(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::value(
            Instruction::RightBracket(usize::max_value()),
            bytes::complete::tag("]"),
        )
        .parse(source)
    }

    fn parse_out(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::value(Instruction::Output, bytes::complete::tag(".")).parse(source)
    }

    fn parse_in(source: &'source [u8]) -> nom::IResult<&'source [u8], Self> {
        nom::combinator::value(Instruction::Input, bytes::complete::tag(",")).parse(source)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let input: &[u8] = b"-+><";
        dbg!(multi::many0(Instruction::parse).parse(input));
    }
}

// impl<'source> Instruction {
//     fn parse_add(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = usize, Error = nom::error::Error<&'source [u8]>> {
//         multi::many0_count(bytes::complete::tag("+"))
//     }
//
//     fn parse_sub(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = usize, Error = nom::error::Error<&'source [u8]>> {
//         multi::many0_count(bytes::complete::tag("-"))
//     }
//
//     fn parse_left(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = usize, Error = nom::error::Error<&'source [u8]>> {
//         multi::many0_count(bytes::complete::tag("<"))
//     }
//
//     fn parse_right(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = usize, Error = nom::error::Error<&'source [u8]>> {
//         multi::many0_count(bytes::complete::tag(">"))
//     }
//
//     fn parse_l_brk(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = &'source [u8], Error = nom::error::Error<&'source [u8]>>
//     {
//         bytes::complete::tag("[")
//     }
//
//     fn parse_r_brk(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = &'source [u8], Error = nom::error::Error<&'source [u8]>>
//     {
//         bytes::complete::tag("]")
//     }
//
//     fn parse_out(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = &'source [u8], Error = nom::error::Error<&'source [u8]>>
//     {
//         bytes::complete::tag(".")
//     }
//
//     fn parse_in(
//         source: &'source [u8],
//     ) -> impl Parser<&'source [u8], Output = &'source [u8], Error = nom::error::Error<&'source [u8]>>
//     {
//         bytes::complete::tag(",")
//     }
// }
