use std::collections::{HashMap, HashSet};
use std::{fs, io};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TileType {
    Empty,
    Infested,
    RecursiveMap,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Pos(i32, i32);

impl Pos {
    /// Returns a vector of all the neighbors of this position. May include positions which are past
    /// the edges of the map.
    fn neighbors(&self) -> Vec<Pos> {
        vec![
            Pos(self.0 - 1, self.1),
            Pos(self.0 + 1, self.1),
            Pos(self.0, self.1 - 1),
            Pos(self.0, self.1 + 1),
        ]
    }
}

struct Map {
    inner: HashMap<Pos, TileType>,
    layer: usize,
}

impl Map {
    fn new(layer: usize) -> Map {
        let mut inner = HashMap::new();

        for y in 0..5 {
            for x in 0..5 {
                inner.insert(Pos(x, y), TileType::Empty);
            }
        }

        Map { inner, layer }
    }

    /// Parsing a map works by first reading the data from the string into an intermediate hashmap
    /// containing each character in the map, and their positions. From this representation its
    /// easier to read the portal names and positions. This intermediate hashmap is then used to
    /// build the real map.
    fn from_str(input: String, layer: usize) -> Map {
        let mut inner = HashMap::with_capacity(input.len());

        for (y, line) in input.lines().enumerate() {
            for (x, character) in line.chars().enumerate() {
                inner.insert(
                    Pos(x as i32, y as i32),
                    match character {
                        '#' => TileType::Infested,
                        '?' => TileType::RecursiveMap,
                        _ => TileType::Empty,
                    },
                );
            }
        }

        Map {
            inner,
            layer: layer,
        }
    }

    /// Returns if the given position may be visited. Allows an optional MultiMap to be provided,
    /// in which case the layer above and below will also be included.
    fn infested_neighbors(&self, position: &Pos, multi: Option<&MultiMap>) -> usize {
        let immediate_neighbors = position
            .neighbors()
            .into_iter()
            .filter(|neighbor| match self.inner.get(&neighbor) {
                Some(TileType::Infested) => true,
                _ => false,
            })
            .count();

        let mut below = 0;
        let mut above = 0;

        if let Some(multi) = multi {
            if let Some(map_below) = multi.0.get(self.layer + 1) {
                below = map_below.infested_neighbors_from_above(position);
            }

            if self.layer > 0 {
                if let Some(map_above) = multi.0.get(self.layer - 1) {
                    above = map_above.infested_neighbors_from_below(position);
                }
            }
        }

        immediate_neighbors + below + above
    }

    /// Receives a position from the map one layer below, and returns the number of infested
    /// neighbors on this layer.
    fn infested_neighbors_from_below(&self, position: &Pos) -> usize {
        let mut neighbors = Vec::with_capacity(2);

        if position.0 == 0 {
            // Left
            neighbors.push(Pos(1, 2));
        }

        if position.0 == 4 {
            // Right
            neighbors.push(Pos(3, 2));
        }

        if position.1 == 0 {
            // Top
            neighbors.push(Pos(2, 1));
        }

        if position.1 == 4 {
            // Bottom
            neighbors.push(Pos(2, 3));
        }

        neighbors
            .into_iter()
            .filter(|neighbor| match self.inner.get(&neighbor) {
                Some(TileType::Infested) => true,
                _ => false,
            })
            .count()
    }

    /// Takes a position from the map one layer above this one, and returns how many tiles in this
    /// map which neighbor it are infested.
    fn infested_neighbors_from_above(&self, position: &Pos) -> usize {
        let mut neighbors = Vec::new();

        if position.0 == 2 && position.1 == 1 {
            neighbors = vec![Pos(0, 0), Pos(1, 0), Pos(2, 0), Pos(3, 0), Pos(4, 0)];
        }

        if position.0 == 3 && position.1 == 2 {
            neighbors = vec![Pos(4, 0), Pos(4, 1), Pos(4, 2), Pos(4, 3), Pos(4, 4)];
        }

        if position.0 == 2 && position.1 == 3 {
            neighbors = vec![Pos(0, 4), Pos(1, 4), Pos(2, 4), Pos(3, 4), Pos(4, 4)];
        }

        if position.0 == 1 && position.1 == 2 {
            neighbors = vec![Pos(0, 0), Pos(0, 1), Pos(0, 2), Pos(0, 3), Pos(0, 4)];
        }

        neighbors
            .into_iter()
            .filter(|neighbor| match self.inner.get(&neighbor) {
                Some(TileType::Infested) => true,
                _ => false,
            })
            .count()
    }

    fn height(&self) -> usize {
        (self.inner.keys().max_by_key(|Pos(_, y)| y).unwrap().1 + 1) as usize
    }

    fn width(&self) -> usize {
        (self.inner.keys().max_by_key(|Pos(x, _)| x).unwrap().0 + 1) as usize
    }

    /// Calculatest the biodiversity rating of the map. Each tile position is multiplied by an
    /// increasing power of two, from left-to-right, each row at a time.
    fn biodiversity(&self) -> i32 {
        let mut power = 1;
        let mut bio = 0;

        for y in 0..self.height() {
            for x in 0..self.width() {
                if let Some(TileType::Infested) = self.inner.get(&Pos(x as i32, y as i32)) {
                    bio += power;
                }

                power = power * 2;
            }
        }

        bio
    }

    // Creates a new Map, stepping forward once in the simulation. An Infested tile with exactly one
    // infested neighbor becomes empty. A Empty tile with oen or two Infested neighbors becomes
    // infested.
    fn step_forward(&self, multi: Option<&MultiMap>) -> Map {
        let mut new_inner = self.inner.clone();

        for y in 0..self.height() {
            for x in 0..self.width() {
                let position = Pos(x as i32, y as i32);
                let infested_neighbors = self.infested_neighbors(&position, multi);

                match self.inner.get(&position) {
                    Some(TileType::Infested) => {
                        if infested_neighbors != 1 {
                            new_inner.insert(position, TileType::Empty);
                        }
                    }
                    Some(TileType::Empty) => {
                        if infested_neighbors == 1 || infested_neighbors == 2 {
                            new_inner.insert(position, TileType::Infested);
                        }
                    }
                    Some(TileType::RecursiveMap) => {}
                    None => {}
                }
            }
        }

        Map {
            inner: new_inner,
            layer: self.layer,
        }
    }
}

impl From<String> for Map {
    /// Parsing a map works by first reading the data from the string into an intermediate hashmap
    /// containing each character in the map, and their positions. From this representation its
    /// easier to read the portal names and positions. This intermediate hashmap is then used to
    /// build the real map.
    fn from(input: String) -> Map {
        Map::from_str(input, 0)
    }
}

struct MultiMap(Vec<Map>);

impl MultiMap {
    fn new(middle: Map, layers: usize) -> MultiMap {
        let mut maps = Vec::with_capacity(layers);
        let mut middle = middle;
        let mid_layer = middle.layer;

        for i in 0..layers {
            let mut map = Map::new(i);

            if i < layers - 1 {
                map.inner.insert(Pos(2, 2), TileType::RecursiveMap);
            }

            maps.push(map);
        }

        if mid_layer < layers - 1 {
            middle.inner.insert(Pos(2, 2), TileType::RecursiveMap);
        }

        maps[mid_layer] = middle;

        MultiMap(maps)
    }

    fn step_forward(&self) -> MultiMap {
        MultiMap(
            self.0
                .iter()
                .map(|map| map.step_forward(Some(self)))
                .collect::<Vec<_>>(),
        )
    }
}

fn part_one(map: Map) -> i32 {
    let mut map = map;
    let mut seen = HashSet::new();

    seen.insert(map.biodiversity());

    loop {
        map = map.step_forward(None);
        let bio = map.biodiversity();

        if seen.contains(&bio) {
            return bio;
        }

        seen.insert(bio);
    }
}

fn part_two(map: Map) -> usize {
    let mut map = map;
    map.layer = 100;

    let mut multi = MultiMap::new(map, 250);

    for _ in 0..200 {
        multi = multi.step_forward();
    }

    let mut count = 0;

    for map in multi.0 {
        count += map
            .inner
            .values()
            .filter(|&&tile| tile == TileType::Infested)
            .count();
    }

    count
}

fn main() -> Result<(), io::Error> {
    let map = Map::from(fs::read_to_string("data/map.txt")?);
    println!("Part one: {}", part_one(map));

    let map = Map::from(fs::read_to_string("data/map.txt")?);
    println!("Part two: {}", part_two(map));

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
    fn test_parse_map() {
        let map = Map::from(trim_leading_whitespace(
            "....#
             #..#.
             #..##
             ..#..
             #....",
        ));

        assert_eq!(map.inner.get(&Pos(-1, -1)), None);
        assert_eq!(map.inner.get(&Pos(0, 0)), Some(&TileType::Empty));
        assert_eq!(map.inner.get(&Pos(0, 4)), Some(&TileType::Infested));
        assert_eq!(map.inner.get(&Pos(4, 0)), Some(&TileType::Infested));
        assert_eq!(map.inner.get(&Pos(4, 1)), Some(&TileType::Empty));
        assert_eq!(map.inner.get(&Pos(4, 4)), Some(&TileType::Empty));
        assert_eq!(map.inner.get(&Pos(4, 5)), None);
        assert_eq!(map.inner.get(&Pos(5, 0)), None);
    }

    #[test]
    fn test_infested_neighbors() {
        let map = Map::from(trim_leading_whitespace(
            "....#
             #..#.
             #..##
             ..#..
             #....",
        ));

        let infested = map.infested_neighbors(&Pos(0, 0), None);
        assert_eq!(infested, 1);

        let infested = map.infested_neighbors(&Pos(3, 2), None);
        assert_eq!(infested, 2);

        let infested = map.infested_neighbors(&Pos(1, 0), None);
        assert_eq!(infested, 0);
    }

    #[test]
    fn test_biodiversity_rating() {
        let map = Map::from(trim_leading_whitespace(
            ".....
             .....
             .....
             #....
             .#...",
        ));

        assert_eq!(map.biodiversity(), 2129920);
    }

    #[test]
    fn test_step_forward() {
        let map = Map::from(trim_leading_whitespace(
            "....#
             #..#.
             #..##
             ..#..
             #....",
        ));

        let new = map.step_forward(None);

        assert_eq!(new.inner.get(&Pos(0, 0)), Some(&TileType::Infested));

        assert_eq!(new.inner.get(&Pos(0, 1)), Some(&TileType::Infested));
        assert_eq!(new.inner.get(&Pos(1, 1)), Some(&TileType::Infested));
        assert_eq!(new.inner.get(&Pos(2, 1)), Some(&TileType::Infested));
        assert_eq!(new.inner.get(&Pos(3, 1)), Some(&TileType::Infested));
        assert_eq!(new.inner.get(&Pos(4, 1)), Some(&TileType::Empty));

        assert_eq!(new.inner.get(&Pos(0, 4)), Some(&TileType::Empty));
        assert_eq!(new.inner.get(&Pos(1, 4)), Some(&TileType::Infested));
        assert_eq!(new.inner.get(&Pos(4, 4)), Some(&TileType::Empty));
    }

    #[test]
    fn test_create_multimap() {
        let map = Map::from_str(
            trim_leading_whitespace(
                "....#
                 #..#.
                 #..##
                 ..#..
                 #....",
            ),
            1,
        );

        let multi = MultiMap::new(map, 3);

        // All tiles on the upper level are empty, exept for the middle which is a RecursiveMap.
        assert_eq!(
            multi.0[0]
                .inner
                .values()
                .filter(|&&tt| tt == TileType::Empty)
                .count(),
            24
        );

        assert_eq!(
            multi.0[0].inner.get(&Pos(2, 2)),
            Some(&TileType::RecursiveMap)
        );

        // The middle map is the original.
        assert_eq!(
            multi.0[1]
                .inner
                .values()
                .filter(|&&tt| tt == TileType::Infested)
                .count(),
            8
        );

        assert_eq!(
            multi.0[1].inner.get(&Pos(2, 2)),
            Some(&TileType::RecursiveMap)
        );

        // All tiles on the lower level are empty?
        assert_eq!(
            multi.0[2]
                .inner
                .values()
                .filter(|&&tt| tt == TileType::Empty)
                .count(),
            25
        );
    }

    #[test]
    fn test_neighbors_above() {
        let map = Map::from(trim_leading_whitespace(
            "#..##
             ...##
             ..?..
             ...#.
             .####",
        ));

        // Neighbors are the left column.
        assert_eq!(map.infested_neighbors_from_above(&Pos(1, 2)), 1);

        // Neighbors are the top row.
        assert_eq!(map.infested_neighbors_from_above(&Pos(2, 1)), 3);

        // Neighbors are the right column.
        assert_eq!(map.infested_neighbors_from_above(&Pos(3, 2)), 3);

        // Neighbors are the bottom row.
        assert_eq!(map.infested_neighbors_from_above(&Pos(2, 3)), 4);

        // No neighbors.
        assert_eq!(map.infested_neighbors_from_above(&Pos(0, 0)), 0);
        assert_eq!(map.infested_neighbors_from_above(&Pos(4, 4)), 0);
        assert_eq!(map.infested_neighbors_from_above(&Pos(1, 1)), 0);
    }

    #[test]
    fn test_neighbors_below() {
        let map = Map::from(trim_leading_whitespace(
            ".....
             ..#..
             ..?#.
             ...#.
             .....",
        ));

        // Neighbors ara the upper middle cell and left middle cell.
        assert_eq!(map.infested_neighbors_from_below(&Pos(0, 0)), 1);

        // Neighbor is the upper middle cell.
        assert_eq!(map.infested_neighbors_from_below(&Pos(1, 0)), 1);
        assert_eq!(map.infested_neighbors_from_below(&Pos(2, 0)), 1);
        assert_eq!(map.infested_neighbors_from_below(&Pos(3, 0)), 1);

        // Neighbor is the upper middle cell and right middle cell.
        assert_eq!(map.infested_neighbors_from_below(&Pos(4, 0)), 2);

        // Neighbor is the right middle cell.
        assert_eq!(map.infested_neighbors_from_below(&Pos(4, 1)), 1);
        assert_eq!(map.infested_neighbors_from_below(&Pos(4, 2)), 1);
        assert_eq!(map.infested_neighbors_from_below(&Pos(4, 3)), 1);

        // Neighbor is the lower middle cell and right middle cell.
        assert_eq!(map.infested_neighbors_from_below(&Pos(4, 4)), 1);

        // Neighbor is the lower middle cell.
        assert_eq!(map.infested_neighbors_from_below(&Pos(1, 4)), 0);
        assert_eq!(map.infested_neighbors_from_below(&Pos(2, 4)), 0);
        assert_eq!(map.infested_neighbors_from_below(&Pos(3, 4)), 0);

        // // No neighbors.
        assert_eq!(map.infested_neighbors_from_above(&Pos(0, 0)), 0);
        assert_eq!(map.infested_neighbors_from_above(&Pos(4, 4)), 0);
        assert_eq!(map.infested_neighbors_from_above(&Pos(1, 1)), 0);
    }
}
