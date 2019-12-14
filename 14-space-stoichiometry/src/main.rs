use std::collections::HashMap;
use std::fs;
use std::io;

#[derive(Debug, PartialEq, Eq)]
struct Reactant {
    name: String,
    quantity: i64,
}

impl From<String> for Reactant {
    /// Parses a string into a Reactant. Assumes that the string is a valid reaction definition. A
    /// definition should be a number and a name for the reactant split by whitespace.
    ///
    /// from will panic if the definition string is not in the expected format.
    fn from(definition: String) -> Reactant {
        let mut split = definition.split_whitespace();

        Reactant {
            quantity: split
                .next()
                .expect("Expected a reaction quantity")
                .to_string()
                .parse::<i64>()
                .expect("Reactant quantity is not a number"),
            name: split.next().expect("Expected a reaction name").to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Reaction {
    inputs: Vec<Reactant>,
    output: Reactant,
}

impl From<String> for Reaction {
    /// Takes a string representing a complete reaction and returns a Struct representing the inputs
    /// and output.
    fn from(line: String) -> Reaction {
        let mut parts = line.split(" => ");
        let input_strs = parts.next().expect("Expected input reactants").split(',');

        let mut inputs = Vec::new();

        // left hand side is a string of inputs, right hand side is a string representing a single
        // output
        for input_str in input_strs {
            inputs.push(Reactant::from(input_str.trim().to_string()))
        }

        Reaction {
            inputs,
            output: Reactant::from(
                parts
                    .next()
                    .expect("Expected output reactant")
                    .trim()
                    .to_string(),
            ),
        }
    }
}

/// Reads the input containing a list of all reactions and returns a hashmap of the output names to
/// their Reaction.
fn parse_input(input: String) -> HashMap<String, Reaction> {
    input
        .lines()
        .map(|line| {
            let reaction = Reaction::from(line.trim().to_string());
            (reaction.output.name.clone(), reaction)
        })
        .collect::<HashMap<String, Reaction>>()
}

fn ore_from_fuel(reactions: &HashMap<String, Reaction>, fuel_amount: i64) -> i64 {
    // Keep track of the name of the resources we want more of.
    let mut wanted_names = Vec::new();

    // Keep track of how much of each resource we want.
    let mut wanted = HashMap::new();

    wanted_names.push("FUEL".to_string());
    wanted.insert("FUEL".to_string(), fuel_amount);

    while let Some(wanted_name) = wanted_names.pop() {
        let reaction = reactions
            .get(&wanted_name)
            .expect(&format!("Expected reaction {} to exist", wanted_name));

        // The amount of a resource we need is the amount determined in previous iterations, divided
        // by however many is produced by the reaction.
        let needed =
            ((*wanted.get(&wanted_name).unwrap() as f64) / reaction.output.quantity as f64).ceil();

        for input in &reaction.inputs {
            // Queue up production of however much of the input is required.
            let required_amount = wanted.entry(input.name.clone()).or_insert(0);
            *required_amount += (needed * input.quantity as f64) as i64;

            // Queue up the input, as long as the reaction exists. If it doesn't, it will be ORE.
            if reactions.contains_key(&input.name) {
                wanted_names.push(input.name.clone())
            }
        }

        // This output from this reaction will be produced in future iterations, so this resource
        // is no longer needed.
        let wanted_output_amount = wanted.entry(reaction.output.name.clone()).or_insert(0);
        *wanted_output_amount -= (needed as i64) * reaction.output.quantity;
    }

    *wanted.get(&"ORE".to_string()).expect("Expected ORE amount")
}

/// Takes a map of reactions and returns how many ORE are required to produce one FUEL.
fn part_one(reactions: HashMap<String, Reaction>) -> i64 {
    ore_from_fuel(&reactions, 1)
}

/// Do a binary search to see how much FUEL is produced by the target amount of ORE.
fn part_two(reactions: HashMap<String, Reaction>, target: i64) -> i64 {
    // Find the minimum amount of ore which would be needed for one unit of fuel.
    let mut low = target / ore_from_fuel(&reactions, 1);

    // Best case scenario one fuel comes from one ore.
    let mut high = target;

    while high > low {
        let mid = (high + low) / 2;

        if mid == low {
            break;
        }

        let ore = ore_from_fuel(&reactions, mid);

        if ore > target {
            high = mid;
        } else {
            low = mid;
        }
    }

    low
}

fn main() -> Result<(), io::Error> {
    let data = fs::read_to_string("data/reactions.txt")?;
    let data = data.trim();

    println!("Part one: {:?}", part_one(parse_input(data.to_string())));

    println!(
        "Part two: {:?}",
        part_two(parse_input(data.to_string()), 1_000_000_000_000)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trim_leading_whitespace(string: &str) -> String {
        let lines: Vec<&str> = string.lines().map(|line| line.trim()).collect();
        lines.join("\n")
    }

    #[test]
    fn test_reactant_from_string() {
        assert_eq!(
            Reactant::from("2 A".to_string()),
            Reactant {
                name: "A".to_string(),
                quantity: 2
            }
        );

        assert_eq!(
            Reactant::from("6 AX".to_string()),
            Reactant {
                name: "AX".to_string(),
                quantity: 6
            }
        );
    }

    #[test]
    fn test_reaction_from_string() {
        assert_eq!(
            Reaction::from("10 ORE => 2 A".to_string()),
            Reaction {
                inputs: vec![Reactant {
                    name: "ORE".to_string(),
                    quantity: 10
                }],
                output: Reactant {
                    name: "A".to_string(),
                    quantity: 2
                }
            }
        );

        assert_eq!(
            Reaction::from("10 ORE, 2 A => 1 B".to_string()),
            Reaction {
                inputs: vec![
                    Reactant {
                        name: "ORE".to_string(),
                        quantity: 10
                    },
                    Reactant {
                        name: "A".to_string(),
                        quantity: 2
                    },
                ],
                output: Reactant {
                    name: "B".to_string(),
                    quantity: 1
                }
            }
        );
    }

    #[test]
    fn test_parse_input() {
        let input = trim_leading_whitespace(
            "10 ORE => 10 A
             1 ORE => 1 B
             7 A, 1 B => 1 C
             7 A, 1 C => 1 D
             7 A, 1 D => 1 E
             7 A, 1 E => 1 FUEL",
        );

        let parsed = parse_input(input);

        assert_eq!(
            parsed.get("FUEL"),
            Some(&Reaction {
                inputs: vec![
                    Reactant {
                        name: "A".to_string(),
                        quantity: 7,
                    },
                    Reactant {
                        name: "E".to_string(),
                        quantity: 1
                    }
                ],
                output: Reactant {
                    name: "FUEL".to_string(),
                    quantity: 1,
                }
            })
        );

        assert_eq!(
            parsed.get("B"),
            Some(&Reaction {
                inputs: vec![Reactant {
                    name: "ORE".to_string(),
                    quantity: 1,
                },],
                output: Reactant {
                    name: "B".to_string(),
                    quantity: 1,
                }
            })
        );
    }

    #[test]
    fn test_part_one() {
        let reactions = parse_input(trim_leading_whitespace(
            "10 ORE => 10 A
             1 ORE => 1 B
             7 A, 1 B => 1 C
             7 A, 1 C => 1 D
             7 A, 1 D => 1 E
             7 A, 1 E => 1 FUEL",
        ));

        assert_eq!(part_one(reactions), 31);

        let reactions = parse_input(trim_leading_whitespace(
            "9 ORE => 2 A
             8 ORE => 3 B
             7 ORE => 5 C
             3 A, 4 B => 1 AB
             5 B, 7 C => 1 BC
             4 C, 1 A => 1 CA
             2 AB, 3 BC, 4 CA => 1 FUEL",
        ));

        assert_eq!(part_one(reactions), 165);

        let reactions = parse_input(trim_leading_whitespace(
            "157 ORE => 5 NZVS
             165 ORE => 6 DCFZ
             44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
             12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
             179 ORE => 7 PSHF
             177 ORE => 5 HKGWZ
             7 DCFZ, 7 PSHF => 2 XJWVT
             165 ORE => 2 GPVTF
             3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
        ));

        assert_eq!(part_one(reactions), 13312);

        let reactions = parse_input(trim_leading_whitespace(
            "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
             17 NVRVD, 3 JNWZP => 8 VPVL
             53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
             22 VJHF, 37 MNCFX => 5 FWMGM
             139 ORE => 4 NVRVD
             144 ORE => 7 JNWZP
             5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
             5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
             145 ORE => 6 MNCFX
             1 NVRVD => 8 CXFTF
             1 VJHF, 6 MNCFX => 4 RFSQX
             176 ORE => 6 VJHF",
        ));

        assert_eq!(part_one(reactions), 180697);

        let reactions = parse_input(trim_leading_whitespace(
            "171 ORE => 8 CNZTR
             7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
             114 ORE => 4 BHXH
             14 VRPVC => 6 BMBT
             6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
             6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
             15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
             13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
             5 BMBT => 4 WPTQ
             189 ORE => 9 KTJDG
             1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
             12 VRPVC, 27 CNZTR => 2 XDBXC
             15 KTJDG, 12 BHXH => 5 XCVML
             3 BHXH, 2 VRPVC => 7 MZWV
             121 ORE => 7 VRPVC
             7 XCVML => 6 RJRHP
             5 BHXH, 4 VRPVC => 5 LTCX",
        ));

        assert_eq!(part_one(reactions), 2210736);
    }

    #[test]
    fn test_part_two() {
        let reactions = parse_input(trim_leading_whitespace(
            "157 ORE => 5 NZVS
             165 ORE => 6 DCFZ
             44 XJWVT, 5 KHKGT, 1 QDVJ, 29 NZVS, 9 GPVTF, 48 HKGWZ => 1 FUEL
             12 HKGWZ, 1 GPVTF, 8 PSHF => 9 QDVJ
             179 ORE => 7 PSHF
             177 ORE => 5 HKGWZ
             7 DCFZ, 7 PSHF => 2 XJWVT
             165 ORE => 2 GPVTF
             3 DCFZ, 7 NZVS, 5 HKGWZ, 10 PSHF => 8 KHKGT",
        ));

        assert_eq!(part_two(reactions, 1_000_000_000_000), 82892753);

        let reactions = parse_input(trim_leading_whitespace(
            "2 VPVL, 7 FWMGM, 2 CXFTF, 11 MNCFX => 1 STKFG
             17 NVRVD, 3 JNWZP => 8 VPVL
             53 STKFG, 6 MNCFX, 46 VJHF, 81 HVMC, 68 CXFTF, 25 GNMV => 1 FUEL
             22 VJHF, 37 MNCFX => 5 FWMGM
             139 ORE => 4 NVRVD
             144 ORE => 7 JNWZP
             5 MNCFX, 7 RFSQX, 2 FWMGM, 2 VPVL, 19 CXFTF => 3 HVMC
             5 VJHF, 7 MNCFX, 9 VPVL, 37 CXFTF => 6 GNMV
             145 ORE => 6 MNCFX
             1 NVRVD => 8 CXFTF
             1 VJHF, 6 MNCFX => 4 RFSQX
             176 ORE => 6 VJHF",
        ));

        assert_eq!(part_two(reactions, 1_000_000_000_000), 5586022);

        let reactions = parse_input(trim_leading_whitespace(
            "171 ORE => 8 CNZTR
             7 ZLQW, 3 BMBT, 9 XCVML, 26 XMNCP, 1 WPTQ, 2 MZWV, 1 RJRHP => 4 PLWSL
             114 ORE => 4 BHXH
             14 VRPVC => 6 BMBT
             6 BHXH, 18 KTJDG, 12 WPTQ, 7 PLWSL, 31 FHTLT, 37 ZDVW => 1 FUEL
             6 WPTQ, 2 BMBT, 8 ZLQW, 18 KTJDG, 1 XMNCP, 6 MZWV, 1 RJRHP => 6 FHTLT
             15 XDBXC, 2 LTCX, 1 VRPVC => 6 ZLQW
             13 WPTQ, 10 LTCX, 3 RJRHP, 14 XMNCP, 2 MZWV, 1 ZLQW => 1 ZDVW
             5 BMBT => 4 WPTQ
             189 ORE => 9 KTJDG
             1 MZWV, 17 XDBXC, 3 XCVML => 2 XMNCP
             12 VRPVC, 27 CNZTR => 2 XDBXC
             15 KTJDG, 12 BHXH => 5 XCVML
             3 BHXH, 2 VRPVC => 7 MZWV
             121 ORE => 7 VRPVC
             7 XCVML => 6 RJRHP
             5 BHXH, 4 VRPVC => 5 LTCX",
        ));

        assert_eq!(part_two(reactions, 1_000_000_000_000), 460664);
    }
}
