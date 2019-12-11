use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

#[derive(Debug, PartialEq)]
enum ProgramState {
    /// Indicates that the program has terminated and will not -- or cannot -- continue.
    Halt(Option<i64>),
    /// Indicates that the program has output a value which may be consumed by another program. The
    /// program may be resumed by calling `[Program::run]` again.
    Output(i64),
}

struct Program {
    opcodes: Vec<i64>,
    pointer: usize,
    inputs: VecDeque<i64>,
    relative_base: usize,
}

impl Program {
    fn new(opcodes: Vec<i64>) -> Program {
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
    fn push_input(&mut self, input: i64) {
        self.inputs.push_back(input);
    }

    /// Returns the next instruction to be executed, or None if no instructions remain.
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

    fn run(&mut self) -> ProgramState {
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
    fn run_capturing_output(&mut self) -> Vec<i64> {
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

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn turn(&self, left: bool) -> Direction {
        match self {
            Direction::Up => match left {
                true => Direction::Left,
                false => Direction::Right,
            },
            Direction::Down => match left {
                true => Direction::Right,
                false => Direction::Left,
            },
            Direction::Left => match left {
                true => Direction::Down,
                false => Direction::Up,
            },
            Direction::Right => match left {
                true => Direction::Up,
                false => Direction::Down,
            },
        }
    }

    fn next_position(&self, position: &(i64, i64)) -> (i64, i64) {
        match &self {
            Direction::Up => (position.0, position.1 - 1),
            Direction::Down => (position.0, position.1 + 1),
            Direction::Left => (position.0 - 1, position.1),
            Direction::Right => (position.0 + 1, position.1),
        }
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

/// Calculates the maximum signal which may be sent to the thrusters depending on the setting of
/// each amplifier. Part one of day seven.
fn non_feedback_amplifier_power(intcodes: &Vec<i64>, settings: Vec<i64>) -> i64 {
    let mut last_output = 0;

    for input in settings.iter() {
        let mut amplifier = Program::new(intcodes.clone());

        amplifier.push_input(*input);
        amplifier.push_input(last_output);

        match amplifier.run() {
            ProgramState::Halt(Some(value)) => last_output = value,
            ProgramState::Halt(None) => panic!("Unexpected Halt without value in part 1"),
            ProgramState::Output(value) => last_output = value,
        }
    }

    last_output
}

/// Calculates the maximum signal which may be sent to the thrusters depending on the setting of
/// each amplifier. In this case, the last amplifier is routed back to the first in a feedback loop.
/// Each amplifier is contintually stopped when it produces output, and resumed when new input is
/// available until all have halted.
fn feedback_amplifier_power(intcodes: &Vec<i64>, settings: Vec<i64>) -> i64 {
    let mut amplifiers = VecDeque::new();
    let mut last_output = 0;

    for input in settings.iter() {
        let mut amplifier = Program::new(intcodes.clone());

        // Provide initial phase setting.
        amplifier.push_input(*input);
        amplifiers.push_back(amplifier)
    }

    while let Some(mut amplifier) = amplifiers.pop_front() {
        amplifier.push_input(last_output);

        match amplifier.run() {
            ProgramState::Halt(value) => {
                last_output = value.unwrap_or(last_output);
            }
            ProgramState::Output(value) => {
                last_output = value;

                // A program which produced an output will be resumed later.
                amplifiers.push_back(amplifier);
            }
        }
    }

    last_output
}

/// Contains the pixels visited by a robot, and the color painted in each. The internal hash map
/// contains keys of coordinates (x, y), and the color painted (0 for black, 1 for white).
struct Canvas(HashMap<(i64, i64), usize>);

impl Canvas {
    fn new() -> Canvas {
        Canvas(HashMap::new())
    }
}

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let min_x = self.0.keys().min_by_key(|(x, _)| x).unwrap().0;
        let max_x = self.0.keys().max_by_key(|(x, _)| x).unwrap().0;
        let min_y = self.0.keys().min_by_key(|(_, y)| y).unwrap().1;
        let max_y = self.0.keys().max_by_key(|(_, y)| y).unwrap().1;

        let width = (max_x + 1) - min_x;
        let height = (max_y + 1) - min_y;

        // Two characters per pixel, plus a newline per row.
        let mut output = String::with_capacity(((2 * width) * height + height) as usize);

        for y in min_y..(max_y + 1) {
            for x in min_x..(max_x + 1) {
                match self.0.get(&(x, y)) {
                    Some(color) if *color != 0 => {
                        output.push('#');
                    }
                    _ => output.push(' '),
                }

                output.push(' ');
            }

            output.push('\n');
        }

        write!(f, "{}", output)
    }
}

struct PainterRobot {
    program: Program,
}

impl PainterRobot {
    fn new(program: Program) -> PainterRobot {
        PainterRobot { program }
    }

    /// Runs the painter robot.
    fn paint(mut self, initial_color: usize) -> Canvas {
        let mut position = (0, 0);
        let mut direction = Direction::Up;
        let mut prev_output = None;

        // Map contains a list of coordinates visited by the robot, and the color painted.
        let mut canvas = Canvas::new();

        canvas.0.insert((0, 0), initial_color);

        // The robot starts over a black panel.
        self.program.push_input(initial_color as i64);

        // The program sends two outputs each time the robot moves. The first is the color to be painted
        // (0 is black, 1 is white), and the second is the direction it will turn (0 is left, 1 is
        // right).
        loop {
            match self.program.run() {
                ProgramState::Output(value) => {
                    // Robot has moved. Collect the values
                    match prev_output {
                        None => prev_output = Some(value),
                        Some(color) => {
                            // We have two values. The robot is ready to move.
                            canvas.0.insert(position, color as usize);

                            let old_dir = direction;

                            // Set the new direction of the robot.
                            direction = old_dir.turn(value == 0);

                            // Set the new position of the robot.
                            position = direction.next_position(&position);

                            // set new position
                            prev_output = None;

                            // Tell the robot the color of the panel it is sitting on.
                            self.program.push_input(value);
                        }
                    }
                }
                ProgramState::Halt(Some(value)) => {
                    println!("halting with {}", value);
                    break;
                }
                ProgramState::Halt(None) => break,
            }
        }

        canvas
    }
}

fn main() {
    let intcodes = read_intcodes("data/intcodes.txt");

    println!(
        "Part one: {:?}",
        PainterRobot::new(Program::new(intcodes.clone()))
            .paint(0)
            .0
            .len()
    );

    println!("Part two:");

    println!("{}", PainterRobot::new(Program::new(intcodes)).paint(1));
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
    fn test_program() {
        let mut program = Program::new(vec![1002, 4, 3, 4, 33]);
        program.run();

        assert_eq!(program.opcodes, vec![1002, 4, 3, 4, 99]);
    }

    #[test]
    fn test_day_5_part_2_equal_position_mode() {
        // Program tests that the input is equal to 8. If so, it returns 1.
        let intcodes = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];

        let mut program = Program::new(intcodes.clone());
        program.push_input(8);

        assert_eq!(program.run(), ProgramState::Halt(Some(1)));

        let mut program = Program::new(intcodes.clone());
        program.push_input(7);

        assert_eq!(program.run(), ProgramState::Halt(Some(0)));
    }

    #[test]
    fn test_day_5_part_2_less_than_position_mode() {
        // Program tests that the input is less than 8. If so, it returns 1.
        let intcodes = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];

        let mut program = Program::new(intcodes.clone());
        program.push_input(7);

        assert_eq!(program.run(), ProgramState::Halt(Some(1)));

        let mut program = Program::new(intcodes.clone());
        program.push_input(8);

        assert_eq!(program.run(), ProgramState::Halt(Some(0)));
    }

    #[test]
    fn test_day_5_part_2_greater_than_immediate_mode() {
        // Program tests that the input is less than 8. If so, it returns 1.
        let intcodes = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];

        let mut program = Program::new(intcodes.clone());
        program.push_input(8);

        assert_eq!(program.run(), ProgramState::Halt(Some(1)));

        let mut program = Program::new(intcodes.clone());
        program.push_input(7);

        assert_eq!(program.run(), ProgramState::Halt(Some(0)));
    }

