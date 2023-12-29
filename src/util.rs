use crate::language::EMPTY_STRING;
use crate::nfa::*;
use itertools::Itertools;
use std::collections::vec_deque::VecDeque;

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
pub fn create_power_set(set: &[u32]) -> Vec<Vec<u32>> {
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

/// Calculates the epsilon clojure for a given state
/// of a given nfa.
///
/// # Example
/// ```rust
/// use fmsi::nfa::Nfa;
/// use fmsi::util::state_epsilon_clojure;
///
/// let nfa = Nfa::from("a|(ab|b)*").unwrap();
/// let result = state_epsilon_clojure(&nfa, 0);
///
/// assert_eq!(vec![0, 1, 3, 4, 5, 6, 9], result);
/// ```
#[allow(dead_code)]
pub fn state_epsilon_clojure(nfa: &Nfa, state: u32) -> Vec<u32> {
    let mut clojure = vec![state];
    let mut queue: VecDeque<u32> = VecDeque::new();

    // add the given state to the queue
    queue.push_back(state);

    loop {
        // grab the first element from the queue
        let current = queue.pop_front();

        match current {
            Some(current) => {
                if let Some(destinations) = nfa.get_transition((current, EMPTY_STRING)) {
                    for &state in destinations {
                        // if there are epsilon transitions for this state
                        // add them to the clojure
                        if !clojure.contains(&state) {
                            clojure.push(state);
                        }

                        // if they aren't already in the queue, add them
                        if !queue.contains(&state) {
                            queue.push_back(state);
                        }
                    }
                }
            }
            // if there are no elements left, the algorithm is done
            None => break,
        }
    }
    clojure.sort();
    clojure
}

/// Calculates the epsilon clojure for a given set of states
/// of a given nfa.
///
/// # Example
/// ```rust
/// use fmsi::nfa::Nfa;
/// use fmsi::util::set_epsilon_clojure;
///
/// let nfa = Nfa::from("a|(ab|b)*").unwrap();
/// let result = set_epsilon_clojure(&nfa, &[0, 8]);
///
/// assert_eq!(vec![0, 1, 3, 4, 5, 6, 8, 9], result);
/// ```
pub fn set_epsilon_clojure(nfa: &Nfa, set: &[u32]) -> Vec<u32> {
    let mut clojure: Vec<u32> = Vec::new();

    for state in set {
        let mut set_clojure = state_epsilon_clojure(nfa, *state);
        set_clojure.retain(|&element| !clojure.contains(&element));
        clojure.extend(set_clojure);
    }

    clojure.sort();
    clojure
}

/// Returns a set to which a given set transitions to
/// for a given symbol.
///
/// # Example
/// ```rust
/// use fmsi::nfa::Nfa;
/// use fmsi::util::set_transitions;
///
/// let nfa = Nfa::from("a|(ab|b)*").unwrap();
/// let result = set_transitions(&nfa, &[7, 9], 'b');
///
/// assert_eq!(vec![8, 10], result);
/// ```
pub fn set_transitions(nfa: &Nfa, set: &[u32], symbol: char) -> Vec<u32> {
    let mut target: Vec<u32> = Vec::new();

    for state in set {
        if let Some(destinations) = nfa.get_transition((*state, symbol)) {
            let mut to_add: Vec<u32> = destinations
                .iter()
                .filter(|&d| !target.contains(d))
                .cloned()
                .collect();

            target.append(&mut to_add);
        }
    }

    target.sort();
    target
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn util_power_set() {
        let result = create_power_set(&[0, 1, 2]);
        assert_eq!(
            vec![
                vec![0],
                vec![0, 1],
                vec![0, 1, 2],
                vec![1],
                vec![1, 2],
                vec![2]
            ],
            result
        );

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

    #[test]
    fn nfa_state_epsilon_clojure() {
        let nfa = Nfa::from("a|(ab|b)*").unwrap();

        assert_eq!(vec![0, 1, 3, 4, 5, 6, 9], state_epsilon_clojure(&nfa, 0));
        assert_eq!(vec![4, 5, 6, 8, 9], state_epsilon_clojure(&nfa, 8));
        assert_eq!(vec![5, 6, 9], state_epsilon_clojure(&nfa, 5));
        assert_eq!(vec![1], state_epsilon_clojure(&nfa, 1));
    }

    #[test]
    fn nfa_set_epsilon_clojure() {
        let nfa = Nfa::from("a|(ab|b)*").unwrap();

        assert_eq!(
            vec![0, 1, 3, 4, 5, 6, 8, 9],
            set_epsilon_clojure(&nfa, &[0, 8])
        );
        assert_eq!(
            vec![0, 1, 3, 4, 5, 6, 8, 9],
            set_epsilon_clojure(&nfa, &[0, 1, 3, 4, 5, 6, 8, 9])
        );
        assert_eq!(vec![4, 5, 6, 9], set_epsilon_clojure(&nfa, &[4, 5]));
        assert_eq!(vec![1, 6], set_epsilon_clojure(&nfa, &[1, 6]));
    }

    #[test]
    fn nfa_set_transitions() {
        let nfa = Nfa::from("a|(ab|b)*").unwrap();
        let empty: Vec<u32> = Vec::new();

        assert_eq!(vec![8, 10], set_transitions(&nfa, &[7, 9], 'b'));
        assert_eq!(vec![2, 7], set_transitions(&nfa, &[1, 6], 'a'));
        assert_eq!(empty, set_transitions(&nfa, &[0, 3, 4], 'a'));
    }
}
