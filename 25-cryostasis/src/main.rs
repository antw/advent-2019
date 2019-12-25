/// Day 25 was completed manually with pen-and-paper.
///
/// The solution requires that you Take:
///
///   * easter egg - in the kitchen
///   * mutex - in the holodeck
///   * astronaut ice cream - hot chocolate fountain
///   * tambourine - engineering
///
/// 1. Move south then east to the holodeck. Take the mutex.
/// 2. Move east to the hot chocolate fountain. Take the astronaut ice cream.
/// 3. Move  south. Take the tambourine.
/// 4. Move north, west, south, south, west, and south to the kitcen. Take the easter egg.
/// 5. Move west to the security checkpoint.
use std::io::{self, BufRead};

extern crate intcode;
use intcode::{Program, ProgramState};

fn main() -> Result<(), io::Error> {
    let mut program = Program::from_file("data/intcodes.txt")?;
    let stdin = io::stdin();

    loop {
        match program.run() {
            ProgramState::Output(output) => {
                print!("{}", output as u8 as char);
            }
            ProgramState::Wait => {
                let mut iter = stdin.lock().lines();
                let input = iter.next().unwrap().unwrap();

                for character in input.bytes() {
                    program.push_input(character as i64);
                }

                program.push_input('\n' as i64);
            }
            ProgramState::Halt => break,
        }
    }

    Ok(())
}
