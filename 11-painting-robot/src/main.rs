use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate intcode;
use intcode::{Program, ProgramState};

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

        // Set the color of the starting panel.
        self.program.push_input(initial_color as i64);

        // The program sends two outputs each time the robot moves. The first is the color to be
        // painted (0 is black, 1 is white), and the second is the direction it will turn (0 is
        // left, 1 is right).
        loop {
            match self.program.run() {
                ProgramState::Output(value) => {
                    // Robot has moved. Collect the values
                    match prev_output {
                        None => prev_output = Some(value),
                        Some(color) => {
                            // We have two values. The robot is ready to move.
                            canvas.0.insert(position, color as usize);

                            // Set the new direction and position of the robot.
                            direction = direction.turn(value == 0);
                            position = direction.next_position(&position);

                            // Next iteration will be a color.
                            prev_output = None;

                            // Tell the robot the color of the panel it is sitting on.
                            self.program.push_input(
                                (*canvas.0.get(&position).unwrap_or(&0)) as i64
                            );
                        }
                    }
                }
                ProgramState::Halt => break,
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
    fn test_part_one() {
        let intcodes = read_intcodes("data/intcodes.txt");
        let touched = PainterRobot::new(Program::new(intcodes)).paint(0).0.len();

        assert_eq!(touched, 2088);
    }

    #[test]
    fn test_part_two() {
        let intcodes = read_intcodes("data/intcodes.txt");

        let canvas = PainterRobot::new(Program::new(intcodes)).paint(1);
        let printed = format!("{}", canvas);

        assert_eq!(
            printed,
            concat!(
                "  #     #   # # #       # #       # #     # # # #   #           # #     # # #         \n",
                "  #     #   #     #   #     #   #     #   #         #         #     #   #     #       \n",
                "  #     #   #     #   #         #     #   # # #     #         #         #     #       \n",
                "  #     #   # # #     #         # # # #   #         #         #         # # #         \n",
                "  #     #   #   #     #     #   #     #   #         #         #     #   #             \n",
                "    # #     #     #     # #     #     #   #         # # # #     # #     #             \n",
            )
        );
    }
}
