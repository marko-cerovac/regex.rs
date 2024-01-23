use super::Dfa;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
enum Token {
    Symbol(char),
    EmptyString,
    EmptySet,
    Union,
    KleeneStar,
    OpenParent,
    ClosedParent,
}

#[derive(Clone)]
pub struct Equation(Vec<Token>);

impl Equation {
    pub fn new() -> Self {
        Equation(Vec::new())
    }

    /// Adds parenthese
    pub fn add_parenthasis(&mut self) {
        // don't add if there are some already
        if matches!(self.0.first().unwrap(), Token::OpenParent)
            && matches!(self.0.last().unwrap(), Token::ClosedParent)
        {
            return;
        }

        // don't add if there's only one token
        if self.0.len() > 1 {
            self.0.insert(0, Token::OpenParent);
            self.0.push(Token::ClosedParent);
        }
    }

    /// Used for simplyfying a single (non-complex) equation
    pub fn simplify(&mut self) {
        if self.0.len() == 1 {
            return;
        }
        let mut done = false;

        while !done {
            done = true;
            // find the closing bracket
            if let Some(index) = self
                .0
                .iter()
                .position(|token| matches!(token, Token::ClosedParent))
            {
                if let Some(opening_parent) = self.0.get(index - 2) {
                    // if there's only one symbol surrounded by parentheses,
                    // remove them
                    if matches!(opening_parent, Token::OpenParent) {
                        self.0.remove(index);
                        self.0.remove(index - 2);
                        done = false;
                    // if the expression consists of empty parentheses,
                    // remove them
                    } else if let Some(opening_parent) = self.0.get(index - 1) {
                        if matches!(opening_parent, Token::OpenParent) {
                            self.0.remove(index);
                            self.0.remove(index - 1);
                            done = false;
                        }
                    }
                }
            }
            // println!("{:?}", self.0); // DBG

            // if you find an empty string, remove it
            if let Some(empty_index) = self
                .0
                .iter()
                .position(|token| matches!(token, Token::EmptyString))
            {
                // if there is a union operator after the empty string,
                // remove it too
                if let Some(token) = self.0.get(empty_index + 1) {
                    if matches!(
                        token,
                        Token::Symbol(_) | Token::OpenParent | Token::EmptyString) 
                    {
                        self.0.remove(empty_index);
                        done = false;
                    }
                }
            }
            // println!("{:?}", self.0); // DBG
        }

        // if there is only one token left,
        // make sure it is valid
        if self.0.len() == 1 {
            match self.0[0] {
                Token::Symbol(_) => {}
                Token::EmptyString => {}
                _ => {
                    self.0.remove(0);
                }
            }
        }
        // println!("{:?}", self.0); // DBG
    }

    /// Returns true if the equation consists only of an empty set
    pub fn is_empty_set(&self) -> bool {
        if self.0.len() != 1 {
            return false;
        }

        matches!(self.0[0], Token::EmptySet)
    }
}

impl Default for Equation {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Equation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut regex = String::with_capacity(self.0.len());

        for token in self.0.iter() {
            match token {
                Token::Symbol(symbol) => regex.push(*symbol),
                Token::Union => regex.push('|'),
                Token::KleeneStar => regex.push('*'),
                Token::OpenParent => regex.push('('),
                Token::ClosedParent => regex.push(')'),
                Token::EmptyString => {
                    // regex.push_str("\\0");
                    regex.push('𝜖');
                    // regex.push_str("[empty string]");
                }
                Token::EmptySet => {},
            }
        }
        write!(f, "{}", regex)
    }
}

pub fn get_regex(old_dfa: &Dfa) -> Equation {
    let mut dfa = old_dfa.clone();
    let mut old_lookup_table: HashMap<(u32, u32, u32), Equation> = HashMap::new();

    dfa.increment_states(1);
    let num_states = dfa.states.len() as u32;

    // get the initial table where k = 0
    for i in 1..=num_states {
        for j in 1..=num_states {
            old_lookup_table.insert((i, j, 0), get_initial_eq(&dfa, (i, j)));
        }
    }

    // R(i, j, k) = R(i, j, k - 1) + R(i, k, k-1)R(k, k, k -1)* + R(k, j, k - 1)
    for k in 1..=num_states {
        let mut new_lookup_table: HashMap<(u32, u32, u32), Equation> = HashMap::new();

        for i in 1..=num_states {
            for j in 1..=num_states {
                // fetch the current values and qpply the formula
                let mut eq: Equation = Equation::new();
                let mut r1 = old_lookup_table // R(i, j, k - 1)
                    .get(&(i, j, k - 1))
                    .unwrap()
                    .clone();
                // println!("{:?}", r1.0); // DBG
                let mut r2 = old_lookup_table // R(i, k, k - 1)
                    .get(&(i, k, k - 1))
                    .unwrap()
                    .clone();
                // println!("{:?}", r2.0); // DBG
                let mut r3 = old_lookup_table // R(k, k, k - 1)
                    .get(&(k, k, k - 1))
                    .unwrap()
                    .clone();
                // println!("{:?}", r3.0); // DBG
                let mut r4 = old_lookup_table // R(k, j, k - 1)
                    .get(&(k, j, k - 1))
                    .unwrap()
                    .clone();
                // println!("{:?}", r4.0); // DBG

                // if r1 is an empty set, disregard it
                if !r1.is_empty_set() {
                    r1.add_parenthasis();
                    // println!("{:?}", r1.0); // DBG
                    r1.simplify();
                    // println!("{:?}", r1.0); // DBG
                    eq.0.append(&mut r1.0);
                    eq.0.push(Token::Union);
                }

                // if r2 or r4 are empty sets, disregard everything
                if !r2.is_empty_set() && !r4.is_empty_set() {
                    r2.add_parenthasis();
                    // println!("{:?}", r2.0); // DBG
                    r2.simplify();
                    // println!("{:?}", r2.0); // DBG
                    eq.0.append(&mut r2.0);
                    // empty_set* = empty_string
                    if r3.is_empty_set() {
                        eq.0.push(Token::EmptyString);
                    } else {
                        r3.add_parenthasis();
                        // println!("{:?}", r3.0); // DBG
                        r3.0.push(Token::KleeneStar);
                        r3.simplify();
                        // println!("{:?}", r3.0); // DBG
                        eq.0.append(&mut r3.0);
                    }
                    r4.add_parenthasis();
                    // println!("{:?}", r4.0); // DBG
                    r4.simplify();
                    // println!("{:?}", r4.0); // DBG
                    eq.0.append(&mut r4.0);
                } else {
                    // remove the trailing union operator
                    eq.0.pop();
                }

                if eq.0.is_empty() {
                    eq.0.push(Token::EmptySet);
                }

                // println!("{:?}", eq.0); // DBG

                // push the equation to the new round
                new_lookup_table.insert((i, j, k), eq);
            }
        }
        old_lookup_table = new_lookup_table;
    }

    let mut regex: Equation = Equation::new();
    for accept_state in dfa.accept_states.iter() {
        let mut current = old_lookup_table
            .get(&(1, *accept_state, num_states))
            .unwrap()
            .0
            .clone();
        regex.0.append(&mut current);
        regex.0.push(Token::Union);
    }
    regex.0.pop();
    regex
}

