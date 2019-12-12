use std::collections::HashSet;
use std::ops::Index;

#[derive(Debug, PartialEq, Eq)]
struct Position {
    x: i64,
    y: i64,
    z: i64,
}

impl Index<usize> for Position {
    type Output = i64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Unknown index: {}", index),
        }
    }
}

impl Position {
    fn new(x: i64, y: i64, z: i64) -> Position {
        Position { x, y, z }
    }
    fn add(&self, other: &Position) -> Position {
        Position::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

struct Moon {
    position: Position,
    velocity: Position,
}

impl Moon {
    fn new(x: i64, y: i64, z: i64) -> Moon {
        Moon {
            position: Position::new(x, y, z),
            velocity: Position::new(0, 0, 0),
        }
    }

    fn pot(&self) -> i64 {
        self.position.x.abs() + self.position.y.abs() + self.position.z.abs()
    }

    fn kin(&self) -> i64 {
        self.velocity.x.abs() + self.velocity.y.abs() + self.velocity.z.abs()
    }

    fn energy(&self) -> i64 {
        self.pot() * self.kin()
    }
}

fn apply_gravity(moons: &mut Vec<Moon>) {
    for i in 0..moons.len() {
        for j in i + 1..moons.len() {
            // There has to be a better way of doing this.
            if moons[i].position.x > moons[j].position.x {
                moons[i].velocity.x -= 1;
                moons[j].velocity.x += 1;
            } else if moons[i].position.x < moons[j].position.x {
                moons[i].velocity.x += 1;
                moons[j].velocity.x -= 1;
            }

            if moons[i].position.y > moons[j].position.y {
                moons[i].velocity.y -= 1;
                moons[j].velocity.y += 1;
            } else if moons[i].position.y < moons[j].position.y {
                moons[i].velocity.y += 1;
                moons[j].velocity.y -= 1;
            }

            if moons[i].position.z > moons[j].position.z {
                moons[i].velocity.z -= 1;
                moons[j].velocity.z += 1;
            } else if moons[i].position.z < moons[j].position.z {
                moons[i].velocity.z += 1;
                moons[j].velocity.z -= 1;
            }
        }
    }
}

fn apply_velocity(moons: &mut Vec<Moon>) {
    for moon in moons.iter_mut() {
        moon.position = moon.position.add(&moon.velocity);
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    let mut a = a;
    let mut b = b;

    while b > 0 {
        let m = a % b;
        a = b;
        b = m;
    }

    a
}

fn lcm(a: i64, b: i64) -> i64 {
    a * b / gcd(a, b)
}

struct AxisPositionCache {
    attr_num: usize,
    seen: HashSet<Vec<(i64, i64)>>,
    value: Option<usize>,
}

impl AxisPositionCache {
    fn new(attr_num: usize) -> AxisPositionCache {
        AxisPositionCache {
            attr_num,
            seen: HashSet::new(),
            value: None,
        }
    }

    fn has_value(&self) -> bool {
        self.value.is_some()
    }

    fn key_for(&self, moons: &Vec<Moon>) -> Vec<(i64, i64)> {
        moons
            .iter()
            .map(|moon| (moon.position[self.attr_num], moon.velocity[self.attr_num]))
            .collect::<Vec<(i64, i64)>>()
    }

    /// Inserts the given key into the cache unless the cache already has a value. If the key has
    /// been encountered previously, the cache value is set.
    fn seen_or_insert(&mut self, iteration: usize, key: Vec<(i64, i64)>) {
        if self.seen.contains(&key) {
            self.value = Some(iteration);
        } else {
            self.seen.insert(key);
        }
    }
}

/// Keeps track of previously seen positions and velocities for each axis and once repeats have been
/// found for all three, calculates the lowest common multiple of all three.
///
/// This is a little computationally wasteful since it continues computing axes until a value for
/// all three has been found. In my input, the first repeat for the Y axis is in step 96236, while
/// the repeat for X is 231614: this means that 135,378 further Y axis positions and velocities are
/// calculated by apply_gravity and apply_velocity, even though they aren't needed.
fn part_two(moons: &mut Vec<Moon>) -> i64 {
    let mut step = 0;

    let mut x = AxisPositionCache::new(0);
    let mut y = AxisPositionCache::new(1);
    let mut z = AxisPositionCache::new(2);

    loop {
        if !x.has_value() {
            x.seen_or_insert(step, x.key_for(moons));
        }

        if !y.has_value() {
            y.seen_or_insert(step, y.key_for(moons));
        }

        if !z.has_value() {
            z.seen_or_insert(step, z.key_for(moons));
        }

        if x.has_value() && y.has_value() && z.has_value() {
            return lcm(
                x.value.unwrap() as i64,
                lcm(y.value.unwrap() as i64, z.value.unwrap() as i64),
            );
        }

        apply_gravity(moons);
        apply_velocity(moons);

        step += 1;
    }
}

fn main() {
    let mut moons = vec![
        Moon::new(-1, 7, 3),
        Moon::new(12, 2, -13),
        Moon::new(14, 18, -8),
        Moon::new(17, 4, -4),
    ];

    for _ in 0..1000 {
        apply_gravity(&mut moons);
        apply_velocity(&mut moons);
    }

    let energy = moons.iter().fold(0, |memo, moon| memo + moon.energy());

    println!("Part one: {}", energy);

    let mut moons = vec![
        Moon::new(-1, 7, 3),
        Moon::new(12, 2, -13),
        Moon::new(14, 18, -8),
        Moon::new(17, 4, -4),
    ];

    println!("Part two: {}", part_two(&mut moons));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_move() {
        let one = Position { x: -1, y: 0, z: 2 };

        let two = Position {
            x: 2,
            y: -10,
            z: -7,
        };

        let moved = one.add(&two);

        assert_eq!(moved.x, 1);
        assert_eq!(moved.y, -10);
        assert_eq!(moved.z, -5);
    }

    #[test]
    fn test_part_one_example() {
        let mut moons = vec![
            Moon::new(-1, 0, 2),
            Moon::new(2, -10, -7),
            Moon::new(4, -8, 8),
            Moon::new(3, 5, -1),
        ];

        apply_gravity(&mut moons);
        apply_velocity(&mut moons);

        assert_eq!(moons[0].velocity, Position::new(3, -1, -1));
        assert_eq!(moons[1].velocity, Position::new(1, 3, 3));
        assert_eq!(moons[2].velocity, Position::new(-3, 1, -3));
        assert_eq!(moons[3].velocity, Position::new(-1, -3, 1));

        assert_eq!(moons[0].position, Position::new(2, -1, 1));
        assert_eq!(moons[1].position, Position::new(3, -7, -4));
        assert_eq!(moons[2].position, Position::new(1, -7, 5));
        assert_eq!(moons[3].position, Position::new(2, 2, 0));

        apply_gravity(&mut moons);
        apply_velocity(&mut moons);

        assert_eq!(moons[0].velocity, Position::new(3, -2, -2));
        assert_eq!(moons[1].velocity, Position::new(-2, 5, 6));
        assert_eq!(moons[2].velocity, Position::new(0, 3, -6));
        assert_eq!(moons[3].velocity, Position::new(-1, -6, 2));

        assert_eq!(moons[0].position, Position::new(5, -3, -1));
        assert_eq!(moons[1].position, Position::new(1, -2, 2));
        assert_eq!(moons[2].position, Position::new(1, -4, -1));
        assert_eq!(moons[3].position, Position::new(1, -4, 2));

        for _ in 0..8 {
            apply_gravity(&mut moons);
            apply_velocity(&mut moons);
        }

        assert_eq!(moons[0].velocity, Position::new(-3, -2, 1));
        assert_eq!(moons[1].velocity, Position::new(-1, 1, 3));
        assert_eq!(moons[2].velocity, Position::new(3, 2, -3));
        assert_eq!(moons[3].velocity, Position::new(1, -1, -1));

        assert_eq!(moons[0].position, Position::new(2, 1, -3));
        assert_eq!(moons[1].position, Position::new(1, -8, 0));
        assert_eq!(moons[2].position, Position::new(3, -6, 1));
        assert_eq!(moons[3].position, Position::new(2, 0, 4));
    }

    #[test]
    fn test_part_two_example() {
        let mut moons = vec![
            Moon::new(-1, 0, 2),
            Moon::new(2, -10, -7),
            Moon::new(4, -8, 8),
            Moon::new(3, 5, -1),
        ];

        assert_eq!(part_two(&mut moons), 2772);
    }

    #[test]
    fn test_moon_potential_energy() {
        let moon = Moon::new(2, 1, 3);
        assert_eq!(moon.pot(), 6);
    }

    #[test]
    fn test_moon_kinetic_energy() {
        let mut moon = Moon::new(1, 8, 0);
        moon.velocity = Position::new(1, 1, 3);

        assert_eq!(moon.kin(), 5);
    }

    #[test]
    fn test_moon_energy() {
        let mut moon = Moon::new(1, 8, 0);
        moon.velocity = Position::new(1, 1, 3);

        assert_eq!(moon.energy(), 45);
    }
}
