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

    /// Constructor returns a minimized dfa from a given regex
    pub fn from(regex: &str) -> Result<Self, &'static str> {
        let mut dfa = Nfa::from(regex)?.to_dfa();
        dfa.minimize()?;
        Ok(dfa)
    }

    #[inline]
    fn start_state(&self) -> u32 {
        *self.states.first().unwrap()
    }

    fn is_accept_state(&self, state: u32) -> bool {
        self.accept_states.contains(&state)
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

    /// Determines if the two sets are n-equivalent to each other.
    /// Returns true if they are, and false if they are not.
    fn are_equivalent(&self, sets: &[Vec<u32>], first: u32, second: u32) -> bool {
        // first find the set indexes for the first and second state
        let first_id = sets.iter().position(|s| s.contains(&first)).unwrap();
        let second_id = sets.iter().position(|s| s.contains(&second)).unwrap();

        if first_id != second_id {
            return false;
        }

        for &symbol in self.alphabet.iter() {
            let first_transition_id = sets
                .iter()
                .position(|set| set.contains(self.transition_fn.get(&(first, symbol)).unwrap()))
                .unwrap();
            let second_transition_id = sets
                .iter()
                .position(|set| set.contains(self.transition_fn.get(&(second, symbol)).unwrap()))
                .unwrap();

            if first_transition_id != second_transition_id {
                return false;
            }
        }

        true
    }

    pub fn minimize(&mut self) -> Result<(), &'static str> {
        // create two equivalence sets,
        // one for final, and one for non final states
        let mut old_eqvl = vec![self.accept_states.clone()];
        old_eqvl.push(
            self.states
                .iter()
                .filter(|e| !self.accept_states.contains(e))
                .cloned()
                .collect(),
        );
        let mut done = false;

        while !done {
            let mut new_eqvl: Vec<Vec<u32>> = Vec::new();
            // if a modification gets done, set this to false
            done = true;

            for set in &old_eqvl {
                for state_idx in 0..set.len() {
                    let current_state = set[state_idx];
                    let mut eq_found = false;
                    // if the state is first in it's set,
                    // add it to the new equivalence in a new set
                    if state_idx == 0 {
                        new_eqvl.push(vec![set[state_idx]]);
                        continue;
                    }

                    // check for equivalence with all prior states in the set
                    for i in 0..state_idx {
                        let comparison_state = set[i];

                        // if the states are equivalent
                        if self.are_equivalent(&old_eqvl, comparison_state, current_state) {
                            // find the new equivalence set that contains the comparison state
                            // and add the current state to that set
                            new_eqvl
                                .iter_mut()
                                .find(|set| set.contains(&comparison_state))
                                .unwrap()
                                .push(current_state);

                            eq_found = true;
                            break;
                        } else {
                            done = false;
                        }
                    }

                    // if the state is not equivalent with any states in it's set,
                    // create a new set for it and ad it there
                    if !eq_found {
                        new_eqvl.push(vec![current_state]);
                    }
                }
            }
            // replace the old equivalence set with the new one
            old_eqvl = new_eqvl;
        }

        let mut dfa = Dfa::new();
        dfa.alphabet = self.alphabet.clone();

        // lookup table used for translating state sets to states
        let mut lookup_table: HashMap<&Vec<u32>, u32> = HashMap::new();
        for (index, set) in old_eqvl.iter().enumerate() {
            lookup_table.insert(set, index as u32);
            dfa.add_state();

            if set.iter().any(|&state| self.is_accept_state(state)) {
                dfa.add_accept_state(index as u32);
            }
        }
        dfa.remove_state();

        for set in &old_eqvl {
            let state = set.first().unwrap();

            for &symbol in self.alphabet.iter() {
                let dest = self.transition_fn.get(&(*state, symbol)).unwrap();

                let dest_set = old_eqvl.iter().find(|set| set.contains(dest)).unwrap();
                let dest_set = lookup_table.get(&dest_set).unwrap();
                let source_set = lookup_table.get(&set).unwrap();

                dfa.add_transition(&(*source_set, symbol), *dest_set)?;
            }
        }
        *self = dfa;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_equivalence() {
        let dfa = Dfa::from("a|(ab|b)*").unwrap();
        let sets = vec![vec![0, 1, 2, 4], vec![3, 5]];

        assert!(dfa.are_equivalent(&sets, 1, 2));
        assert!(dfa.are_equivalent(&sets, 2, 4));
        assert!(dfa.are_equivalent(&sets, 1, 4));

        assert!(!dfa.are_equivalent(&sets, 0, 1));
        assert!(!dfa.are_equivalent(&sets, 0, 2));
        assert!(!dfa.are_equivalent(&sets, 0, 4));
        assert!(!dfa.are_equivalent(&sets, 3, 5));
        assert!(!dfa.are_equivalent(&sets, 0, 3));
        assert!(!dfa.are_equivalent(&sets, 0, 5));
        assert!(!dfa.are_equivalent(&sets, 1, 3));
        assert!(!dfa.are_equivalent(&sets, 1, 5));
        assert!(!dfa.are_equivalent(&sets, 2, 3));
        assert!(!dfa.are_equivalent(&sets, 4, 5));
    }

    #[test]
    fn dfa_minimization() {
        let mut dfa = Dfa::from("a|(ab|b)*").unwrap();
        let _ = dfa.minimize();

        // Dfa {
        //     states: [0, 1, 2, 3, 4],
        //     alphabet: ['a', 'b'],
        //     transition_fn: {
        //         (0, 'a'):1,
        //         (0, 'b'): 2,
        //         (1, 'a'): 3,
        //         (1, 'b'): 2,
        //         (2, 'a'): 4,
        //         (2, 'b'): 2,
        //         (3, 'a'): 3,
        //         (3, 'b'): 3,
        //         (4, 'a'): 3,
        //         (4, 'b'): 2
        //     },
        //     accept_states: [0, 1, 2]
        // }
        assert!(dfa.is_complete());
        assert_eq!(vec![0, 1, 2, 3, 4], dfa.states);
        assert_eq!(vec!['a', 'b'], dfa.alphabet);
        assert_eq!(vec![0, 1, 2], dfa.accept_states);
        assert_eq!(1, *dfa.transition_fn.get(&(0, 'a')).unwrap());
        assert_eq!(2, *dfa.transition_fn.get(&(0, 'b')).unwrap());
        assert_eq!(3, *dfa.transition_fn.get(&(1, 'a')).unwrap());
        assert_eq!(2, *dfa.transition_fn.get(&(1, 'b')).unwrap());
        assert_eq!(4, *dfa.transition_fn.get(&(2, 'a')).unwrap());
        assert_eq!(2, *dfa.transition_fn.get(&(2, 'b')).unwrap());
        assert_eq!(3, *dfa.transition_fn.get(&(3, 'a')).unwrap());
        assert_eq!(3, *dfa.transition_fn.get(&(3, 'b')).unwrap());
        assert_eq!(3, *dfa.transition_fn.get(&(4, 'a')).unwrap());
        assert_eq!(2, *dfa.transition_fn.get(&(4, 'b')).unwrap());
    }
}
