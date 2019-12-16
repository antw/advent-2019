#![feature(repeat_generic_slice)]

use std::fs;
use std::io;

/// Takes a vector and cycles through each element in turn. Once it reaches the end of the vector it
/// yields the first value again, and so on. A `repeat` may be provided; each element in the vector
/// will be repeated this many times before proceeding to the next.
struct RepeatingCycleIterator {
    values: Vec<i32>,
    repeat: usize,
    current_repeat: usize,
    iterations: usize,
}

impl RepeatingCycleIterator {
    fn new(values: Vec<i32>, repeat: usize) -> RepeatingCycleIterator {
        assert!(repeat > 0);

        RepeatingCycleIterator {
            current_repeat: 1,
            iterations: 0,
            repeat,
            values,
        }
    }
}

impl Iterator for RepeatingCycleIterator {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self
            .values
            .get(self.iterations % self.values.len())
            .unwrap();

        if self.current_repeat == self.repeat {
            self.current_repeat = 1;
            self.iterations += 1;
        } else {
            self.current_repeat += 1;
        }

        Some(*value)
    }
}

fn flawed_frequency_transmission(transmission: Vec<i32>, iterations: usize) -> Vec<i32> {
    let mut transmission = transmission;
    let base_pattern = vec![0, 1, 0, -1];

    for _ in 0..iterations {
        let mut calculated = Vec::with_capacity(transmission.len());

        // TODO: This can probably be done cleaner with a Zip, but then we'd have to have the
        //       pattern iterator automatically skip the first value.

        for i in 0..transmission.len() {
            let mut sum = 0;
            let mut pattern = RepeatingCycleIterator::new(base_pattern.clone(), i + 1);

            // Discard the first pattern value.
            pattern.next();

            for value in &transmission {
                sum += value * pattern.next().unwrap();
            }

            calculated.push(sum.abs() % 10);
        }

        transmission = calculated;
    }

    transmission
}

/// Cheats by assuming that the repeating pattern is always 1 for the digits in the transmission
/// which we need to sum.
fn flawed_frequency_transmission_with_offset(
    transmission: Vec<i32>,
    iterations: usize,
    offset: usize,
) -> Vec<i32> {
    let len = transmission.len() - offset - 1;

    let mut transmission = transmission
        .into_iter()
        .cycle()
        .skip(offset)
        .take(len)
        .collect::<Vec<i32>>();

    for _ in 0..iterations {
        for i in (0..len - 1).rev() {
            transmission[i] = (transmission[i] + transmission[i + 1]).abs() % 10;
        }
    }

    transmission[0..8].to_vec()
}

/// Reads the transmission file at the `path`.
fn read_transmission(path: &str) -> Result<Vec<i32>, io::Error> {
    Ok(fs::read_to_string(path)?
        .trim()
        .chars()
        .map(|character| character as i32 - 48)
        .collect::<Vec<i32>>())
}

fn vec_to_number(numbers: Vec<i32>) -> i32 {
    let mut number = 0;
    let mut base = 1;
    let mut numbers = numbers;

    numbers.reverse();

    for num in numbers {
        number += num * base;
        base = base * 10;
    }

    number
}

fn part_one(transmission: Vec<i32>) -> String {
    flawed_frequency_transmission(transmission, 100)
        .into_iter()
        .take(8)
        .map(|num| format!("{}", num))
        .collect::<String>()
}

/// While part one was my own work (and quite easy), part two is heavily inspired by others'
/// solutions. I'm still not really sure I understand the optimization here...
fn part_two(transmission: Vec<i32>) -> String {
    let offset = vec_to_number(
        transmission
            .iter()
            .take(7)
            .map(|&val| val)
            .collect::<Vec<i32>>(),
    ) as usize;

    let real_signal =
        flawed_frequency_transmission_with_offset(transmission.repeat(10_000), 100, offset);

    real_signal
        .into_iter()
        .map(|num| format!("{}", num))
        .collect::<String>()
}

