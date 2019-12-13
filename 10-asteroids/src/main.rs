/// Oh god, don't look at it!
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fs;
use std::iter::FromIterator;

/// Described a position on the map, occupied by an asteroid.
#[derive(Debug, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn angle(&self, other: &Point) -> f64 {
        let x_distance = self.x - other.x;
        let y_distance = self.y - other.y;
        let angle = x_distance.atan2(y_distance) * 180.0 / std::f64::consts::PI;

        if angle < 0.0 {
            angle + 360.0
        } else {
            angle
        }
    }
}

/// Stores an target point, and memoizes both the angle and distance from an origin to the target.
struct Ray<'a> {
    target: &'a Point,
    angle: f64,
    distance: f64,
}

impl<'a> Ray<'a> {
    fn new(origin: &'a Point, target: &'a Point) -> Ray<'a> {
        let angle = origin.angle(&target);

        Ray {
            target,
            // I have to sort by negative angle, ensuring that those directly north come first
            // (hence -360). I'm not entirely sure why; probably messed something up in
            // Point::angle?
            angle: if angle == 0.0 { -360.0 } else { -angle },
            distance: origin.distance(&target),
        }
    }
}

/// Given a path to an asteroid map, reads the map and returns a vector containing all the Points
/// where an asteroid is located.
fn build_map(data: &str) -> Vec<Point> {
    let mut asteroids = Vec::new();

    for (y, line) in data.lines().enumerate() {
        for (x, character) in line.chars().enumerate() {
            if character == '#' {
                asteroids.push(Point {
                    x: x as f64,
                    y: y as f64,
                })
            }
        }
    }

    asteroids
}

/// Given a list of asteroid positions, and an origin asteroid, calculates the angle from the origin
/// to all the asteroids (except the origin) in the list.
fn visible_from_location<'a>(asteroids: &'a Vec<Point>, origin: &'a Point) -> Vec<Ray<'a>> {
    asteroids
        .iter()
        .filter(|asteroid| *asteroid != origin)
        .map(|point| Ray::new(origin, point))
        .collect::<Vec<Ray>>()
}

fn part_one<'a>(asteroids: &'a Vec<Point>) -> (&'a Point, usize) {
    let mut max = 0;
    let mut best = &asteroids[0];

    for asteroid in asteroids {
        let mut angles = visible_from_location(&asteroids, &asteroid);

        // Have to sort in order for dedup_by_key to remove all duplicates.
        angles.sort_by(|left, right| {
            left.angle
                .partial_cmp(&right.angle)
                .unwrap_or(Ordering::Equal)
        });

        angles.dedup_by_key(|angle| angle.angle);

        if angles.len() > max {
            max = angles.len();
            best = asteroid;
        }
    }

    (best, max)
}

fn part_two(asteroids: &Vec<Point>, station: &Point, bet: usize) -> Option<f64> {
    let mut angles = visible_from_location(asteroids, station);

    // Sort first by distance...
    angles.sort_unstable_by(|left, right| {
        left.distance
            .partial_cmp(&right.distance)
            .unwrap_or(Ordering::Equal)
    });

    // ...then by angle. This gives us a vector where asteroids are sorted first by angle, and when
    // any have the same angle, then by distance.
    angles.sort_by(|left, right| {
        left.angle
            .partial_cmp(&right.angle)
            .unwrap_or(Ordering::Equal)
    });

    let mut angles = VecDeque::from_iter(&angles);
    let mut count = 0;

    while let Some(asteroid) = angles.pop_front() {
        // The first asteroid popped off is always a new angle.
        count += 1;

        if count == bet {
            return Some(asteroid.target.x * 100.0 + asteroid.target.y);
        }

        while let Some(next) = angles.front() {
            if next.angle == asteroid.angle {
                // Rotate any other asteroids with the same angle to the back of the queue. This
                // would be better to find the index of the first entry with a different angle, and
                // rotate all at once.
                angles.rotate_left(1);
            } else {
                break;
            }
        }
    }

    None
}

fn main() -> Result<(), std::io::Error> {
    let data = fs::read_to_string("data/asteroids.txt")?;
    let data = data.trim();

    let map = build_map(data);

    let (station, asteroids_visible) = part_one(&map);

    println!("Part one: {:?}", asteroids_visible);
    println!("Part two: {:?}", part_two(&map, station, 200));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn trim_leading_whitespace(string: &str) -> String {
        let lines: Vec<&str> = string.lines().map(|line| line.trim()).collect();
        lines.join("\n")
    }

    #[test]
    fn test_part_one() {
        let map = trim_leading_whitespace(
            "......#.#.
             #..#.#....
             ..#######.
             .#.#.###..
             .#..#.....
             ..#....#.#
             #..#....#.
             .##.#..###
             ##...#..#.
             .#....####",
        );

        let map = build_map(&map);
        let (station, visible) = part_one(&map);

        assert_eq!(visible, 33);
        assert_eq!(station, &Point { x: 5.0, y: 8.0 });

        let map = trim_leading_whitespace(
            "#.#...#.#.
             .###....#.
             .#....#...
             ##.#.#.#.#
             ....#.#.#.
             .##..###.#
             ..#...##..
             ..##....##
             ......#...
             .####.###.",
        );

        let map = build_map(&map);
        let (station, visible) = part_one(&map);

        assert_eq!(visible, 35);
        assert_eq!(station, &Point { x: 1.0, y: 2.0 });

        let map = trim_leading_whitespace(
            ".#..#..###
             ####.###.#
             ....###.#.
             ..###.##.#
             ##.##.#.#.
             ....###..#
             ..#.#..#.#
             #..#.#.###
             .##...##.#
             .....#.#..",
        );

        let map = build_map(&map);
        let (station, visible) = part_one(&map);

        assert_eq!(visible, 41);
        assert_eq!(station, &Point { x: 6.0, y: 3.0 });

        let map = trim_leading_whitespace(
            ".#..##.###...#######
             ##.############..##.
             .#.######.########.#
             .###.#######.####.#.
             #####.##.#.##.###.##
             ..#####..#.#########
             ####################
             #.####....###.#.#.##
             ##.#################
             #####.##.###..####..
             ..######..##.#######
             ####.##.####...##..#
             .#####..#.######.###
             ##...#.##########...
             #.##########.#######
             .####.#.###.###.#.##
             ....##.##.###..#####
             .#.#.###########.###
             #.#.#.#####.####.###
             ###.##.####.##.#..##",
        );

        let map = build_map(&map);
        let (station, visible) = part_one(&map);

        assert_eq!(visible, 210);
        assert_eq!(station, &Point { x: 11.0, y: 13.0 });
    }

    #[test]
    fn test_part_two() {
        let map = trim_leading_whitespace(
            ".#..##.###...#######
             ##.############..##.
             .#.######.########.#
             .###.#######.####.#.
             #####.##.#.##.###.##
             ..#####..#.#########
             ####################
             #.####....###.#.#.##
             ##.#################
             #####.##.###..####..
             ..######..##.#######
             ####.##.####...##..#
             .#####..#.######.###
             ##...#.##########...
             #.##########.#######
             .####.#.###.###.#.##
             ....##.##.###..#####
             .#.#.###########.###
             #.#.#.#####.####.###
             ###.##.####.##.#..##",
        );

        let map = build_map(&map);
        let answer = part_two(&map, &Point { x: 11.0, y: 13.0 }, 200);

        assert_eq!(answer, Some(802.0));
    }
}
