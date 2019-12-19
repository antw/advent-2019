use std::io;

extern crate intcode;
use intcode::{Program, ProgramState};

fn part_one(intcodes: Vec<i64>) -> usize {
    let mut beam = 0;

    for y in 0..50 {
        for x in 0..50 {
            if is_inside_beam(intcodes.clone(), x, y) {
                beam += 1;
            }
        }
    }

    beam
}

fn is_inside_beam(intcodes: Vec<i64>, x: i64, y: i64) -> bool {
    let mut program = Program::new(intcodes);

    program.push_input(x);
    program.push_input(y);

    match program.run() {
        ProgramState::Output(0) => false,
        ProgramState::Output(1) => true,
        _ => unreachable!(),
    }
}

fn part_two(intcodes: Vec<i64>) -> i64 {
    // The beam spreads out (somewhat) diagonally. If we're not in the beam at a particular point
    // then we're not far enough to the right.
    let mut x = 0;

    // The beam isn't wide enough in the first 100 positions.
    let mut y = 100;

    loop {
        // Check (hopefully) the bottom left position of the tractor beam.
        if is_inside_beam(intcodes.clone(), x, y) {
            // Check 100 positions to the right and 100 positions up.
            if is_inside_beam(intcodes.clone(), x + 99, y - 99) {
                return x * 10000 + y - 99;
            }
        } else {
            x += 1;
        }

        y += 1;
    }
}

fn main() -> Result<(), io::Error> {
    let intcodes = intcode::load_intcodes_from_file("data/intcodes.txt")?;

    println!("Part one: {}", part_one(intcodes.clone()));
    println!("Part two: {}", part_two(intcodes));

    Ok(())
}
