use itertools::Itertools;

/// Generates a power set for a given set
///
/// # Examples
/// ```rust
/// use fmsi::util;
///
/// let result = util::create_power_set(&vec![0, 1, 2]);
///
/// assert_eq!(vec![
///     vec![0],
///     vec![0, 1],
///     vec![0, 1, 2],
///     vec![1],
///     vec![1, 2],
///     vec![2]
/// ], result);
/// ```
pub fn create_power_set(set: &Vec<u32>) -> Vec<Vec<u32>> {
    let powerset: Vec<Vec<u32>> = (0..=set.len())
        .tuple_combinations()
        .map(|(start, end)| (start as u32..end as u32).collect_vec())
        .collect();

    powerset
}

/// Checks the fiven regular expression for correctness
///
/// # Examples
/// ```rust
/// use fmsi::util;
/// 
/// // valid regex
/// assert!(util::check_for_correctness("ab|b(ab|c)*").is_ok());
///
/// // invalid regex
/// assert!(util::check_for_correctness("ab|b(a|*c))").is_err());
/// ```
pub fn check_for_correctness(regex: &str) -> Result<(), &'static str> {
    let mut counter = 0;
    let invalid_patterns = ["(|", "|)", "(*", "|*", "||", "**"];

    for symbol in regex.chars() {
        if symbol == '(' {
            counter += 1;
        } else if symbol == ')' {
            counter -= 1;
        }
    }

    for pattern in invalid_patterns.iter() {
        if regex.contains(pattern) {
            return Err("Regex contains an invalid pattern");
        }
    }

    if counter == 0 {
        Ok(())
    } else {
        Err("Brackets don't match")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn util_power_set() {
        let result = create_power_set(&vec![0, 1, 2]);
        assert_eq!(vec![
            vec![0],
            vec![0, 1],
            vec![0, 1, 2],
            vec![1],
            vec![1, 2],
            vec![2]
        ], result);

        println!("{:?}", result);
    }

    #[test]
    fn checking_for_correctness() {
        assert!(check_for_correctness("(*ab)").is_err());
        assert!(check_for_correctness("(|ab)").is_err());
        assert!(check_for_correctness("(ab|)").is_err());
        assert!(check_for_correctness("a||b").is_err());
        assert!(check_for_correctness("ab**").is_err());
        assert!(check_for_correctness("(ab|*)").is_err());
        assert!(check_for_correctness("(ab|b)*)").is_err());

        assert!(check_for_correctness("a|b(ab*|a)*").is_ok());
    }
}
