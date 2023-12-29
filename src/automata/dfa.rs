// use crate::automata::iters::*;
use crate::automata::traits::*;
use std::collections::HashMap;
use std::default::Default;

#[allow(dead_code)] // TODO: remove
#[derive(Debug)]
pub struct Dfa {
    states: Vec<u32>,
    alphabet: Vec<char>,
    transition_fn: HashMap<(u32, char), u32>,
    accept_states: Vec<u32>,
}

impl Default for Dfa {
    fn default() -> Self {
        Dfa {
            states: vec![0],
            alphabet: Vec::new(),
            transition_fn: HashMap::new(),
            accept_states: Vec::new(),
        }
    }
}

impl State for Dfa {
    fn add_state(&mut self) {
        self.states.push(self.states.last().unwrap() + 1)
    }

    fn remove_state(&mut self) {
        if self.states.len() == 1 {
            return;
        }

        let target = self.states.pop().unwrap();

        if let Some(index) = self.accept_states.iter().position(|&e| e == target) {
            self.accept_states.remove(index);
        }

        // uklanja iz tabele svaku tranziciju u kojoj se pojavljuje taget kao trenutno ili buduce
        // stanje
        self.transition_fn
            .retain(|&(curr_state, _), new_state| curr_state != target && *new_state != target);
    }

    fn add_accept_state(&mut self, state: u32) {
        if self.states.contains(&state) {
            self.accept_states.push(state);
            self.accept_states.sort();
        }
    }

    fn remove_accept_state(&mut self, state: u32) {
        if let Some(index) = self.accept_states.iter().position(|&e| e == state) {
            self.accept_states.remove(index);
        }
    }
}

impl Alphabet for Dfa {
    fn add_symbol(&mut self, symbol: char) {
        if !self.alphabet.contains(&symbol) {
            self.alphabet.push(symbol);
        }
    }

    fn remove_symbol(&mut self, symbol: char) {
        if let Some(position) = self.alphabet.iter().position(|e| *e == symbol) {
            self.alphabet.remove(position);
        }

        self.transition_fn
            .retain(|&(_, transition_symbol), _| transition_symbol != symbol);
    }
}

#[allow(dead_code)] // TODO: remove
impl Dfa {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    fn start_state(&self) -> u32 {
        *self.states.first().unwrap()
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

    /// Checks if self is complete.
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

    /// Processes the given string and returns Ok(true) if it
    /// ends up in an accept state.
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
