// use crate::automata::iters::*;
use crate::automata::iters::*;
use crate::automata::traits::*;
use crate::nfa::Nfa;
use std::collections::HashMap;
use std::default::Default;
mod conversion;

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

impl Transition for Dfa {
    fn add_transition(&mut self, source: &(u32, char), target: u32) -> Result<(), &'static str> {
        if !self.states.contains(&source.0) {
            return Err("The source state isn't a valid state");
        }

        if !self.states.contains(&target) {
            return Err("The target state isn't a valid state");
        }

        if !self.alphabet.contains(&source.1) {
            return Err("The symbol is not in the alphabet");
        }

        if self.transition_fn.get(source).is_some() {
            return Err("The transition already exists");
        }

        self.transition_fn.insert(*source, target);

        Ok(())
    }
    //
    // fn remove_transition(&mut self, source: &(u32, char), target: u32) {
    //     self.transition_fn.remove(source);
    // }
}

impl StateIter for Dfa {
    fn is_empty(&self) -> bool {
        self.states.len() == 1
    }

    #[inline]
    fn states_iter(&self) -> impl Iterator<Item = &u32> {
        self.states.iter()
    }

    #[inline]
    fn states_iter_mut(&mut self) -> impl Iterator<Item = &mut u32> {
        self.states.iter_mut()
    }

    #[inline]
    fn accept_states_iter(&self) -> impl Iterator<Item = &u32> {
        self.accept_states.iter()
    }

    #[inline]
    fn accept_states_iter_mut(&mut self) -> impl Iterator<Item = &mut u32> {
        self.accept_states.iter_mut()
    }
}

impl TransitionIter for Dfa {
    type Target = u32;

    #[inline]
    fn transitions_iter(&self) -> impl Iterator<Item = (&(u32, char), &u32)> {
        self.transition_fn.iter()
    }

    #[inline]
    fn transitions_iter_mut(&mut self) -> impl Iterator<Item = (&(u32, char), &mut u32)> {
        self.transition_fn.iter_mut()
    }

    fn get_transition(&self, key: (u32, char)) -> Option<&u32> {
        self.transition_fn.get(&key)
    }
}

#[allow(dead_code)] // TODO: remove
impl Dfa {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from(regex: &str) -> Result<Self, &'static str> {
        let nfa = Nfa::from(regex)?;
        Ok(nfa.to_dfa())
    }

    #[inline]
    fn start_state(&self) -> u32 {
        *self.states.first().unwrap()
    }

    /// Checks if self is complete.
    pub fn is_complete(&self) -> bool {
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

    fn minimize(&mut self) {
        // create two equivalence sets,
        // one for final, and one for non final states
        let mut sets = vec![self.accept_states.clone()];
        sets.push(
            self.states
                .iter()
                .filter(|e| !self.accept_states.contains(e))
                .cloned()
                .collect(),
        );

        todo!();
    }
}
