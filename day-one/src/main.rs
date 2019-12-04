use std::fs::File;
use std::io::{BufRead, BufReader};

fn calculate_fuel(mass: f64) -> f64 {
    let own_fuel = (mass / 3.0).floor() - 2.0;

    if own_fuel < 0.0 {
        return 0.0;
    }

    return own_fuel + calculate_fuel(own_fuel);
}

// https://riptutorial.com/rust/example/4275/read-a-file-line-by-line
fn main() {
    // Open the file in read-only mode, ignoring errors.
    let file = File::open("masses.txt").unwrap();
    let reader = BufReader::new(file);
    let mut modules = 0.0;
    let mut fuel = 0.0;

    for (_, line) in reader.lines().enumerate() {
        let mass = line.unwrap().parse::<f64>().unwrap();

        modules += (mass / 3.0).floor() - 2.0;
        fuel += calculate_fuel(mass);
    }

    println!("Module mass: {}", modules);
    println!("Fuel required by modules: {}", fuel);
}
