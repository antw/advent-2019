use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate intcode;
use intcode::{Program, ProgramState};

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

/// Runs the BOOST program in self-test mode (input = 1).
fn part_one(intcodes: &Vec<i64>) -> Vec<i64> {
    let mut program = Program::new(intcodes.clone());
    program.push_input(1);
    program.run_capturing_output()
}

/// Runs the BOOST program in sensor boost mode (input = 2).
fn part_two(intcodes: &Vec<i64>) -> Vec<i64> {
    let mut program = Program::new(intcodes.clone());
    program.push_input(2);
    program.run_capturing_output()
}

fn main() {
    let intcodes = read_intcodes("data/intcodes.txt");

    println!("Part one: {:?}", part_one(&intcodes));
    println!("Part two: {:?}", part_two(&intcodes));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_9() {
        // Program outputs a copy of itself.
        let mut intcodes = vec![
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];

        let mut program = Program::new(intcodes.clone());

        assert_eq!(program.run_capturing_output(), intcodes);

        // Program outputs a 16-digit number.
        let mut intcodes = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];

        let mut program = Program::new(intcodes);
        let result = program.run();

        assert_eq!(result, ProgramState::Output(1219070632396864));

        // Program outputs the middle number.
        let mut intcodes = vec![104, 1125899906842624, 99];

        let mut program = Program::new(intcodes);
        let result = program.run();

        assert_eq!(result, ProgramState::Output(1125899906842624));
    }
}
