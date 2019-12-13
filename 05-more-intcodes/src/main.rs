use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

/// Parameters may be retrieved from the program in one of two ways.
///
/// In `Position` mode, the instruction will read the value at the program address. If the program
/// pointer is at 1, and the value contained in this address is 50, the program will look up and
/// return the value currently stored at memory address 50.
///
/// In `Immediate` mode, the parameter is interpreted as a value. If the program pointer is at 1,
/// and the value contained in this address is 50, the literal value 50 will be returned.
#[derive(Debug, PartialEq)]
enum ParamMode {
    Position,
    Immediate,
}

impl ParamMode {
    fn from_digit(digit: i64) -> ParamMode {
        if digit == 0 {
            ParamMode::Position
        } else {
            ParamMode::Immediate
        }
    }

    fn value_at(&self, position: usize, instructions: &Vec<i64>) -> i64 {
        match self {
            ParamMode::Position => instructions[self.position(position, &instructions)] as i64,
            ParamMode::Immediate => instructions[position] as i64,
        }
    }

    fn position(&self, position: usize, instructions: &Vec<i64>) -> usize {
        match self {
            ParamMode::Position => instructions[position] as usize,
            ParamMode::Immediate => position as usize,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Instruction {
    Add,
    Mul,
    Input,
    Output,
    JumpIfTrue,
    JumpIfFalse,
    LessThan,
    Equal,
    Exit,
}

impl Instruction {
    fn from_opcode(digit: i64) -> Instruction {
        match digit {
            1 => Instruction::Add,
            2 => Instruction::Mul,
            3 => Instruction::Input,
            4 => Instruction::Output,
            5 => Instruction::JumpIfTrue,
            6 => Instruction::JumpIfFalse,
            7 => Instruction::LessThan,
            8 => Instruction::Equal,
            9 => Instruction::Exit,
            _ => panic!("Unknown opcode: {}", digit),
        }
    }

    /// The size of the instruction including its operands.
    ///
    /// For example, an Add instruction contains the opcode for the instruction, an address for the
    /// first operand, an address for the second operand, and an address for the result.
    fn size(&self) -> usize {
        match self {
            Instruction::Add => 4,
            Instruction::Mul => 4,
            Instruction::Input => 2,
            Instruction::Output => 2,
            Instruction::JumpIfTrue => 3,
            Instruction::JumpIfFalse => 3,
            Instruction::LessThan => 4,
            Instruction::Equal => 4,
            Instruction::Exit => 0,
        }
    }

    /// Returns if the instruction will adjust the program pointer.
    fn jumps(&self) -> bool {
        match self {
            Instruction::JumpIfTrue | Instruction::JumpIfFalse => true,
            _ => false,
        }
    }
}

/// Contains an instruction to be executed, and the [`ParamMode`] of each parameter.
#[derive(Debug)]
struct InstructionWithMode {
    instruction: Instruction,
    mode_one: ParamMode,
    mode_two: ParamMode,
    mode_three: ParamMode,
}

impl InstructionWithMode {
    /// Converts an i64 to a InstructionWithMode describing the instruction and up to three parameter
    /// modes.
    fn from_intcode(intcode: i64) -> InstructionWithMode {
        let mut digits = Vec::new();
        let mut intcode = intcode;

        while intcode > 9 {
            digits.push(intcode % 10);
            intcode = intcode / 10;
        }

        digits.push(intcode);
        digits.resize_with(5, Default::default);

        InstructionWithMode {
            instruction: Instruction::from_opcode(digits[0]),
            mode_one: ParamMode::from_digit(digits[2]),
            mode_two: ParamMode::from_digit(digits[3]),
            mode_three: ParamMode::from_digit(digits[4]),
        }
    }

    /// See [Instruction::size]
    fn size(&self) -> usize {
        self.instruction.size()
    }

    /// Tells the program by how much to increase the program pointer after executing this
    /// instruction.
    fn jump_size(&self) -> usize {
        if self.instruction.jumps() {
            0
        } else {
            self.size()
        }
    }
}

struct Program {
    opcodes: Vec<i64>,
    pointer: usize,
}

impl Program {
    fn new(opcodes: Vec<i64>) -> Program {
        Program {
            opcodes,
            pointer: 0,
        }
    }

    /// Jumps to the specified memory `address`.
    fn jump(&mut self, address: usize) {
        self.pointer = address;
    }

    /// Jumps the program poitner forward by the specified number of places.
    fn jump_forward(&mut self, by: usize) {
        self.pointer += by;
    }

    /// Sets a `value` at the given program `address`.
    fn set(&mut self, address: usize, value: i64) {
        self.opcodes[address] = value;
    }

    /// Reads a the value at `address` directly.
    fn read(&self, address: usize) -> i64 {
        self.opcodes[address]
    }

    /// Returns the next instruction to be executed, or None if no instructions remain.
    fn next(&mut self) -> Option<InstructionWithMode> {
        if self.pointer < self.opcodes.len() {
            return Some(InstructionWithMode::from_intcode(self.read(self.pointer)));
        }

        None
    }

    /// Takes a single parameter from the program memory. This paramter is always a memory position.
    fn take_one_param(&self, instruction: &InstructionWithMode) -> usize {
        instruction
            .mode_one
            .position(self.pointer + 1, &self.opcodes)
    }

    /// Takes two parameters from the program memory. These paramters are always values read from
    /// program memory.
    fn take_two_params(&self, instruction: &InstructionWithMode) -> (i64, i64) {
        let value_one = instruction
            .mode_one
            .value_at(self.pointer + 1, &self.opcodes);

        let value_two = instruction
            .mode_two
            .value_at(self.pointer + 2, &self.opcodes);

        (value_one, value_two)
    }

    /// Takes three parameters from the program memory. The first two are values read from program
    /// memory, and the third is a memory position.
    fn take_three_params(&self, instruction: &InstructionWithMode) -> (i64, i64, usize) {
        let value_one = instruction
            .mode_one
            .value_at(self.pointer + 1, &self.opcodes);

        let value_two = instruction
            .mode_two
            .value_at(self.pointer + 2, &self.opcodes);

        let address = instruction
            .mode_three
            .position(self.pointer + 3, &self.opcodes);

        (value_one, value_two, address)
    }
}

/// Provided with a path to a file containing an intcode program, reads the file and returns a
/// vector of the intcodes.
fn read_intcodes(path: &str) -> Vec<i64> {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut first_line = String::new();
    reader.read_line(&mut first_line).unwrap();

    first_line
        .trim()
        .split(",")
        .map(|intcode| intcode.parse::<i64>().unwrap())
        .collect()
}

/// Takes a vector of intcodes and runs the program, returning the final program intcodes.
fn run_program(intcodes: Vec<i64>) -> Vec<i64> {
    let mut program = Program::new(intcodes);

    while let Some(instruction) = program.next() {
        match instruction.instruction {
            Instruction::Add => {
                let (left, right, out) = program.take_three_params(&instruction);
                program.set(out, left + right);
            }
            Instruction::Mul => {
                let (left, right, out) = program.take_three_params(&instruction);
                program.set(out, left * right);
            }
            Instruction::Input => {
                let save_to = program.take_one_param(&instruction);
                let mut input = String::new();

                print!("Enter number to use as input: ");
                io::stdout().flush().expect("Expected to flush output.");

                io::stdin()
                    .read_line(&mut input)
                    .expect("error: unable to read input");

                program.set(
                    save_to,
                    input.trim().parse::<i64>().expect("not a valid number"),
                );
            }
            Instruction::Output => {
                println!("{}", program.read(program.take_one_param(&instruction)));
            }
            Instruction::JumpIfTrue => {
                let (condition, value) = program.take_two_params(&instruction);

                if condition != 0 {
                    program.jump(value as usize);
                } else {
                    program.jump_forward(instruction.size());
                }
            }
            Instruction::JumpIfFalse => {
                let (condition, value) = program.take_two_params(&instruction);

                if condition == 0 {
                    program.jump(value as usize);
                } else {
                    program.jump_forward(instruction.size());
                }
            }
            Instruction::LessThan => {
                let (first, second, out) = program.take_three_params(&instruction);

                if first < second {
                    program.set(out, 1);
                } else {
                    program.set(out, 0);
                }
            }
            Instruction::Equal => {
                let (first, second, out) = program.take_three_params(&instruction);

                if first == second {
                    program.set(out, 1);
                } else {
                    program.set(out, 0);
                }
            }
            Instruction::Exit => break,
        }
        program.jump_forward(instruction.jump_size());
    }

    program.opcodes
}

fn main() {
    run_program(read_intcodes("intcodes.txt"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_instruction() {
        let instruction = InstructionWithMode::from_intcode(1);

        assert_eq!(instruction.instruction, Instruction::Add);
        assert_eq!(instruction.mode_one, ParamMode::Position);
        assert_eq!(instruction.mode_two, ParamMode::Position);
        assert_eq!(instruction.mode_three, ParamMode::Position);

        let instruction = InstructionWithMode::from_intcode(1002);

        assert_eq!(instruction.instruction, Instruction::Mul);
        assert_eq!(instruction.mode_one, ParamMode::Position);
        assert_eq!(instruction.mode_two, ParamMode::Immediate);
        assert_eq!(instruction.mode_three, ParamMode::Position);

        let instruction = InstructionWithMode::from_intcode(2);

        assert_eq!(instruction.instruction, Instruction::Mul);
        assert_eq!(instruction.mode_one, ParamMode::Position);
        assert_eq!(instruction.mode_two, ParamMode::Position);
        assert_eq!(instruction.mode_three, ParamMode::Position);

        let instruction = InstructionWithMode::from_intcode(10002);

        assert_eq!(instruction.instruction, Instruction::Mul);
        assert_eq!(instruction.mode_one, ParamMode::Position);
        assert_eq!(instruction.mode_two, ParamMode::Position);
        assert_eq!(instruction.mode_three, ParamMode::Immediate);

        let instruction = InstructionWithMode::from_intcode(11102);

        assert_eq!(instruction.instruction, Instruction::Mul);
        assert_eq!(instruction.mode_one, ParamMode::Immediate);
        assert_eq!(instruction.mode_two, ParamMode::Immediate);
        assert_eq!(instruction.mode_three, ParamMode::Immediate);

        let instruction = InstructionWithMode::from_intcode(99);

        assert_eq!(instruction.instruction, Instruction::Exit);
    }

    #[test]
    fn test_position_mode_get() {
        let mode = ParamMode::Position;
        let instructions = vec![1, 2, 3, 4, 5, 6];

        assert_eq!(mode.value_at(0, &instructions), 2);
        assert_eq!(mode.value_at(1, &instructions), 3);
    }

    #[test]
    fn test_immediate_mode_get() {
        let mode = ParamMode::Immediate;
        let instructions = vec![1, 2, 3, 4, 5, 6];

        assert_eq!(mode.value_at(0, &instructions), 1);
        assert_eq!(mode.value_at(1, &instructions), 2);
    }

    #[test]
    fn test_program() {
        let result = run_program(vec![1002, 4, 3, 4, 33]);
        assert_eq!(result, vec![1002, 4, 3, 4, 99]);
    }
}
