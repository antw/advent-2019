#![deny(missing_docs)]

//! Contains the intcode interpreter from Advent of Code 2019, written in Rust. A program is
//! initialized with a vector of intcode instructions which are parsed into instructions, executed
//! by the program.
//!
//! The program may be given inputs before or during execution with [`Program::push_input()`].
//!
//! The program pauses execution whenever an output is produced; [`Program::run()`] will return
//! a [`ProgramState::Output`] containing an i64 allowing you to do what you need with the output,
//! and then resume execution of the program by calling [`Program::run()`] again. When the program
//! finishes executing, [`ProgramState::Halt`] is returned. The `Halt` contains an `Option`: this
//! will be `Some` if the yielded a value and them immediately finished.
//!
//! A typical pattern where you need to act on the outputs of the program during execution is to
//! use a loop:
//!
//! ```norun
//! loop {
//!     match self.program.run() {
//!         ProgramState::Output(value) => {
//!             // do something with output
//!         },
//!         ProgramState::Halt(Some(value)) => {
//!             // optionally do something with output
//!             break;
//!         },
//!         ProgramState::Halt(None) => break,
//!     }
//! }
//! ```
//!
//! In the even that you don't need to do anything with the outputs during execution, you may
//! instead use [`Program::run_capturing_output()`] which will return a `Vec<i64>` containing all
//! of the outputs produced by the program during execution.

use std::collections::VecDeque;

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
    Relative,
}

impl ParamMode {
    fn from_digit(digit: i64) -> ParamMode {
        match digit {
            0 => ParamMode::Position,
            1 => ParamMode::Immediate,
            2 => ParamMode::Relative,
            _ => panic!("Invalid param mode: {}", digit),
        }
    }

    fn value_at(&self, position: usize, program: &Program) -> i64 {
        program.read(self.position(position, &program))
    }

