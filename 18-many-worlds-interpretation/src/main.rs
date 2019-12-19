//! The travelling salesman says hi?
//!
//! I spent a long time trying to figure out a "correct" solution to this, before seeing that most
//! on /r/adventofcode treated it as the travelling saleman problem. It may be possible to optimise
//! by calculating the path from each key to each other key only once, and keeping track of which
//! doors are blocking the path before determing if the path is valid or not. I haven't done that
//! since running with --release presents an answer within a couple of minutes.
//!
//! See `minimum_steps` for the main calculation.

use std::collections::{HashMap, HashSet, VecDeque};
use std::{fs, io};

#[derive(Debug, PartialEq, Eq)]
enum TileType {
    Wall,
    Empty,
    Key(char),
    Door(char),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Pos(u32, u32);

impl Pos {
    /// Returns a vector of all the neighbors of this position. May include positions which are not
    /// part of the map (e.g. (-1, -1)).
    fn neighbors(&self) -> Vec<Pos> {
        vec![
            Pos(self.0 - 1, self.1),
            Pos(self.0 + 1, self.1),
            Pos(self.0, self.1 - 1),
            Pos(self.0, self.1 + 1),
        ]
    }

    /// Returns a vector of neighbors to the position which may be visited by a Robot.
    fn visitable_neighbors(&self, map: &Map) -> Vec<Pos> {
        self.neighbors()
            .into_iter()
            .filter(|pos| map.visitable(pos))
            .collect::<Vec<Pos>>()
    }
}

/// Describes a path through the Map from a starting position (not contained in the struct). The
/// Path contains the distance from the start to the `end_position`, and the ID of which robot can
/// traverse the path (always 0 in part 1, 0-3 in part 2).
struct Path {
    distance: u32,
    end_position: Pos,
    robot_id: usize,
}

struct Map {
    inner: HashMap<Pos, TileType>,
    starts: Vec<Pos>,
}

impl Map {
    /// Returns if the given position may be visited by the robot.
    fn visitable(&self, position: &Pos) -> bool {
        match self.inner.get(position) {
            Some(TileType::Wall) | None => false,
            _ => true,
        }
    }

    /// Returns a HashMap where each key is the ID of a reachable key in the map, and each value is
    /// a tuple containing the distance from the start position, and the position.
    fn reachable_keys(&self, start: Pos, have: &CharMaskSet) -> HashMap<char, Path> {
        let mut reachable = HashMap::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut distance: u32 = 1;

        queue.push_back(start);

        while queue.len() != 0 {
            let mut new_queue = VecDeque::new();

            while let Some(pos) = queue.pop_front() {
                for neighbor in pos.visitable_neighbors(self) {
                    // We've been here.
                    if visited.contains(&neighbor) {
                        continue;
                    }

                    // Ignore if wall.
                    if let Some(TileType::Wall) = self.inner.get(&neighbor) {
                        continue;
                    }

                    // Ignore if door for which we don't have a key.
                    if let Some(TileType::Door(character)) = self.inner.get(&neighbor) {
                        if !have.contains(&character.to_ascii_lowercase()) {
                            continue;
                        }
                    }

                    visited.insert(pos);

                    if let Some(TileType::Key(character)) = self.inner.get(&neighbor) {
                        if have.contains(character) {
                            // Already have this key: keep going.
                            new_queue.push_back(neighbor);
                        } else {
                            // If we collected a key, add it. No point in traversing further as any
                            // other keys we find will be more distant on this path than this one.
                            reachable.insert(
                                *character,
                                Path {
                                    distance: distance,
                                    end_position: neighbor,
                                    robot_id: 0,
                                },
                            );
                        }
                    } else {
                        // Otherwise keep traversing.
                        new_queue.push_back(neighbor);
                    }
                }
            }

            queue = new_queue;
            distance += 1;
        }

        reachable
    }

