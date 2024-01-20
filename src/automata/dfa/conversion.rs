#![allow(dead_code)] // REMOVE
use super::Dfa;
use crate::nfa::{StateIter, TransitionIter};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
#[allow(unused)] // REMOVE
pub enum Token {
    Symbol(char),
    State(u32),
    Operator(char),
    OpenParent,
    ClosedParent,
    EmptyString,
}

/// Calculates the state equation for a given state.
/// Returns the list of states that transition into the
/// given state and the symbol for the transition
/// sepparated by the union symbol.
fn single_state_equation(dfa: &Dfa, state: u32) -> Vec<Token> {
    let mut expr: Vec<Token> = Vec::new();

    if state == *dfa.states.first().unwrap() {
        expr.push(Token::EmptyString);
        expr.push(Token::Operator('|'));
    }

    dfa.transitions_iter()
        .filter(|(_, &dest)| dest == state)
        .for_each(|(&(state, symbol), _)| {
            expr.push(Token::State(state));
            expr.push(Token::Symbol(symbol));
            expr.push(Token::Operator('|'));
        });

    if let Token::Operator('|') = expr.last().unwrap() {
        expr.pop();
    }

    expr
}

/// Returns a lookup table that maps states to their equations
fn all_states_equations(dfa: &Dfa) -> HashMap<u32, Vec<Token>> {
    let mut lookup: HashMap<u32, Vec<Token>> = HashMap::new();

    dfa.states_iter().for_each(|&state| {
        lookup.insert(state, single_state_equation(dfa, state));
    });

    lookup
}

/// Checks if the equation is reduced to a single state
/// for example: q1 = q1(ab) + q1(abc) + q1(bc) -> true
/// for example: q1 = q1(ab) + q2(abc) + q3(bc) -> false
fn equation_is_reduced(equation: &[Token], state: u32) -> bool {
    let mut reduced = true;

    for token in equation {
        if let Token::State(member) = token {
            if *member != state {
                reduced = false;
            }
        }
    }
    reduced
}

/// Checks if the expression is valid for the use of Ardens theorem.
/// The expression should be of the form: R = Q + RP
fn valid_for_ardens_theorem(equation: &[Token], state: u32) -> bool {
    if equation
        .iter()
        .filter(|&token| matches!(token, Token::State(_)))
        .count()
        != 1
    {
        return false;
    }

    let result = equation.iter().find(|&token| {
        if let Token::State(s) = token {
            *s == state
        } else {
            false
        }
    });

    result.is_some()
}

///  Applies Ardens theorem to a given equation.
///  Checks for the pattern: R = Q + RP
///  and replaces it with the pattern: R = QP*
fn ardens_theorem(equation: &mut Vec<Token>, state: u32) {
    // find the position of the state in the equation
    let state_idx = equation.iter().find_position(|&token| {
        if let Token::State(s) = token {
            *s == state
        } else {
            false
        }
    });
    if state_idx.is_none() {
        return;
    }
    let state_idx = state_idx.unwrap().0;

    match equation[state_idx - 1] {
        Token::Operator(op) => {
            if op != '|' {
                return;
            }
        }
        _ => return,
    }

    let star_idx = equation.iter().skip(state_idx).find_position(|&token| {
        if let Token::Operator(op) = token {
            *op == '|'
        } else {
            false
        }
    });
    let star_idx = match star_idx {
        Some(idx) => state_idx + idx.0 - 1,
        None => equation.len() - 1,
    };

    let copy = equation.clone();
    equation.clear();

    copy[0..state_idx - 1]
        .to_vec()
        .iter()
        .for_each(|token| equation.push(token.clone()));
    equation.push(Token::OpenParent);
    copy[state_idx + 1..=star_idx]
        .to_vec()
        .iter()
        .for_each(|token| equation.push(token.clone()));
    equation.push(Token::ClosedParent);
    equation.push(Token::Operator('*'));
    copy[star_idx + 1..copy.len()]
        .to_vec()
        .iter()
        .for_each(|token| equation.push(token.clone()));
}

