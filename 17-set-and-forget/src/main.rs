use std::collections::HashMap;
use std::fmt;
use std::io;

extern crate intcode;
use intcode::{Program, ProgramState};

#[derive(PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Eq)]
enum TileType {
    Empty,
    Scaffold,
    Robot(Direction),
}

impl From<usize> for TileType {
    fn from(digit: usize) -> TileType {
        match digit {
            35 => TileType::Scaffold,
            46 => TileType::Empty,
            60 => TileType::Robot(Direction::Left),
            62 => TileType::Robot(Direction::Right),
            94 => TileType::Robot(Direction::Up),
            118 => TileType::Robot(Direction::Down),
            _ => panic!("Unknown tile type digit: {}", digit),
        }
    }
}

/// Contains the pixels visited by a robot, and the color painted in each. The internal hash map
/// contains keys of coordinates (x, y), and the color painted (0 for black, 1 for white).
struct Canvas(HashMap<(i64, i64), TileType>);

impl Canvas {
    fn new() -> Canvas {
        Canvas(HashMap::new())
    }

    fn intersections(&self) -> Vec<(i64, i64)> {
        let scaffolds = self
            .0
            .keys()
            .filter(|&pos| *self.0.get(pos).unwrap() == TileType::Scaffold)
            .collect::<Vec<&(i64, i64)>>();
        let mut intersections = Vec::new();

        for &(x, y) in scaffolds {
            let neighbors = vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
            let mut scaffold_neighbors = 0;

            for neighbour in neighbors {
                match self.0.get(&neighbour) {
                    Some(TileType::Scaffold) => scaffold_neighbors += 1,
                    _ => {}
                }
            }

            if scaffold_neighbors >= 3 {
                intersections.push((x, y));
            }
        }

        intersections
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
                    Some(TileType::Scaffold) => output.push('#'),
                    Some(TileType::Empty) => output.push('·'),
                    Some(TileType::Robot(Direction::Up)) => output.push('^'),
                    Some(TileType::Robot(Direction::Right)) => output.push('>'),
                    Some(TileType::Robot(Direction::Down)) => output.push('v'),
                    Some(TileType::Robot(Direction::Left)) => output.push('<'),
                    None => panic!("Empty map position: {:?}", (x, y)),
                }

                output.push(' ');
            }

            output.push('\n');
        }

        write!(f, "{}", output)
    }
}

fn part_one(program: Program) -> i64 {
    let mut program = program;
    let mut map = Canvas::new();
    let mut x = 0;
    let mut y = 0;

    while let ProgramState::Output(value) = program.run() {
        match value {
            10 => {
                x = -1;
                y += 1;
            }
            _ => {
                map.0.insert((x, y), TileType::from(value as usize));
            }
        }

        x += 1;
    }

    let mut sum = 0;

    for (x, y) in map.intersections() {
        sum += x * y;
    }

    sum
}

fn part_two(program: Program) -> i64 {
    let mut program = program;

    // Segments solved by hand.
    let sequence = "A,C,A,B,A,C,B,C,B,C\n";
    let movement = "R,10,R,10,R,6,R,4\nR,4,L,4,L,10,L,10\nR,10,R,10,L,4\n";

    for character in sequence.chars() {
        program.push_input(character as i64);
    }

    for character in movement.chars() {
        program.push_input(character as i64);
    }

    // No video output.
    program.push_input('n' as i64);
    program.push_input('\n' as i64);

    let mut dust = 0;

    while let ProgramState::Output(value) = program.run() {
        dust = value;
    }

    dust
}

fn main() -> Result<(), io::Error> {
    let program = Program::from_file("data/intcodes.txt")?;
    println!("Part one: {}", part_one(program));

    let mut intcodes = intcode::load_intcodes_from_file("data/intcodes.txt")?;
    intcodes[0] = 2;
    let program = Program::new(intcodes);

    println!("Part two: {}", part_two(program));

    Ok(())
}
