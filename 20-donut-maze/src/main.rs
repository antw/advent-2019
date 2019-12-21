use std::collections::HashMap;
use std::{fs, io};

extern crate pathfinding;
use pathfinding::directed::bfs::bfs;

#[derive(Debug, PartialEq, Eq)]
enum TileType {
    Wall,
    Empty,
    // Portal contains the position of the other end of the portal and whether the portal recurses
    // deeper into the maze (1) or goes back towards the original maze (-1).
    Portal(Pos, i32),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Pos(u32, u32);

impl Pos {
    /// Returns a vector of all the neighbors of this position. May include positions which are past
    /// the edges of the map.
    fn neighbors(&self) -> Vec<Pos> {
        // cap 5 because "including_portals" may add another Pos.
        let mut neighbors = Vec::with_capacity(5);

        neighbors.push(Pos(self.0 + 1, self.1));
        neighbors.push(Pos(self.0, self.1 + 1));

        if self.0 > 0 {
            neighbors.push(Pos(self.0 - 1, self.1));
        }

        if self.1 > 0 {
            neighbors.push(Pos(self.0, self.1 - 1));
        }

        neighbors
    }

    fn neighbors_including_portals(&self, map: &Map) -> Vec<Pos> {
        let mut neighbors = self.neighbors();

        if let Some(TileType::Portal(other_pos, _)) = map.inner.get(self) {
            neighbors.push(*other_pos);
        }

        neighbors
    }

