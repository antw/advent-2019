use std::collections::VecDeque;
use std::io;

extern crate intcode;
use intcode::{Program, ProgramState};

struct Switch {
    programs: Vec<Program>,
}

impl Switch {
    fn new() -> Switch {
        Switch {
            programs: Vec::new(),
        }
    }

    /// Adds a new program to the switch. Returns the address of the program.
    fn push(&mut self, program: Program) -> usize {
        self.programs.push(program);
        self.programs.len() - 1
    }

    fn run(&mut self, part_one: bool) -> i64 {
        // Store the outputs from each program. Once a program has two outputs stored we send to
        // the receiving program.
        let mut outputs = vec![VecDeque::with_capacity(2); self.programs.len()];

        // Can't send inputs directly to the receiver as this requires two mutable references. Store
        // the inputs in a queue and sent immediately prior to running the program.
        let mut inputs = vec![VecDeque::new(); self.programs.len()];

        let mut nat = NAT::new(self.programs.len());

        // With my input, the Y values sent by the NAT to the first program area a series of
        // decreasing numbers. Therefore, to find the first duplicate, its good enough to store the
        // last number sent and compare. This may not be true for all inputs.
        let mut last_nat_send = None;

        loop {
            for (index, program) in self.programs.iter_mut().enumerate() {
                for input in inputs.get_mut(index).unwrap().drain(0..) {
                    program.push_input(input);
                }

                loop {
                    match program.run() {
                        ProgramState::Output(value) => {
                            let program_outputs = outputs.get_mut(index).unwrap();

                            match program_outputs.len() {
                                0 | 1 => program_outputs.push_back(value),
                                2 => {
                                    // We have enough values to send to a receiver.
                                    let receiver_id = program_outputs.pop_front().unwrap() as usize;
                                    let x = program_outputs.pop_front().unwrap();

                                    if receiver_id == 255 {
                                        if part_one {
                                            return value;
                                        }

                                        nat.receive(x, value);
                                    } else {
                                        let receiver = inputs.get_mut(receiver_id).unwrap();

                                        receiver.push_back(x);
                                        receiver.push_back(value);

                                        nat.ready(receiver_id);
                                    }
                                }
                                _ => panic!(
                                    "Unexpected program outputs length: {}",
                                    program_outputs.len()
                                ),
                            }
                        }
                        ProgramState::Wait => {
                            nat.waiting(index);

                            if nat.is_stalled() {
                                // Sent the NATs last packet to program 0.
                                if let Some((x, y)) = nat.last_packet {
                                    let receiver = inputs.get_mut(0).unwrap();

                                    receiver.push_back(x);
                                    receiver.push_back(y);

                                    if let Some(previous) = last_nat_send {
                                        if previous == y {
                                            return y;
                                        }
                                    }

                                    last_nat_send = Some(y);

                                    nat.last_packet = None;
                                    nat.ready(0);
                                }
                            }

                            // Move on to the next program.
                            break;
                        },
                        ProgramState::Halt => return -1,
                    }
                }
            }
        }
    }
}

struct NAT {
    last_packet: Option<(i64, i64)>,
    waiting: Vec<bool>,
}

impl NAT {
    fn new(capacity: usize) -> NAT {
        NAT { last_packet: None, waiting: vec![false; capacity] }
    }

    fn receive(&mut self, x: i64, y: i64) {
        self.last_packet = Some((x, y));
    }

    /// Informs the NAT that the program at address `n` it waiting for an input.
    fn waiting(&mut self, n: usize) {
        self.waiting[n] = true;
    }

    /// Returns whether the network has stalled with all programs waiting for an input.
    fn is_stalled(&self) -> bool {
        self.waiting.iter().filter(|&&wait| wait).count() == self.waiting.len()
    }

    fn ready(&mut self, n: usize) {
        self.waiting[n] = false;
    }
}

fn initialize_switch(intcodes: Vec<i64>) -> Switch {
    let mut switch = Switch::new();

    for id in 0..50 {
        let mut program = Program::new(intcodes.clone());

        program.push_input(id);
        program.push_input(-1);

        switch.push(program);
    }

    switch
}

fn part_one(intcodes: Vec<i64>) -> i64 {
    initialize_switch(intcodes).run(true)
}

fn part_two(intcodes: Vec<i64>) -> i64 {
    initialize_switch(intcodes).run(false)
}

fn main() -> Result<(), io::Error> {
    let intcodes = intcode::load_intcodes_from_file("data/intcodes.txt")?;

    println!("Part one: {}", part_one(intcodes.clone()));
    println!("Part two: {}", part_two(intcodes));

    Ok(())
}
