use std::io;

extern crate intcode;
use intcode::{Program, ProgramState};

fn run_springdroid(program: Program, instructions: Vec<&str>) {
    let mut program = program;

    for instruction in instructions {
        for character in instruction.chars() {
            program.push_input(character as u8 as i64);
        }
    }

    while let ProgramState::Output(output) = program.run() {
        if output < 255 {
            print!("{}", output as u8 as char);
        } else {
            println!("{}", output);
        }
    }
}

fn part_one(program: Program) {
    // This is practically identical to the "jump is A, B, and C are empty" example from AoC, except
    // that is jumps if any of A, B, or C are empty and D is not.
    run_springdroid(
        program,
        vec![
            "NOT A J\n", // J = !A (A = no ground)
            "NOT B T\n", // T = !B (B = no ground)
            "OR T J\n",  // J = !A || !B (A or B = no ground)
            "NOT C T\n", // T = !C (C = no ground)
            "OR T J\n",  // J = !A || !B || !C (A or B or C = no ground)
            "AND D J\n", // J = (!A || !B || !C) && D (A or B or C = no ground, D = ground)
            "WALK\n",
        ],
    );
}

fn part_two(program: Program) {
    run_springdroid(
        program,
        // Asserts that either E or H have ground, preventing the droid from jumping too soon.
        //
        // @ = droid position, v = potential jump landing site
        //
        // .................    .................    .................    .................
        // .................    .................    .................    .................
        // @...v............    .@...v...........    ..@...v...v......    ...@...v.........
        // #####.#.#...#.###    #####.#.#...#.###    #####.#.#...#.###    #####.#.#...#.###
        //  ABCDEFGHI             ABCDEFGHI             ABCDEFGHI             ABCDEFGHI
        //
        // .................    .................    .................    .................
        // .................    .................    .................    .................
        // ....@...v........    ........@...v....    ............@...v    ................@
        // #####.#.#...#.###    #####.#.#...#.###    #####.#.#...#.###    #####.#.#...#.###
        //      ABCDEFGHI                ABCDEFGH                 ABCD
        vec![
            "NOT A J\n", // J = !A (A = no ground)
            "NOT B T\n", // T = !B (B = no ground)
            "OR T J\n",  // J = !A || !B (A or B = no ground)
            "NOT C T\n", // T = !C (C = no ground)
            "OR T J\n",  // J = !A || !B || !C (A or B or C = no ground)
            "AND D J\n", // J = (!A || !B || !C) && D (A or B or C = no ground, D = ground)
            //
            "NOT E T\n", // T = !E (E = no ground)
            "NOT T T\n", // T = E (E = ground)
            "OR H T\n",  // T = E || H (E or H = ground)
            "AND T J\n", // J = (!A || !B || !C) && D && (E || H)
            //
            "RUN\n",
        ],
    );
}

fn main() -> Result<(), io::Error> {
    let program = Program::from_file("data/intcodes.txt")?;
    part_one(program);

    let program = Program::from_file("data/intcodes.txt")?;
    part_two(program);

    Ok(())
}