fn main() -> Result<(), io::Error> {
    let transmission = read_transmission("data/transmission.txt")?;

    println!("Part one: {}", part_one(transmission.clone()));
    println!("Part two: {}", part_two(transmission));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_iterator_repeat_once() {
        let iter = RepeatingCycleIterator::new(vec![1, 2, 3], 1);

        assert_eq!(
            iter.take(7).collect::<Vec<i32>>(),
            vec![1, 2, 3, 1, 2, 3, 1],
        );
    }

    #[test]
    fn test_cycle_iterator_repeat_twice() {
        let iter = RepeatingCycleIterator::new(vec![1, 2, 3], 2);

        assert_eq!(
            iter.take(16).collect::<Vec<i32>>(),
            vec![1, 1, 2, 2, 3, 3, 1, 1, 2, 2, 3, 3, 1, 1, 2, 2],
        );
    }

    #[test]
    fn test_cycle_iterator_repeat_thrice() {
        let iter = RepeatingCycleIterator::new(vec![1, 2, 3], 3);

        assert_eq!(
            iter.take(10).collect::<Vec<i32>>(),
            vec![1, 1, 1, 2, 2, 2, 3, 3, 3, 1],
        );
    }

    #[test]
    fn test_vec_to_number() {
        assert_eq!(vec_to_number(vec![1, 2, 3, 4,]), 1234);
        assert_eq!(vec_to_number(vec![1, 3, 1, 4, 6]), 13146);
        assert_eq!(vec_to_number(vec![0, 3, 0, 4, 6]), 3046);
    }

    #[test]
    fn test_basic_signal() {
        let result = flawed_frequency_transmission(vec![1, 2, 3, 4, 5, 6, 7, 8], 1);
        assert_eq!(result, vec![4, 8, 2, 2, 6, 1, 5, 8]);

        let result = flawed_frequency_transmission(vec![1, 2, 3, 4, 5, 6, 7, 8], 2);
        assert_eq!(result, vec![3, 4, 0, 4, 0, 4, 3, 8]);

        let result = flawed_frequency_transmission(vec![1, 2, 3, 4, 5, 6, 7, 8], 4);
        assert_eq!(result, vec![0, 1, 0, 2, 9, 4, 9, 8]);
    }

    #[test]
    fn test_signal() {
        let result = flawed_frequency_transmission(
            vec![
                8, 0, 8, 7, 1, 2, 2, 4, 5, 8, 5, 9, 1, 4, 5, 4, 6, 6, 1, 9, 0, 8, 3, 2, 1, 8, 6, 4,
                5, 5, 9, 5,
            ],
            100,
        );

        assert_eq!(
            result.into_iter().take(8).collect::<Vec<i32>>(),
            vec![2, 4, 1, 7, 6, 1, 7, 6],
        );

        let result = flawed_frequency_transmission(
            vec![
                1, 9, 6, 1, 7, 8, 0, 4, 2, 0, 7, 2, 0, 2, 2, 0, 9, 1, 4, 4, 9, 1, 6, 0, 4, 4, 1, 8,
                9, 9, 1, 7,
            ],
            100,
        );

        assert_eq!(
            result.into_iter().take(8).collect::<Vec<i32>>(),
            vec![7, 3, 7, 4, 5, 4, 1, 8]
        );

        let result = flawed_frequency_transmission(
            vec![
                6, 9, 3, 1, 7, 1, 6, 3, 4, 9, 2, 9, 4, 8, 6, 0, 6, 3, 3, 5, 9, 9, 5, 9, 2, 4, 3, 1,
                9, 8, 7, 3,
            ],
            100,
        );

        assert_eq!(
            result.into_iter().take(8).collect::<Vec<i32>>(),
            vec![5, 2, 4, 3, 2, 1, 3, 3]
        );
    }

    #[test]
    fn test_real_signal() {
        let result = part_two(vec![
            0, 3, 0, 3, 6, 7, 3, 2, 5, 7, 7, 2, 1, 2, 9, 4, 4, 0, 6, 3, 4, 9, 1, 5, 6, 5, 4, 7, 4,
            6, 6, 4,
        ]);

        assert_eq!(result, "84462026".to_string());
    }
}