    fn position(&self, position: usize, program: &Program) -> usize {
        match self {
            ParamMode::Position => program.read(position) as usize,
            ParamMode::Immediate => position,
            ParamMode::Relative => (program.relative_base as i64 + program.read(position)) as usize,
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
    SetRelativeBase,
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
            9 => Instruction::SetRelativeBase,
            99 => Instruction::Exit,
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
            Instruction::SetRelativeBase => 2,
            Instruction::Exit => 1,
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
        InstructionWithMode {
            instruction: Instruction::from_opcode(intcode % 100),
            mode_one: ParamMode::from_digit((intcode / 100) % 10),
            mode_two: ParamMode::from_digit((intcode / 1000) % 10),
            mode_three: ParamMode::from_digit(intcode / 10000),
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

/// Returned by [`Program::run()`] to indicate the current state of the program. Running the program
/// returns either a value yielded by the program, with the expectation that the program should be
/// resumed when ready ([`ProgramState::Output`]), or that the program has finished
/// ([`ProgramState::Halt`]) and should not be resumed.
#[derive(Debug, PartialEq)]
pub enum ProgramState {
    /// Indicates that the program has terminated and will not -- or cannot -- continue.
    Halt(Option<i64>),
    /// Indicates that the program has output a value which may be consumed by another program. The
    /// program may be resumed by calling [`Program::run`] again.
    Output(i64),
}

/// The opcode program!
pub struct Program {
    opcodes: Vec<i64>,
    pointer: usize,
    inputs: VecDeque<i64>,
    relative_base: usize,
}

impl Program {
    /// Creates a new [`Program`] using the given opcodes as instructions.
    pub fn new(opcodes: Vec<i64>) -> Program {
        Program {
            opcodes,
            pointer: 0,
            inputs: VecDeque::new(),
            relative_base: 0,
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
        if address > self.opcodes.len() - 1 {
            self.opcodes.resize(address + 1, 0);
        }

        self.opcodes[address] = value;
    }

    /// Reads a the value at `address` directly.
    fn read(&self, address: usize) -> i64 {
        if address > self.opcodes.len() - 1 {
            return 0;
        }

        self.opcodes[address]
    }

    /// Places an i64 into the input queue.
    pub fn push_input(&mut self, input: i64) {
        self.inputs.push_back(input);
    }

    /// Returns the next instruction to be executed, or None if no instructions remain.
    /// TODO: Rename this to front() since it doesn't advance the pointer?
    fn next(&self) -> Option<InstructionWithMode> {
        if self.pointer < self.opcodes.len() {
            return Some(InstructionWithMode::from_intcode(self.read(self.pointer)));
        }

        None
    }

    /// Takes a single parameter from the program memory. This paramter is always a memory position.
    fn take_one_param(&self, instruction: &InstructionWithMode) -> usize {
        instruction.mode_one.position(self.pointer + 1, &self)
    }

    /// Takes two parameters from the program memory. These paramters are always values read from
    /// program memory.
    fn take_two_params(&self, instruction: &InstructionWithMode) -> (i64, i64) {
        let value_one = instruction.mode_one.value_at(self.pointer + 1, &self);

        let value_two = instruction.mode_two.value_at(self.pointer + 2, &self);

        (value_one, value_two)
    }

    /// Takes three parameters from the program memory. The first two are values read from program
    /// memory, and the third is a memory position.
    fn take_three_params(&self, instruction: &InstructionWithMode) -> (i64, i64, usize) {
        let value_one = instruction.mode_one.value_at(self.pointer + 1, &self);

        let value_two = instruction.mode_two.value_at(self.pointer + 2, &self);

        let address = instruction.mode_three.position(self.pointer + 3, &self);

        (value_one, value_two, address)
    }

    /// Runs the program until the next output is yielded, or the program reaches an Exit
    /// instruction.
    pub fn run(&mut self) -> ProgramState {
        while let Some(instruction) = self.next() {
            match instruction.instruction {
                Instruction::Add => {
                    let (left, right, out) = self.take_three_params(&instruction);
                    self.set(out, left + right);
                }
                Instruction::Mul => {
                    let (left, right, out) = self.take_three_params(&instruction);
                    self.set(out, left * right);
                }
                Instruction::Input => {
                    let save_to = self.take_one_param(&instruction);

                    match self.inputs.pop_front() {
                        Some(value) => self.set(save_to, value),
                        None => panic!("No input available"),
                    }
                }
                Instruction::Output => {
                    // Can't use take_one_param as it returns a usize, which will be invalid if the
                    // expected value is negative.
                    let value = instruction.mode_one.value_at(self.pointer + 1, &self);

                    self.jump_forward(instruction.jump_size());

                    // next should always be Some. It may be an Exit instruction.
                    return match self.next().unwrap().instruction {
                        Instruction::Exit => ProgramState::Halt(Some(value)),
                        _ => ProgramState::Output(value),
                    };
                }
                Instruction::JumpIfTrue => {
                    let (condition, value) = self.take_two_params(&instruction);

                    if condition != 0 {
                        self.jump(value as usize);
                    } else {
                        self.jump_forward(instruction.size());
                    }
                }
                Instruction::JumpIfFalse => {
                    let (condition, value) = self.take_two_params(&instruction);

                    if condition == 0 {
                        self.jump(value as usize);
                    } else {
                        self.jump_forward(instruction.size());
                    }
                }
                Instruction::LessThan => {
                    let (first, second, out) = self.take_three_params(&instruction);

                    if first < second {
                        self.set(out, 1);
                    } else {
                        self.set(out, 0);
                    }
                }
                Instruction::Equal => {
                    let (first, second, out) = self.take_three_params(&instruction);

                    if first == second {
                        self.set(out, 1);
                    } else {
                        self.set(out, 0);
                    }
                }
                Instruction::SetRelativeBase => {
                    let value = instruction.mode_one.value_at(self.pointer + 1, &self);
                    self.relative_base = (self.relative_base as i64 + value) as usize;
                }
                Instruction::Exit => break,
            }

            self.jump_forward(instruction.jump_size());
        }

        ProgramState::Halt(None)
    }

    /// Runs the program until it halts, returning a vector containing all outputs yielded.
    pub fn run_capturing_output(&mut self) -> Vec<i64> {
        let mut output = Vec::new();

        loop {
            match self.run() {
                ProgramState::Output(value) => output.push(value),
                ProgramState::Halt(Some(value)) => {
                    output.push(value);
                    break;
                }
                ProgramState::Halt(None) => break,
            }
        }

        output
    }
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
        let program = Program::new(instructions);

        assert_eq!(mode.value_at(0, &program), 2);
        assert_eq!(mode.value_at(1, &program), 3);
    }

    #[test]
    fn test_immediate_mode_get() {
        let mode = ParamMode::Immediate;
        let instructions = vec![1, 2, 3, 4, 5, 6];
        let program = Program::new(instructions);

        assert_eq!(mode.value_at(0, &program), 1);
        assert_eq!(mode.value_at(1, &program), 2);
    }

    #[test]
    fn test_relative_mode_get() {
        let mode = ParamMode::Relative;
        let instructions = vec![1, 2, 3, 4, 5, 6];
        let mut program = Program::new(instructions);

        // Relative base is 0. Read the value at address 0 and add it to the relative base. This
        // gives us index 1, and a value of 2.
        assert_eq!(mode.value_at(0, &program), 2);

        // Relative base is 0. Read the value at address 1 and add it to the relative base. This
        // gives us index 2, and a value of 3.
        assert_eq!(mode.value_at(1, &program), 3);

        program.relative_base = 2;

        // Relative base is 2. Read the value at address 0 and add it to the relative base. This
        // gives us index 3, and a value of 4.
        assert_eq!(mode.value_at(0, &program), 4);

        // Relative base is 2. Read the value at address 1 and add it to the relative base. This
        // gives us index 3, and a value of 4.
        assert_eq!(mode.value_at(1, &program), 5);
    }

    #[test]
    fn test_change_relative_base() {
        // Program sets relative base to 2019.
        let intcodes = vec![109, 19, 99];

        let mut program = Program::new(intcodes);
        program.relative_base = 2000;

        program.run();

        assert_eq!(program.relative_base, 2019);

        // Program sets relative base to 2019 then outputs the value at address 1985 (2019 + -34).
        let intcodes = vec![109, 19, 204, -34, 99];

        let mut program = Program::new(intcodes);

        program.set(1985, 1337);
        program.relative_base = 2000;

        assert_eq!(program.run(), ProgramState::Halt(Some(1337)));
    }

    #[test]
    fn test_program() {
        let mut program = Program::new(vec![1002, 4, 3, 4, 33]);
        program.run();

        assert_eq!(program.opcodes, vec![1002, 4, 3, 4, 99]);
    }
}