// Returns an initial equation for for a transition
// between two states that doesn't pass through
// eny aditional states
fn get_initial_eq(dfa: &Dfa, (i, j): (u32, u32)) -> Equation {
    let mut symbols: Vec<char> = Vec::new();
    let mut eqv = Equation::new();

    if i == j {
        eqv.0.push(Token::EmptyString);
        eqv.0.push(Token::Union);
    }

    dfa.transition_fn
        .iter()
        .filter(|(&(src, _), &dest)| src == i && dest == j)
        .for_each(|((_, symbol), _)| {
            symbols.push(*symbol);
        });

    // the reason for this Vec is so that
    // the symbols get sorted first and then added to the
    // table
    symbols.sort();
    for symbol in symbols {
        eqv.0.push(Token::Symbol(symbol));
        eqv.0.push(Token::Union);
    }

    // remove the last union
    eqv.0.pop();

    if eqv.0.is_empty() {
        eqv.0.push(Token::EmptySet);
    }

    eqv
}

#[cfg(test)]
mod tests {
    use crate::automata::traits::{Alphabet, State, Transition};

    use super::*;

    #[test]
    fn initial_eq() {
        let mut dfa = Dfa::from("a|(ab|b)*").unwrap();
        dfa.increment_states(1);
        let num_states = dfa.states.len() as u32;

        let mut lookup_table: HashMap<(u32, u32, u32), Equation> = HashMap::new();
        for i in 1..=num_states {
            for j in 1..=num_states {
                lookup_table.insert((i, j, 0), get_initial_eq(&dfa, (i, j)));
                print!("({}, {})", i, j);
                println!("{:?}", get_initial_eq(&dfa, (i, j)).0);
                println!();
            }
        }

        assert_eq!(
            vec![Token::EmptyString],
            lookup_table.get(&(1, 1, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('a')],
            lookup_table.get(&(1, 2, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('b')],
            lookup_table.get(&(1, 3, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(1, 4, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(1, 5, 0)).unwrap().0
        );

        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(2, 1, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptyString],
            lookup_table.get(&(2, 2, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('b')],
            lookup_table.get(&(2, 3, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('a')],
            lookup_table.get(&(2, 4, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(2, 5, 0)).unwrap().0
        );

        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(3, 1, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(3, 2, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptyString, Token::Union, Token::Symbol('b')],
            lookup_table.get(&(3, 3, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(3, 4, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('a')],
            lookup_table.get(&(3, 5, 0)).unwrap().0
        );

        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(4, 1, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(4, 2, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(4, 3, 0)).unwrap().0
        );
        assert_eq!(
            vec![
                Token::EmptyString,
                Token::Union,
                Token::Symbol('a'),
                Token::Union,
                Token::Symbol('b')
            ],
            lookup_table.get(&(4, 4, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(4, 5, 0)).unwrap().0
        );

        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(5, 1, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptySet],
            lookup_table.get(&(5, 2, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('b')],
            lookup_table.get(&(5, 3, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::Symbol('a')],
            lookup_table.get(&(5, 4, 0)).unwrap().0
        );
        assert_eq!(
            vec![Token::EmptyString],
            lookup_table.get(&(5, 5, 0)).unwrap().0
        );
    }

    #[test]
    fn regex_generation() {
        let mut dfa = Dfa::new();
        dfa.add_state();
        dfa.add_symbol('a');
        dfa.add_symbol('b');
        let _ = dfa.add_transition(&(0, 'a'), 1);
        let _ = dfa.add_transition(&(0, 'b'), 0);
        let _ = dfa.add_transition(&(1, 'a'), 1);
        let _ = dfa.add_transition(&(1, 'b'), 1);
        dfa.add_accept_state(1);
        let eq = get_regex(&dfa);
        println!("{}", eq);

        let dfa = Dfa::from("a|(ab|b)*").unwrap();
        let eq = get_regex(&dfa);
        println!("{}", eq);
    }
}
