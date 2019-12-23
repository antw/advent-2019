use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate intcode;
use intcode::{Program, ProgramState};

extern crate permutohedron;

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
            ProgramState::Halt => panic!("Unexpected Halt without value in part 1"),
            ProgramState::Wait => panic!("No input available"),
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
            ProgramState::Output(value) => {
                last_output = value;

                // A program which produced an output will be resumed later.
                amplifiers.push_back(amplifier);
            }
            _ => { /* program halted */ },
        }
    }

    last_output
}

fn part_one(intcodes: &Vec<i64>) -> i64 {
    let mut max_output = 0;

    let mut inputs = vec![0, 1, 2, 3, 4];
    let heap = permutohedron::Heap::new(&mut inputs);

    for permutation in heap {
        let last_output = non_feedback_amplifier_power(&intcodes, permutation);

        if last_output > max_output {
            max_output = last_output;
        }
    }

    max_output
}

/// This is similar to part one, except that rather than iterating through each amplifier once, we
/// need to keep iterating until the last amplifier halts constantly feeding the output from an
/// amplifier into the next.
fn part_two(intcodes: &Vec<i64>) -> i64 {
    let mut max_output = 0;

    let mut inputs = vec![5, 6, 7, 8, 9];
    let heap = permutohedron::Heap::new(&mut inputs);

    for permutation in heap {
        let last_output = feedback_amplifier_power(&intcodes, permutation);

        if last_output > max_output {
            max_output = last_output;
        }
    }

    max_output
}

fn main() {
    let intcodes = read_intcodes("data/intcodes.txt");

    println!("Part 1: {}", part_one(&intcodes));
    println!("Part 2: {}", part_two(&intcodes));
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
