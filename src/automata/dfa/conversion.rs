use crate::nfa::TransitionIter;

use super::Dfa;

pub enum Token {
    Symbol(char),
    State(u32),
    Operator(char),
    EmptyString,
}

pub fn single_state_equation(dfa: &Dfa, state: u32) -> (u32, Vec<Token>) {
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

    (state, expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::nfa::Nfa;

    #[test]
    fn dfa_single_state_equation() {
        let nfa = Nfa::from("0*1*").unwrap();
        println!("{:?}", nfa);
        let dfa = nfa.to_dfa();
        println!("{:?}", dfa);

    }
}
