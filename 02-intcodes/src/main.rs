use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process;

fn read_intcodes(path: &str) -> Vec<usize> {
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    let mut first_line = String::new();
    reader.read_line(&mut first_line).unwrap();

    first_line
        .trim()
        .split(",")
        .map(|intcode| intcode.parse::<usize>().unwrap())
        .collect()
}

fn positions(intcodes: &Vec<usize>, position: usize) -> (usize, usize, usize) {
    let lop_pos = intcodes[position + 1];
    let rop_pos = intcodes[position + 2];
    let out_pos = intcodes[position + 3];

    (lop_pos, rop_pos, out_pos)
}

fn run_program(intcodes: Vec<usize>) -> Vec<usize> {
    let mut intcodes = intcodes;
    let mut position = 0;

    while position < intcodes.len() {
        let code = intcodes[position];

        match code {
            1 => {
                let (left, right, out) = positions(&intcodes, position);
                intcodes[out] = intcodes[left] + intcodes[right];

                position += 4;
            }
            2 => {
                let (left, right, out) = positions(&intcodes, position);
                intcodes[out] = intcodes[left] * intcodes[right];

                position += 4;
            }
            99 => break,
            anything => {
                eprintln!("Unknown intcode: {}", anything);
                process::exit(1);
            }
        }
    }

    intcodes
}

fn main() {
    let mut intcodes = read_intcodes("intcodes.txt");

    for noun in 0..100 {
        for verb in 0..100 {
            let mut intcodes = intcodes.clone();

            intcodes[1] = noun;
            intcodes[2] = verb;

            let result = run_program(intcodes);

            if result[0] == 19690720 {
                println!("Noun: {} Verb: {}", noun, verb);
                break;
            }
        }
    }
}
