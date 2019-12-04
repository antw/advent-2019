use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Deref;

struct Token {
    direction: char,
    length: usize,
}

impl Token {
    fn vector(&self) -> (isize, isize) {
        match self.direction {
            'R' => (1, 0),
            'L' => (-1, 0),
            'U' => (0, 1),
            'D' => (0, -1),
            _ => panic!("Unknown direction: {}", self.direction),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Position(isize, isize);

struct Wire(HashMap<Position, isize>);

impl<'a> Wire {
    /// Creates a Wire describing all the occupied positions in the panel by the wire.
    ///
    /// Wire wraps around a HashMap where each key is a position in which the wire is found, and
    /// each value is the number of steps requires to backtrack from the position to the central
    /// port at (0, 0).
    fn from_tokens(wire: &Vec<Token>) -> Self {
        let mut wire_info = HashMap::new();
        let mut x = 0;
        let mut y = 0;
        let mut count = 0;

        for token in wire {
            for _ in 0..token.length {
                let vector = token.vector();

                count += 1;
                x += vector.0;
                y += vector.1;

                wire_info.insert(Position(x, y), count);
            }
        }

        Wire(wire_info)
    }

    /// Given another `[Wire]`, returns a vector of all positions where the wires intersect.
    fn intersection(&'a self, other: &'a Wire) -> Vec<&'a Position> {
        let mut intersections = Vec::new();

        for key in other.keys() {
            if self.contains_key(key) {
                intersections.push(key)
            }
        }

        intersections
    }
}

impl Deref for Wire {
    type Target = HashMap<Position, isize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Reads the file at `path`, parsing each non-blank line into a vector of `[Token]` describing the
/// path of the line.
fn read_wires(path: &str) -> Vec<Vec<Token>> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.unwrap())
        .filter(|line| line.len() > 0)
        .map(|line| string_to_tokens(&line))
        .collect()
}

/// Parses a comma-separated list of tokens into a vector of `[Token]`.
fn string_to_tokens(string: &str) -> Vec<Token> {
    string
        .trim()
        .split(",")
        .map(|token| {
            let mut chars = token.chars();

            Token {
                direction: chars.next().unwrap(),
                length: chars.collect::<String>().parse::<usize>().unwrap(),
            }
        })
        .collect()
}

/// Takes a vector of known intersections between two wires and returns the minimum Manhattan
/// distance from an intersection to the central port at (0, 0).
fn min_distance(positions: &Vec<&Position>) -> Option<isize> {
    if positions.len() == 0 {
        return None;
    }

    let mut min = isize::max_value();

    for position in positions {
        let distance = position.0.abs() + position.1.abs();

        if distance < min {
            min = distance;
        }
    }

    Some(min)
}

/// Takes a vector of known intersections between two wires with the two hashmaps representing the
/// wires and returns the minimum number of steps to traverse from an intersection between the two
/// wires back to the central port at (0, 0).
fn min_steps(positions: &Vec<&Position>, wire_one: &Wire, wire_two: &Wire) -> Option<isize> {
    if positions.len() == 0 {
        return None;
    }

    let mut min = isize::max_value();

    for position in positions {
        let steps = wire_one[position] + wire_two[position];

        if steps < min {
            min = steps;
        }
    }

    Some(min)
}

fn main() {
    let wires = read_wires("wires.txt");

    let wire_one = Wire::from_tokens(&wires[0]);
    let wire_two = Wire::from_tokens(&wires[1]);

    let intersections = wire_one.intersection(&wire_two); //wire_intersections(&wire_one, &wire_two);

    println!(
        "distance: {} steps: {}",
        min_distance(&intersections).unwrap(),
        min_steps(&intersections, &wire_one, &wire_two).unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one_example_one() {
        let wire_one = Wire::from_tokens(&string_to_tokens("R75,D30,R83,U83,L12,D49,R71,U7,L72"));

        let wire_two = Wire::from_tokens(&string_to_tokens("U62,R66,U55,R34,D71,R55,D58,R83"));

        assert_eq!(
            min_distance(&wire_one.intersection(&wire_two)).unwrap(),
            159
        );
    }

    #[test]
    fn test_part_one_example_two() {
        let wire_one = Wire::from_tokens(&string_to_tokens(
            "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
        ));

        let wire_two = Wire::from_tokens(&string_to_tokens("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"));

        assert_eq!(
            min_distance(&wire_one.intersection(&wire_two)).unwrap(),
            135
        );
    }

    #[test]
    fn test_part_two_example_one() {
        let wire_one = Wire::from_tokens(&string_to_tokens("R75,D30,R83,U83,L12,D49,R71,U7,L72"));

        let wire_two = Wire::from_tokens(&string_to_tokens("U62,R66,U55,R34,D71,R55,D58,R83"));

        assert_eq!(
            min_steps(&wire_one.intersection(&wire_two), &wire_one, &wire_two).unwrap(),
            610
        );
    }

    #[test]
    fn test_part_two_example_two() {
        let wire_one = Wire::from_tokens(&string_to_tokens(
            "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51",
        ));

        let wire_two = Wire::from_tokens(&string_to_tokens("U98,R91,D20,R16,D67,R40,U7,R15,U6,R7"));

        assert_eq!(
            min_steps(&wire_one.intersection(&wire_two), &wire_one, &wire_two).unwrap(),
            410
        );
    }
}