    /// Computes the reachable keys from each start positions.
    ///
    /// The returned HashMap contains characters as the keys, and a tuple of containing the distance
    /// to the key, the Pos(ition) of the key, and the the start position index from which the key
    /// is reachable.
    fn reachable_keys_multiple(
        &self,
        starts: &Vec<Pos>,
        have: &CharMaskSet,
    ) -> HashMap<char, Path> {
        let mut keys = HashMap::new();

        for (index, start) in starts.iter().enumerate() {
            for (
                character,
                Path {
                    distance,
                    end_position,
                    ..
                },
            ) in self.reachable_keys(*start, have)
            {
                keys.insert(
                    character,
                    Path {
                        distance,
                        end_position,
                        robot_id: index,
                    },
                );
            }
        }

        keys
    }
}

impl From<String> for Map {
    fn from(input: String) -> Map {
        let mut inner = HashMap::new();
        let mut starts = Vec::new();

        for (y, line) in input.lines().enumerate() {
            for (x, character) in line.chars().enumerate() {
                let tile_type = match character {
                    'a'..='z' => TileType::Key(character),
                    'A'..='Z' => TileType::Door(character),
                    '@' => {
                        starts.push(Pos(x as u32, y as u32));
                        TileType::Empty
                    }
                    '.' => TileType::Empty,
                    '#' => TileType::Wall,
                    _ => panic!("Unexpected map character: {}", character),
                };

                inner.insert(Pos(x as u32, y as u32), tile_type);
            }
        }

        Map { inner, starts }
    }
}

/// Describes which keys we already have.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct CharMaskSet(u32);

impl CharMaskSet {
    fn new() -> CharMaskSet {
        CharMaskSet(0)
    }

    fn contains(&self, character: &char) -> bool {
        self.0 & CharMaskSet::char_to_mask(*character) != 0
    }

    fn insert(&mut self, character: char) {
        self.0 += CharMaskSet::char_to_mask(character);
    }

    /// Creates a clone of the CharMaskSet and adds the `character` to the new CharMaskSet.
    fn clone_insert(&self, character: char) -> CharMaskSet {
        let mut new = self.clone();
        new.insert(character);

        new
    }

