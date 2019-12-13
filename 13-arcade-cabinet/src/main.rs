use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate intcode;
use intcode::{Program, ProgramState};

#[derive(PartialEq, Eq)]
enum TileType {
    Blank,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl From<i64> for TileType {
    fn from(digit: i64) -> Self {
        match digit {
            0 => TileType::Blank,
            1 => TileType::Wall,
            2 => TileType::Block,
            3 => TileType::Paddle,
            4 => TileType::Ball,
            _ => panic!("Invalid tile type: {}", digit),
        }
    }
}

struct Arcade {
    program: Program,
    // This could be swapped out for Canvas from day 11 to support rendering to the console. I think
    // this would need an implementation of Format for TileType, or swapping back to just using
    // integers in the canvas.
    canvas: HashMap<(i64, i64), TileType>,
}

impl Arcade {
    fn new(program: Program) -> Arcade {
        Arcade {
            program,
            canvas: HashMap::new(),
        }
    }

    fn run(&mut self) -> i64 {
        let mut x_pos = None;
        let mut y_pos = None;
        let mut score = 0;

        // Paddle only moves left or right.
        let mut paddle_pos = 0;
        let mut ball = None;

        // The program yields three values before an action should be taken: an x position, a y
        // position, and a tile type.
        loop {
            match self.program.run() {
                ProgramState::Output(value) => {
                    match (x_pos, y_pos) {
                        (None, None) => x_pos = Some(value),
                        (Some(_), None) => y_pos = Some(value),
                        (Some(x), Some(y)) => {
                            match (x, y) {
                                (-1, 0) => score = value,
                                _ => {
                                    // We have x, y, and tile type values.
                                    let tile = TileType::from(value);

                                    if tile == TileType::Paddle {
                                        paddle_pos = x;
                                    } else if tile == TileType::Ball {
                                        ball = Some(x);
                                    }

                                    if let Some(ball_pos) = ball {
                                        // Provide joystick input to move the paddle underneath the
                                        // ball.
                                        if ball_pos < paddle_pos {
                                            self.program.push_input(-1);
                                        } else if ball_pos > paddle_pos {
                                            self.program.push_input(1);
                                        } else {
                                            self.program.push_input(0);
                                        }

                                        ball = None;
                                    }

                                    self.canvas.insert((x, y), tile);
                                }
                            }

                            x_pos = None;
                            y_pos = None;
                        }
                        _ => unreachable!(),
                    }
                }
                ProgramState::Halt => break,
            }
        }

        score
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

fn main() {
    let mut intcodes = read_intcodes("data/intcodes.txt");
    let mut arcade = Arcade::new(Program::new(intcodes.clone()));
    arcade.run();

    println!(
        "Part one: {}",
        arcade
            .canvas
            .values()
            .filter(|tile_type| **tile_type == TileType::Block)
            .collect::<Vec<&TileType>>()
            .len()
    );

    // Set first memory address to 2 to play for free.
    intcodes[0] = 2;

    let mut arcade = Arcade::new(Program::new(intcodes));
    println!("Part two: {}", arcade.run());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let intcodes = read_intcodes("data/intcodes.txt");
        let mut arcade = Arcade::new(Program::new(intcodes));
        arcade.run();

        assert_eq!(
            arcade
                .canvas
                .values()
                .filter(|tile_type| **tile_type == TileType::Block)
                .collect::<Vec<&TileType>>()
                .len(),
            369
        );
    }

    #[test]
    fn test_part_two() {
        let mut intcodes = read_intcodes("data/intcodes.txt");

        // Set first memory address to 2 to play for free.
        intcodes[0] = 2;

        let mut arcade = Arcade::new(Program::new(intcodes));

        assert_eq!(arcade.run(), 19210);
    }
}