    #[test]
    fn test_day_5_part_2_less_than_immediate_mode() {
        // Program tests that the input is less than 8. If so, it returns 1.
        let intcodes = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];

        let mut program = Program::new(intcodes.clone());
        program.push_input(7);

        assert_eq!(program.run(), ProgramState::Halt(Some(1)));

        let mut program = Program::new(intcodes.clone());
        program.push_input(8);

        assert_eq!(program.run(), ProgramState::Halt(Some(0)));
    }

    #[test]
    fn test_day_7_part_1() {
        let intcodes = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];

        assert_eq!(
            non_feedback_amplifier_power(&intcodes, vec![4, 3, 2, 1, 0]),
            43210
        );

        let intcodes = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];

        assert_eq!(
            non_feedback_amplifier_power(&intcodes, vec![0, 1, 2, 3, 4]),
            54321
        );

        let intcodes = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];

        assert_eq!(
            non_feedback_amplifier_power(&intcodes, vec![1, 0, 4, 3, 2]),
            65210
        );
    }

    #[test]
    fn test_day_7_part_2() {
        let intcodes = vec![
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ];

        assert_eq!(
            feedback_amplifier_power(&intcodes, vec![9, 8, 7, 6, 5]),
            139629729
        );

        let intcodes = vec![
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ];

        assert_eq!(
            feedback_amplifier_power(&intcodes, vec![9, 7, 8, 5, 6]),
            18216
        );
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
    fn test_day_9() {
        // Program outputs a copy of itself.
        let intcodes = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];

        let mut program = Program::new(intcodes.clone());

        assert_eq!(program.run_capturing_output(), intcodes);

        // Program outputs a 16-digit number.
        let intcodes = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];

        let mut program = Program::new(intcodes);
        let result = program.run();

        assert_eq!(result, ProgramState::Halt(Some(1219070632396864)));

        // Program outputs the middle number.
        let intcodes = vec![104, 1125899906842624, 99];

        let mut program = Program::new(intcodes);
        let result = program.run();

        assert_eq!(result, ProgramState::Halt(Some(1125899906842624)));
    }
}