    #[inline(always)]
    fn char_to_mask(character: char) -> u32 {
        if !character.is_ascii_lowercase() {
            panic!(
                "CharMaskSet may only contain lowercase ASCII. Got: {}",
                character
            );
        }

        1 << (character as u8 - 'a' as u8)
    }
}

impl From<&Vec<char>> for CharMaskSet {
    // Assumes that all characters are lowercase ASCII.
    fn from(characters: &Vec<char>) -> CharMaskSet {
        let mut mask = 0;

        for character in characters {
            mask += CharMaskSet::char_to_mask(*character);
        }

        CharMaskSet(mask)
    }
}

/// Given a vector of start positions, returns a suitable hash key to represent them.
///
/// This feels terribly ugly, but neatly works around &Vec<Pos> not being hashable.
fn starts_key(starts: &Vec<Pos>) -> String {
    starts
        .iter()
        .map(|start| format!("{:?}", start))
        .collect::<Vec<String>>()
        .join("")
}

/// Calculate the minimum steps to collect all keys.
///
/// map - The parsed map.
/// starts - A vector of positions where a robot is located.
/// have - A CharMaskSet containing the keys already collected.
/// seen - A cache of starting positions and collected keys to reduce the number of calculations.
fn minimum_steps(
    map: &Map,
    starts: &Vec<Pos>,
    have: CharMaskSet,
    seen: &mut HashMap<(String, CharMaskSet), u32>,
) -> u32 {
    let cache_key = (starts_key(starts), have);

    if let Some(steps) = seen.get(&cache_key) {
        return *steps;
    }

    let keys = map.reachable_keys_multiple(starts, &have);

    if keys.len() == 0 {
        // All keys are collected when there area no reachable keys
        return 0;
    }

    let mut min_steps = u32::max_value();

    for (character, path) in keys {
        let new_starts = starts
            .iter()
            .enumerate()
            .map(|(index, p)| {
                // Only move the robot we're currently calculating (path.robot_id) to the end of the
                // current path; leave the others at their current position.
                if index == path.robot_id {
                    path.end_position
                } else {
                    *p
                }
            })
            .collect::<Vec<Pos>>();

        let distance =
            path.distance + minimum_steps(&map, &new_starts, have.clone_insert(character), seen);

        if distance < min_steps {
            min_steps = distance;
        }
    }

    // `seen` keeps track of start positions and the keys already collected, and maps them to the
    // minimum number of steps.
    seen.insert(cache_key.clone(), min_steps);

    min_steps
}

/// Computes the shortest path to collect all keys.
fn shortest_path(map: Map) -> u32 {
    let mut seen = HashMap::new();
    minimum_steps(&map, &map.starts, CharMaskSet::new(), &mut seen)
}

fn main() -> Result<(), io::Error> {
    let map = fs::read_to_string("data/map.p1.txt")?;
    let map = Map::from(map);

    println!("Part one: {:?}", shortest_path(map));

    let map = fs::read_to_string("data/map.p2.txt")?;
    let map = Map::from(map);

    println!("Part two: {:?}", shortest_path(map));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trim_leading_whitespace(string: &str) -> String {
        let lines: Vec<&str> = string.lines().map(|line| line.trim()).collect();
        lines.join("\n")
    }

    #[test]
    fn test_part_one_first_example() {
        let map = Map::from(trim_leading_whitespace(
            "#########
             #b.A.@.a#
             #########",
        ));

        assert_eq!(shortest_path(map), 8);
    }

    #[test]
    fn test_part_one_second_example() {
        let map = Map::from(trim_leading_whitespace(
            "########################
             #f.D.E.e.C.b.A.@.a.B.c.#
             ######################.#
             #d.....................#
             ########################",
        ));

        assert_eq!(shortest_path(map), 86);
    }

    #[test]
    fn test_part_one_third_example() {
        let map = Map::from(trim_leading_whitespace(
            "########################
             #...............b.C.D.f#
             #.######################
             #.....@.a.B.c.d.A.e.F.g#
             ########################",
        ));

        assert_eq!(shortest_path(map), 132);
    }

    #[test]
    fn test_part_one_fourth_example() {
        let map = Map::from(trim_leading_whitespace(
            "#################
             #i.G..c...e..H.p#
             ########.########
             #j.A..b...f..D.o#
             ########@########
             #k.E..a...g..B.n#
             ########.########
             #l.F..d...h..C.m#
             #################",
        ));

        assert_eq!(shortest_path(map), 136);
    }

    #[test]
    fn test_part_one_fifth_example() {
        let map = Map::from(trim_leading_whitespace(
            "########################
             #@..............ac.GI.b#
             ###d#e#f################
             ###A#B#C################
             ###g#h#i################
             ########################",
        ));

        assert_eq!(shortest_path(map), 81);
    }

    // Test fails. All others, including the real problem, pass fine. :|
    #[test]
    fn test_part_two_first_example() {
        let map = Map::from(trim_leading_whitespace(
            "#######
              #a.#Cd#
              ##@#@##
              #######
              ##@#@##
              #cB#Ab#
              #######",
        ));

        assert_eq!(shortest_path(map), 8);
    }

    #[test]
    fn test_part_two_second_example() {
        let map = Map::from(trim_leading_whitespace(
            "###############
             #d.ABC.#.....a#
             ######@#@######
             ###############
             ######@#@######
             #b.....#.....c#
             ###############",
        ));

        assert_eq!(shortest_path(map), 24);
    }

    #[test]
    fn test_part_two_third_example() {
        let map = Map::from(trim_leading_whitespace(
            "#############
             #DcBa.#.GhKl#
             #.###@#@#I###
             #e#d#####j#k#
             ###C#@#@###J#
             #fEbA.#.FgHi#
             #############",
        ));

        assert_eq!(shortest_path(map), 32);
    }

    #[test]
    fn test_part_two_fourth_example() {
        let map = Map::from(trim_leading_whitespace(
            "#############
             #g#f.D#..h#l#
             #F###e#E###.#
             #dCba@#@BcIJ#
             #############
             #nK.L@#@G...#
             #M###N#H###.#
             #o#m..#i#jk.#
             #############",
        ));

        assert_eq!(shortest_path(map), 72);
    }

    #[test]
    fn test_char_mask_set() {
        let mut set = CharMaskSet::from(&vec!['a', 'c', 'd']);

        assert!(set.contains(&'a'));
        assert!(set.contains(&'c'));
        assert!(set.contains(&'d'));
        assert!(!set.contains(&'b'));
        assert!(!set.contains(&'e'));

        set.insert('e');
        assert!(set.contains(&'e'));
        assert!(!set.contains(&'b'));
    }

    #[test]
    #[should_panic(expected = "CharMaskSet may only contain lowercase ASCII. Got: A")]
    fn test_char_set_mask_uppercase() {
        CharMaskSet::from(&vec!['A']);
    }
}
