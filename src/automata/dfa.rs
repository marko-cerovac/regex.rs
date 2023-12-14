use std::collections::HashMap;
use std::default::Default;

#[derive(Debug)]
pub struct Dfa {
    states: Vec<u32>,
    alphabet: Vec<char>,
    transition_fn: HashMap<(u32, char), u32>,
    start_state: u32,
    accept_states: Vec<u32>,
}

impl Default for Dfa {
    fn default() -> Self {
        Dfa {
            states: vec![0], // pocetno stanje je 0
            alphabet: Vec::new(),
            transition_fn: HashMap::new(),
            start_state: 0,
            accept_states: Vec::new(),
        }
    }
}

impl Dfa {
    fn new() -> Self {
        Default::default()
    }

    fn add_state(&mut self) {
        let current_largest = self.states.last().unwrap();
        self.states.push(current_largest + 1);
    }

    fn remove_state(&mut self, target: u32) {
        if let Some(index) = self.states.iter().position(|&e| e == target) {
            self.accept_states.remove(index);
        }

        if let Some(index) = self.accept_states.iter().position(|&e| e == target) {
            self.accept_states.remove(index);
        }

        // uklanja iz tabele svaku tranziciju u kojoj se pojavljuje taget kao trenutno ili buduce
        // stanje
        self.transition_fn
            .retain(|&(curr_state, _), new_state| curr_state != target && *new_state != target);
    }

    fn add_symbol(&mut self, symbol: char) {
        if !self.alphabet.contains(&symbol) {
            self.alphabet.push(symbol);
        }
    }

    fn remove_symbol(&mut self, symbol: char) {
        if let Some(position) = self.alphabet.iter().position(|e| *e == symbol) {
            self.alphabet.remove(position);
        }
    }

    fn add_transition(&mut self, source: (u32, char), target: u32) -> Result<(), &'static str> {
        if !self.states.contains(&source.0) {
            return Err("The source state isn't a valid state");
        }

        if !self.states.contains(&target) {
            return Err("The target state isn't a valid state");
        }

        if !self.alphabet.contains(&source.1) {
            return Err("The symbol is not in the alphabet");
        }

        if self.transition_fn.get(&source).is_some() {
            return Err("The transition already exists");
        }

        self.transition_fn.insert(source, target);

        Ok(())
    }

    #[inline]
    fn get_start_state(&self) -> u32 {
        self.start_state
    }

    fn change_start_state(&mut self, new_start_state: u32) -> Result<(), &'static str> {
        if self.states.contains(&new_start_state) {
            self.start_state = new_start_state;
            Ok(())
        } else {
            Err("The selected start state is not a valid state")
        }
    }

    fn add_accept_state(&mut self, state: u32) -> Result<(), &'static str> {
        if !self.states.contains(&state) {
            return Err("The selected accept state is not a valid state");
        }
        self.accept_states.push(state);
        Ok(())
    }

    fn remove_accept_state(&mut self, state: u32) {
        if let Some(index) = self.accept_states.iter().position(|&e| e == state) {
                self.accept_states.remove(index);
        }
    }

    // funkcija provjerava da li je automat stanja
    // konacan i deterministican
    fn is_complete(&self) -> bool {
        for state in self.states.iter() {
            for symbol in &self.alphabet {
                if !self.transition_fn.contains_key(&(*state, *symbol)) {
                    return false;
                }
            }
        }

        true
    }

    // obracunava input i vraca 'true' ako zavrsi u finalnom stanju
    pub fn run(&self, input: &str) -> Result<bool, &'static str> {
        let mut current_state = self.states.first().unwrap();

        for c in input.chars() {
            match self.transition_fn.get(&(*current_state, c)) {
                Some(state) => current_state = state,
                None => return Err("The automata is incomplete"),
            }
            current_state = self.transition_fn.get(&(*current_state, c)).unwrap();
        }

        Ok(self.accept_states.contains(current_state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dfa_creation() {
        let dfa = Dfa::new();

        assert_eq!(vec![0], dfa.states);
        assert_eq!(0, dfa.start_state);
    }

}
