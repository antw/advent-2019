use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate rand;

extern crate intcode;
use intcode::{Program, ProgramState};

#[derive(PartialEq, Eq)]
enum Cell {
    Empty,
    Wall,
    OxygenSystem,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Pos(i32, i32);

impl Pos {
    /// Creates a new Pos, by travelling in the given direction.
    fn travel(&self, direction: &Direction) -> Pos {
        match direction {
            Direction::Up => Pos(self.0, self.1 - 1),
            Direction::Down => Pos(self.0, self.1 + 1),
            Direction::Left => Pos(self.0 - 1, self.1),
            Direction::Right => Pos(self.0 + 1, self.1),
        }
    }

    /// Returns a vector containing neighbors of this position into which the robot may travel.
    fn visitable_neighbors(&self, map: &Canvas) -> Vec<Pos> {
        let mut neighbors = Vec::with_capacity(4);

        for i in 1..5 {
            let dir = Direction::from(i);
            let next_pos = self.travel(&dir);

            if map.visitable(&next_pos) {
                neighbors.push(next_pos);
            }
        }

        neighbors
    }
}

#[derive(PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns a direction randomly.
    fn rand() -> Direction {
        match rand::random::<usize>() % 4 {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => unreachable!(),
        }
    }

    /// The input to be given to the program to represent movement in the direction.
    fn as_input(&self) -> i64 {
        match &self {
            Direction::Up => 1,
            Direction::Down => 2,
            Direction::Left => 4,
            Direction::Right => 3,
        }
    }
}

impl From<i32> for Direction {
    fn from(int: i32) -> Direction {
        match int {
            1 => Direction::Up,
            2 => Direction::Down,
            3 => Direction::Right,
            4 => Direction::Left,
            _ => panic!("Invalid direction int: {}", int),
        }
    }
}

struct Canvas(HashMap<Pos, Cell>);

impl Canvas {
    fn new() -> Canvas {
        Canvas(HashMap::new())
    }

    /// Returns if the given position may be visited by the robot.
    ///
    /// TODO: If the desired position is surrounded by None or Wall, the cell is not visitable.
    ///       Could this be used to exit the map building phase earlier?
    fn visitable(&self, position: &Pos) -> bool {
        if let Some(Cell::Wall) = self.0.get(position) {
            return false;
        }

        true
    }

    /// Calculates the shorted path from the start position to the target position. Returns None if
    /// no path could be found.
    fn shortest_path(&self, start: Pos, target: Pos) -> Option<usize> {
        match self.bfs_distance(start, |&pos| pos == target) {
            (distance, true) => Some(distance),
            _ => None,
        }
    }

    /// Calculates the deepest path from the start position to anywhere else in the Canvas.
    fn deepest_path(&self, start: Pos) -> usize {
        self.bfs_distance(start, |_| false).0
    }

    /// Performs a breadth first search starting at the `start` Pos, until the `predicate` closure
    /// returns true.
    ///
    /// This method returns a tuple of two values: the calculated distance, and a boolean indicating
    /// if the predicate method ever returned true. If the bool is false, a path from the start
    /// position to a position where the predicate is truthy could not be found.
    fn bfs_distance<P>(&self, start: Pos, predicate: P) -> (usize, bool)
    where
        P: Fn(&Pos) -> bool,
    {
        // BFS from the oxygen system to the start position.
        let mut visited = HashSet::new();
        let mut distance = 0;
        let mut queue = VecDeque::new();

        queue.push_back(start);

        while queue.len() != 0 {
            let mut new_queue = VecDeque::new();

            while let Some(pos) = queue.pop_front() {
                for neighbor in pos.visitable_neighbors(self) {
                    if predicate(&neighbor) {
                        return (distance + 1, true);
                    }

                    if !visited.contains(&neighbor) {
                        new_queue.push_back(neighbor);
                        visited.insert(neighbor);
                    }
                }
            }

            queue = new_queue;
            distance += 1;
        }

        (distance - 1, false)
    }
}

/// Takes the intcode program and moves the robot randomly a million times to create the map.
/// Returns the completed map and the position of the oxygen system.
fn build_map(program: Program) -> (Canvas, Pos) {
    let mut program = program;
    let mut map = Canvas::new();

    let mut position = Pos(0, 0);
    let mut direction = Direction::rand();
    let mut next_position = position.travel(&direction);

    let mut oxy_pos = None;

    program.push_input(direction.as_input());

    // Stole this trick from someone else. My map input produces cell types for 1657 map positions,
    // so we can stop the loop as soon as all positions are known. This value may differ for other
    // inputs.
    while map.0.len() < 1657 {
        match program.run() {
            ProgramState::Output(value) => match value {
                0 => {
                    map.0.insert(next_position, Cell::Wall);
                }
                1 => {
                    map.0.insert(next_position, Cell::Empty);
                    position = next_position;
                }
                2 => {
                    map.0.insert(next_position, Cell::OxygenSystem);
                    position = next_position;
                    oxy_pos = Some(position);
                }
                _ => unreachable!(),
            },
            ProgramState::Halt => break,
        }

        // Move the robot in a random direction to uncover more of the map.
        direction = Direction::rand();
        next_position = position.travel(&direction);

        while !map.visitable(&next_position) {
            direction = Direction::rand();
            next_position = position.travel(&direction);
        }

        program.push_input(direction.as_input());
    }

    (map, oxy_pos.expect("Expected to find oxygen system!"))
}

/// Calculates the shortest path from the robot starting position (0, 0) to the oxygen system.
fn part_one(program: Program) -> Option<usize> {
    let (map, oxy_pos) = build_map(program);
    map.shortest_path(Pos(0, 0), oxy_pos)
}

/// Calculates how long it takes oxygen to spread out from the oxygen system into all empty cells.
fn part_two(program: Program) -> usize {
    let (map, oxy_pos) = build_map(program);
    map.deepest_path(oxy_pos)
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
    let intcodes = read_intcodes("data/intcodes.txt");

    let shortest_path = part_one(Program::new(intcodes.clone()))
        .expect("Expected to find path from the oxygen system to (0, 0)");

    println!("Part one: {}", shortest_path);

    let deepest_path = part_two(Program::new(intcodes));

    println!("Part two: {}", deepest_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let intcodes = read_intcodes("data/intcodes.txt");
        let shortest_path = part_one(Program::new(intcodes.clone()));

        assert_eq!(shortest_path, Some(248));
    }

    #[test]
    fn test_part_two() {
        let intcodes = read_intcodes("data/intcodes.txt");
        let deepest_path = part_two(Program::new(intcodes.clone()));

        assert_eq!(deepest_path, 382);
    }
}