    /// Returns a vector of neighbors to the position which may be visited by a Robot and whether
    /// visiting the position causes the maze recursion to increase (+1), stay the same (0), or
    /// become less (-1).
    fn visitable_neighbors(&self, map: &Map, current_layer: i32) -> Vec<(Pos, i32)> {
        self.neighbors_including_portals(map)
            .into_iter()
            .filter(|pos| map.visitable(pos))
            .map(|pos| match map.inner.get(&pos) {
                Some(TileType::Portal(_, layer_delta)) => {
                    if let Some(TileType::Portal(_, _)) = map.inner.get(self) {
                        // If this tile is a portal and the neighbor is a matching portal, then
                        // change the layer level.
                        (pos, current_layer - layer_delta)
                    } else {
                        (pos, current_layer)
                    }
                }
                _ => (pos, current_layer),
            })
            .collect::<Vec<(Pos, i32)>>()
    }
}

fn is_portal_tile(map: &HashMap<Pos, char>, pos: &Pos) -> bool {
    if let Some(character) = map.get(pos) {
        match character {
            'A'..='Z' => return true,
            _ => return false,
        }
    }

    false
}

/// Determines from the position and map size whether a portal at the position is an inner portal
/// (which increases the layer level) or an outer portal (which decreases it).
fn portal_layer_delta(pos: &Pos, map_width: u32, map_height: u32) -> i32 {
    if pos.0 <= 3 || pos.0 >= map_width - 3 || pos.1 <= 3 || pos.1 >= map_height - 3 {
        1
    } else {
        -1
    }
}

/// Determines if the tile has a connection to the map (an Empty tile). This helps when parsing
/// portal names: a tile with part of a name which isn't connected to the map is just part of the
/// name, and not a portal.
fn is_connected_tile(map: &HashMap<Pos, char>, pos: &Pos) -> Option<Pos> {
    for neighbor in pos.neighbors() {
        match map.get(&neighbor) {
            Some('.') => return Some(neighbor),
            _ => {}
        }
    }

    None
}

/// Given the intermediate map and a position, determines the name of the portal at the position.
fn portal_key_from(map: &HashMap<Pos, char>, pos: &Pos) -> String {
    let character = map.get(&pos).unwrap();
    let Pos(x, y) = &pos;

    let right = Pos(x + 1, *y);
    let down = Pos(*x, y + 1);

    // TODO: This is ugly.
    if is_portal_tile(map, &right) {
        format!("{}{}", character, map.get(&right).unwrap())
    } else if is_portal_tile(map, &down) {
        format!("{}{}", character, map.get(&down).unwrap())
    } else if *x > 0 && is_portal_tile(map, &Pos(x - 1, *y)) {
        format!("{}{}", map.get(&Pos(x - 1, *y)).unwrap(), character)
    } else if *y > 0 && is_portal_tile(map, &Pos(*x, y - 1)) {
        format!("{}{}", map.get(&Pos(*x, y - 1)).unwrap(), character)
    } else {
        panic!("Not a two-character portal at: {:?}", pos);
    }
}

struct Map {
    inner: HashMap<Pos, TileType>,
    start: Pos,
    exit: Pos,
}

impl Map {
    /// Returns if the given position may be visited.
    fn visitable(&self, position: &Pos) -> bool {
        match self.inner.get(position) {
            Some(TileType::Wall) | None => false,
            _ => true,
        }
    }
}

impl From<String> for Map {
    /// Parsing a map works by first reading the data from the string into an intermediate hashmap
    /// containing each character in the map, and their positions. From this representation its
    /// easier to read the portal names and positions. This intermediate hashmap is then used to
    /// build the real map.
    fn from(input: String) -> Map {
        // Keep track of the first portal of each key found.
        let mut portals: HashMap<String, Pos> = HashMap::new();

        let mut map = HashMap::with_capacity(input.len());
        let mut intermediate = HashMap::with_capacity(input.len());

        let mut start = Pos(0, 0);
        let mut exit = Pos(0, 0);

        // Start by parsing the string map into a HashMap of characters.
        for (y, line) in input.lines().enumerate() {
            for (x, character) in line.chars().enumerate() {
                intermediate.insert(Pos(x as u32, y as u32), character);
            }
        }

        let map_width = intermediate.keys().max_by_key(|Pos(x, _)| x).unwrap().0 + 1;
        let map_height = intermediate.keys().max_by_key(|Pos(_, y)| y).unwrap().1 + 1;

        // For each character in the intermediate map, create an appropriate tiletype in the real
        // map.
        for (pos, character) in &intermediate {
            match character {
                'A'..='Z' => {
                    // Portal. Check that one of the neighbors is an empty tile and can be visited
                    // otherwise this is just part of the portal ID and not visitable.
                    if let Some(empty_pos) = is_connected_tile(&intermediate, &pos) {
                        let portal_key = portal_key_from(&intermediate, &pos);

                        // If this is the entry or exit portal, store the position.
                        if portal_key == "AA".to_string() {
                            start = empty_pos;
                            continue;
                        } else if portal_key == "ZZ".to_string() {
                            exit = empty_pos;
                            continue;
                        }

                        // This portal tile is connected to the map. If we already have the other
                        // portal in `portals`, we can add both to the map. Otherwise we have to add
                        // this one to the `portals` vec and wait until we've found the other.
                        if let Some(other_pos) = portals.get(&portal_key) {
                            map.insert(
                                empty_pos,
                                TileType::Portal(
                                    *other_pos,
                                    portal_layer_delta(other_pos, map_width, map_height),
                                ),
                            );

                            map.insert(
                                *other_pos,
                                TileType::Portal(
                                    empty_pos,
                                    portal_layer_delta(&empty_pos, map_width, map_height),
                                ),
                            );
                        } else {
                            portals.insert(portal_key, empty_pos);
                        }
                    }
                }
                '.' => {
                    // There may be a portal here. If so, leave it.
                    map.entry(*pos).or_insert(TileType::Empty);
                }
                '#' => {
                    map.insert(*pos, TileType::Wall);
                }
                ' ' => {}
                other => panic!("Unknown map tile: {}", other),
            }
        }

        Map {
            inner: map,
            start,
            exit,
        }
    }
}

/// Calculates the minimum number of steps required to traverse a non-recursive maze.
fn part_one(map: Map) -> usize {
    bfs(
        &map.start,
        |&pos| {
            pos.visitable_neighbors(&map, 0)
                .into_iter()
                .map(|(pos, _)| pos)
        },
        |pos| pos == &map.exit,
    )
    .expect("Expected to find path to the exit")
    .len()
        - 1
}

/// Calculates the minimum number of steps required to traverse a recursive maze where each "inner"
/// portal transports the traveller to a copy of the maze one level deeper, and each "outer" portal
/// returns us one level higher. Only once reaching "ZZ" at layer 0 have we completed the maze.
fn part_two(map: Map) -> usize {
    bfs(
        &(map.start, 0),
        |&(pos, layer)| {
            pos.visitable_neighbors(&map, layer)
                .into_iter()
                // If we're already at the top maze level, we cannot go through an outer portal as
                // that would lead to a negative level.
                .filter(|(_, level)| *level >= 0)
        },
        |&(pos, layer)| pos == map.exit && layer == 0,
    )
    .expect("Expected to find path to the exit")
    .len()
        - 1
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

    #[test]
    fn test_parse_simple_map() {
        let map = Map::from(
            "         A
         A
  #######.#########
  #######.........#
  #######.#######.#
  #######.#######.#
  #######.#######.#
  #####  B    ###.#
BC...##  C    ###.#
  ##.##       ###.#
  ##...DE  F  ###.#
  #####    G  ###.#
  #########.#####.#
DE..#######...###.#
  #.#########.###.#
FG..#########.....#
  ###########.#####
             Z
             Z"
            .to_string(),
        );

        assert_eq!(part_one(map), 23);
    }

    #[test]
    fn test_portal_layer_delta() {
        let map = Map::from(
            "         A
         A
  #######.#########
  #######.........#
  #######.#######.#
  #######.#######.#
  #######.#######.#
  #####  B    ###.#
BC...##  C    ###.#
  ##.##       ###.#
  ##...DE  F  ###.#
  #####    G  ###.#
  #########.#####.#
DE..#######...###.#
  #.#########.###.#
FG..#########.....#
  ###########.#####
             Z
             Z"
            .to_string(),
        );

        // Travelling through outer portal decreases layer level.
        assert_eq!(
            map.inner.get(&Pos(2, 8)),
            Some(&TileType::Portal(Pos(9, 6), -1))
        );

        // Travelling through inner portal increases layer level.
        assert_eq!(
            map.inner.get(&Pos(9, 6)),
            Some(&TileType::Portal(Pos(2, 8), 1))
        );

        // BC inner -> BC outer
        assert_eq!(
            Pos(9, 6).visitable_neighbors(&map, 0),
            vec![(Pos(9, 5), 0), (Pos(2, 8), 1)]
        )
    }

    #[test]
    fn test_parse_complex_map() {
        let map = Map::from(
            "                   A
                   A
  #################.#############
  #.#...#...................#.#.#
  #.#.#.###.###.###.#########.#.#
  #.#.#.......#...#.....#.#.#...#
  #.#########.###.#####.#.#.###.#
  #.............#.#.....#.......#
  ###.###########.###.#####.#.#.#
  #.....#        A   C    #.#.#.#
  #######        S   P    #####.#
  #.#...#                 #......VT
  #.#.#.#                 #.#####
  #...#.#               YN....#.#
  #.###.#                 #####.#
DI....#.#                 #.....#
  #####.#                 #.###.#
ZZ......#               QG....#..AS
  ###.###                 #######
JO..#.#.#                 #.....#
  #.#.#.#                 ###.#.#
  #...#..DI             BU....#..LF
  #####.#                 #.#####
YN......#               VT..#....QG
  #.###.#                 #.###.#
  #.#...#                 #.....#
  ###.###    J L     J    #.#.###
  #.....#    O F     P    #.#...#
  #.###.#####.#.#####.#####.###.#
  #...#.#.#...#.....#.....#.#...#
  #.#####.###.###.#.#.#########.#
  #...#.#.....#...#.#.#.#.....#.#
  #.###.#####.###.###.#.#.#######
  #.#.........#...#.............#
  #########.###.###.#############
           B   J   C
           U   P   P"
                .to_string(),
        );

        assert_eq!(part_one(map), 58);
    }

    #[test]
    fn test_part_two_simple_map() {
        let map = Map::from(
            "         A
         A
  #######.#########
  #######.........#
  #######.#######.#
  #######.#######.#
  #######.#######.#
  #####  B    ###.#
BC...##  C    ###.#
  ##.##       ###.#
  ##...DE  F  ###.#
  #####    G  ###.#
  #########.#####.#
DE..#######...###.#
  #.#########.###.#
FG..#########.....#
  ###########.#####
             Z
             Z"
            .to_string(),
        );

        assert_eq!(part_two(map), 26);
    }

    #[test]
    fn test_part_two_complex_map() {
        let map = Map::from(
            "             Z L X W       C
             Z P Q B       K
  ###########.#.#.#.#######.###############
  #...#.......#.#.......#.#.......#.#.#...#
  ###.#.#.#.#.#.#.#.###.#.#.#######.#.#.###
  #.#...#.#.#...#.#.#...#...#...#.#.......#
  #.###.#######.###.###.#.###.###.#.#######
  #...#.......#.#...#...#.............#...#
  #.#########.#######.#.#######.#######.###
  #...#.#    F       R I       Z    #.#.#.#
  #.###.#    D       E C       H    #.#.#.#
  #.#...#                           #...#.#
  #.###.#                           #.###.#
  #.#....OA                       WB..#.#..ZH
  #.###.#                           #.#.#.#
CJ......#                           #.....#
  #######                           #######
  #.#....CK                         #......IC
  #.###.#                           #.###.#
  #.....#                           #...#.#
  ###.###                           #.#.#.#
XF....#.#                         RF..#.#.#
  #####.#                           #######
  #......CJ                       NM..#...#
  ###.#.#                           #.###.#
RE....#.#                           #......RF
  ###.###        X   X       L      #.#.#.#
  #.....#        F   Q       P      #.#.#.#
  ###.###########.###.#######.#########.###
  #.....#...#.....#.......#...#.....#.#...#
  #####.#.###.#######.#######.###.###.#.#.#
  #.......#.......#.#.#.#.#...#...#...#.#.#
  #####.###.#####.#.#.#.#.###.###.#.###.###
  #.......#.....#.#...#...............#...#
  #############.#.#.###.###################
               A O F   N
               A A D   M"
                .to_string(),
        );

        assert_eq!(part_two(map), 396);
    }
}