fn new_ardens_theorem(equation: &mut Vec<Token>, state: u32) {
    // find the position of the state in the equation
    let state_idx = equation.iter().find_position(|&token| {
        if let Token::State(s) = token {
            *s == state
        } else {
            false
        }
    });
    if state_idx.is_none() {
        return;
    }
    let state_idx = state_idx.unwrap().0;
    let star_idx = equation.len() - 1;

    let copy = equation.clone();
    equation.clear();

    copy[0..state_idx - 1]
        .to_vec()
        .iter()
        .for_each(|token| equation.push(token.clone()));
    equation.push(Token::OpenParent);
    copy[state_idx + 1..=star_idx]
        .to_vec()
        .iter()
        .for_each(|token| equation.push(token.clone()));
    equation.push(Token::ClosedParent);
    equation.push(Token::Operator('*'));
    copy[star_idx + 1..copy.len()]
        .to_vec()
        .iter()
        .for_each(|token| equation.push(token.clone()));
}

pub fn contains_states(equation: &[Token]) -> bool {
    let result = equation
        .iter()
        .find(|&token| matches!(token, Token::State(_)));

    result.is_some()
}

pub fn replace_state(
    equation: &mut Vec<Token>,
    lookup_table: &HashMap<u32, Vec<Token>>,
    state: u32,
) {
    let position = equation.iter().find_position(|&token| {
        if let Token::State(st) = token {
            *st == state
        } else {
            false
        }
    });

    let position = match position {
        Some(target) => target,
        None => return,
    };

    equation.splice(
        position.0..=position.0,
        lookup_table.get(&state).unwrap().clone(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::nfa::Nfa;

    #[test]
    fn dfa_single_state_equation() {
        let nfa = Nfa::from("0*1*").unwrap();
        // println!("{:?}", nfa);
        let dfa = nfa.to_dfa();
        // println!("{:?}", dfa);

        let x = single_state_equation(&dfa, 3);
        println!("{:?}", x);
    }

    #[test]
    fn ardens_theorem_test() {
        let mut input = vec![
            Token::State(1),
            Token::Symbol('a'),
            Token::Symbol('b'),
            Token::Operator('|'),
            Token::State(0),
            Token::Symbol('a'),
            Token::Operator('|'),
            Token::State(2),
            Token::Symbol('b'),
        ];

        ardens_theorem(&mut input, 0);

        assert_eq!(
            vec![
                Token::State(1),
                Token::Symbol('a'),
                Token::Symbol('b'),
                Token::OpenParent,
                Token::Symbol('a'),
                Token::ClosedParent,
                Token::Operator('*'),
                Token::Operator('|'),
                Token::State(2),
                Token::Symbol('b'),
            ],
            input
        );
    }

    #[test]
    fn replacing_states() {
        let mut input = vec![
            Token::State(1),
            Token::Symbol('a'),
            Token::Symbol('b'),
            Token::Operator('|'),
            Token::State(0),
            Token::Symbol('a'),
            Token::Operator('|'),
            Token::State(2),
            Token::Symbol('b'),
        ];

        let mut lookup: HashMap<u32, Vec<Token>> = HashMap::new();
        lookup.insert(
            0,
            vec![
                Token::State(1),
                Token::Symbol('a'),
                Token::Operator('|'),
                Token::State(2),
                Token::Symbol('a'),
                Token::Symbol('b'),
            ],
        );

        replace_state(&mut input, &lookup, 0);

        assert_eq!(
            vec![
                Token::State(1),
                Token::Symbol('a'),
                Token::Symbol('b'),
                Token::Operator('|'),
                Token::State(1),
                Token::Symbol('a'),
                Token::Operator('|'),
                Token::State(2),
                Token::Symbol('a'),
                Token::Symbol('b'),
                Token::Symbol('a'),
                Token::Operator('|'),
                Token::State(2),
                Token::Symbol('b'),
            ],
            input
        );
    }
}
