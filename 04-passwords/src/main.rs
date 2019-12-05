use std::env;
use std::process;

/// Converts an i32 to a vector of its individual digits.
fn number_to_vec(n: i32) -> Vec<i32> {
    // All numbers are six digits.
    assert!(n < 999999);

    let mut digits = Vec::with_capacity(6);
    let mut n = n;

    while n > 9 {
        digits.push(n % 10);
        n = n / 10;
    }

    digits.push(n);
    digits.reverse();
    digits
}

/// Checks a range of numbers to find the total number of valid password for part one and two of
/// the challenge. Returns a 2-element tuple with the answers for part one and two respectively.
fn check_password(range: std::ops::Range<i32>) -> (i32, i32) {
    let mut doubles = 0;
    let mut doubles_exact = 0;

    for number in range {
        let digits = number_to_vec(number);

        if check_increments(&digits) {
            if check_double(&digits, false) {
                doubles += 1;
            }

            if check_double(&digits, true) {
                doubles_exact += 1;
            }
        }
    }

    (doubles, doubles_exact)
}

/// Checks the vector of digits for consecutive numbers. If `exact` is true, only two consecutive
/// number (not three or more) will be considered a valid match.
fn check_double(digits: &Vec<i32>, exact: bool) -> bool {
    let mut iter = digits.iter().peekable();
    let mut prev = iter.next().unwrap();

    while let Some(digit) = iter.next() {
        if digit == prev {
            if !exact || iter.peek().is_none() || *iter.peek().unwrap() != digit {
                // Non-exact matches allow any two or more consecutive digits. Exact matches should
                // return early only if we're at the end of the sequence.
                return true;
            }

            while let Some(peek) = iter.peek() {
                if *peek == digit {
                    iter.next();
                } else {
                    break;
                }
            }
        }

        prev = digit;
    }

    false
}

/// Checks that each number in the vector is equal or greater than its predecessor.
fn check_increments(digits: &Vec<i32>) -> bool {
    for i in 0..(digits.len() - 1) {
        if digits[i + 1] < digits[i] {
            return false;
        }
    }

    true
}

/// Reads the range of numbers to check from the command line.
fn read_numbers() -> Result<std::ops::Range<i32>, &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        return Err("You must provide start and finish numbers.");
    }

    let start = args[1].parse::<i32>();
    let finish = args[2].parse::<i32>();

    if start.is_err() || finish.is_err() {
        return Err("Invalid start or finish number.");
    }

    Ok(start.unwrap()..finish.unwrap())
}

fn main() {
    let range = read_numbers();

    if let Err(message) = range {
        eprintln!("{}", message);
        process::exit(1);
    }

    let (doubles, doubles_exact) = check_password(range.unwrap());
    println!("Part 1: {}  Part 2: {}", doubles, doubles_exact);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_doublet() {
        assert!(check_double(&vec![1, 1, 2, 3, 4, 5], false));
        assert!(check_double(&vec![1, 1, 1, 3, 4, 5], false));
        assert!(check_double(&vec![1, 2, 3, 4, 5, 5], false));
        assert!(check_double(&vec![1, 1, 1, 3, 4, 5], false));

        assert!(!check_double(&vec![1, 2, 3, 4, 5, 6], false));
        assert!(!check_double(&vec![1, 2, 3, 4, 5, 6], false));
    }

    #[test]
    fn test_check_exact_doublet() {
        assert!(check_double(&vec![1, 1, 2, 3, 4, 5], true));
        assert!(check_double(&vec![1, 1, 1, 1, 4, 4], true));
        assert!(!check_double(&vec![1, 1, 1, 3, 4, 5], true));
        assert!(!check_double(&vec![1, 1, 1, 1, 4, 5], true));

        assert!(check_double(&vec![1, 2, 3, 4, 5, 5], true));
        assert!(!check_double(&vec![1, 2, 3, 4, 4, 4], true));

        assert!(!check_double(&vec![1, 2, 3, 4, 5, 6], true));
    }

    #[test]
    fn test_check_increments() {
        assert!(check_increments(&vec![1, 2, 3, 4, 5, 6]));
        assert!(check_increments(&vec![1, 2, 3, 4, 5, 5]));
        assert!(check_increments(&vec![1, 1, 2, 3, 4, 5]));

        assert!(!check_increments(&vec![1, 0, 2, 3, 4, 5]));
        assert!(!check_increments(&vec![1, 2, 3, 4, 5, 4]));
    }

    #[test]
    fn test_part_one() {
        let (part_one, _) = check_password(307237..769058);
        assert_eq!(part_one, 889);
    }

    #[test]
    fn test_part_two() {
        let (_, part_two) = check_password(307237..769058);
        assert_eq!(part_two, 589);
    }
}
