mod operators;

use super::dfa::Dfa;
pub use crate::automata::iters::*;
use crate::automata::traits::*;
use crate::language::EMPTY_STRING;
use crate::util;
use std::collections::{HashMap, VecDeque};
use std::default::Default;

#[derive(Debug, Clone)]
pub struct Nfa {
    states: Vec<u32>,
    alphabet: Vec<char>,
    transition_fn: HashMap<(u32, char), Vec<u32>>,
    accept_states: Vec<u32>,
}

impl Default for Nfa {
    fn default() -> Self {
        Nfa {
            states: vec![0],
            alphabet: vec![EMPTY_STRING],
            transition_fn: HashMap::new(),
            accept_states: Vec::new(),
        }
    }
}

impl State for Nfa {
    fn add_state(&mut self) {
        let last = self.states.last().unwrap();
        self.states.push(last + 1);
    }

    fn remove_state(&mut self) {
        if *self.states.last().unwrap() == 0 {
            return;
        }
        self.states.pop();
    }

    fn add_accept_state(&mut self, state: u32) {
        if self.states.contains(&state) {
            self.accept_states.push(state);
            self.accept_states.sort();
        }
    }

    fn remove_accept_state(&mut self, target: u32) {
        if let Some(index) = self.accept_states.iter().position(|e| *e == target) {
            self.accept_states.remove(index);
        }
    }
}

impl Alphabet for Nfa {
    fn add_symbol(&mut self, symbol: char) {
        if !self.alphabet.contains(&symbol) {
            self.alphabet.push(symbol);
        }
    }

    fn remove_symbol(&mut self, symbol: char) {
        if let Some(position) = self.alphabet.iter().position(|e| *e == symbol) {
            self.alphabet.remove(position);
        }

        self.transition_fn.retain(|&(_, s), _| s != symbol);
    }
}

impl Transition for Nfa {
    fn add_transition(&mut self, source: &(u32, char), target: u32) -> Result<(), &'static str> {
        if !self.states.contains(&source.0) {
            return Err("Source state is not a valid state");
        }
        if !self.alphabet.contains(&source.1) {
            return Err("Transition symbol is not in the alphabet");
        }
        if !self.states.contains(&target) {
            return Err("Destination state is not a valid state");
        }

        match self.transition_fn.get_mut(source) {
            Some(destinations) => {
                if !destinations.contains(&target) {
                    destinations.push(target);
                    destinations.sort();
                }
            }
            None => {
                self.transition_fn.insert(*source, vec![target]);
            }
        }

        Ok(())
    }
    //
    // fn remove_transition(&mut self, source: &(u32, char), target: u32) {
    //     if let Some(destinations) = self.transition_fn.get_mut(source) {
    //         if let Some(index) = destinations.iter().position(|&e| e == target) {
    //             destinations.remove(index);
    //
    //             if destinations.is_empty() {
    //                 self.transition_fn.remove(source);
    //             }
    //         }
    //     }
    // }
}

impl TransitionIter for Nfa {
    type Target = Vec<u32>;

    #[inline]
    fn transitions_iter(&self) -> impl Iterator<Item = (&(u32, char), &Vec<u32>)> {
        self.transition_fn.iter()
    }

    #[inline]
    fn transitions_iter_mut(&mut self) -> impl Iterator<Item = (&(u32, char), &mut Vec<u32>)> {
        self.transition_fn.iter_mut()
    }

    fn get_transition(&self, key: (u32, char)) -> Option<&Vec<u32>> {
        self.transition_fn.get(&key)
    }
}

impl Nfa {
    pub fn from(expression: &str) -> Result<Self, &'static str> {
        util::check_for_correctness(expression)?;

        enum OnReturn {
            Union,
            Bracket,
            Concat,
            Finished,
        }

        let mut stack: Vec<(Nfa, OnReturn)> = vec![(Nfa::new(), OnReturn::Finished)];
        let err_msg = "Regex is not in the correct form";

        for token in expression.chars() {
            let current = stack.last_mut().unwrap();

            match token {
                '(' => {
                    stack.push((Nfa::new(), OnReturn::Bracket));
                }
                ')' => loop {
                    let last = stack.pop().expect(err_msg);
                    match last.1 {
                        OnReturn::Union => {
                            let new_last = stack.last_mut().expect(err_msg);
                            operators::union(&mut new_last.0, last.0)?;
                        }
                        OnReturn::Bracket => {
                            let new_last = stack.last_mut().expect(err_msg);
                            operators::concat(&mut new_last.0, last.0)?;
                            break;
                        }
                        OnReturn::Concat => {
                            let new_last = stack.last_mut().expect(err_msg);
                            operators::concat(&mut new_last.0, last.0)?;
                        }
                        OnReturn::Finished => return Ok(last.0.clone()),
                    }
                },
                '*' => {
                    operators::kleene_star(&mut current.0)?;
                }
                '|' => {
                    // first concatenate previous nfa's on the stack
                    // if they should be concatnated
                    // and then push a new one to the stack
                    loop {
                        let last = stack.pop().expect(err_msg);
                        match last.1 {
                            OnReturn::Concat => {
                                let previous = stack.last_mut().expect(err_msg);
                                operators::concat(&mut previous.0, last.0)?;
                            }
                            _ => {
                                stack.push(last);
                                break;
                            }
                        }
                    }
                    stack.push((Nfa::new(), OnReturn::Union));
                }
                alpha => {
                    let mut new = Nfa::new();
                    new.add_symbol(alpha);
                    new.push_symbol(alpha)?;
                    stack.push((new, OnReturn::Concat));
                }
            }
        }

        while let Some(current) = stack.pop() {
            match current.1 {
                OnReturn::Bracket => {
                    return Err(err_msg);
                }
                OnReturn::Union => {
                    let new_last = stack.last_mut().expect(err_msg);
                    operators::union(&mut new_last.0, current.0)?;
                }
                OnReturn::Concat => {
                    let new_last = stack.last_mut().expect(err_msg);
                    operators::concat(&mut new_last.0, current.0)?;
                }
                OnReturn::Finished => return Ok(current.0),
            }
        }

        Err(err_msg)
    }

    fn new() -> Self {
        Nfa::default()
    }

    #[inline]
    pub fn last_added_state(&self) -> u32 {
        *self.states.last().unwrap()
    }

    #[inline]
    pub fn start_state(&self) -> u32 {
        *self.states.first().unwrap()
    }

    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// Increments state names by a given increment
    fn increment_states(&mut self, increment: u32) {
        let mut lookup_table: HashMap<(u32, char), Vec<u32>> = HashMap::new();

        // increment names in the states and accept states sets
        self.states.iter_mut().for_each(|e| *e += increment);
        self.accept_states.iter_mut().for_each(|e| *e += increment);

        // copy the transition function in to a temporary lookup table
        // (because of the borrow checker)
        // and increement them by increemetn
        self.transition_fn.iter().for_each(|(key, value)| {
            lookup_table.insert((key.0 + increment, key.1), value.clone());
        });

        for entry in lookup_table.iter_mut() {
            entry.1.iter_mut().for_each(|e| *e += increment);
        }
        // move the lookup table to it's own transition fn
        self.transition_fn = lookup_table;
    }

    fn is_accept_state(&self, state: u32) -> bool {
        self.accept_states.contains(&state)
    }

    fn push_symbol(&mut self, symbol: char) -> Result<(), &'static str> {
        if !self.alphabet.contains(&symbol) {
            return Err("The symbol is not in the alphabet");
        }

        let prev_last = self.last_added_state();
        self.add_state();
        let new_last = self.last_added_state();

        if self.accept_states.is_empty() {
            self.add_transition(&(prev_last, symbol), new_last)?;
        } else {
            for state in self.accept_states.clone() {
                self.add_transition(&(state, symbol), new_last)?;
            }
            self.accept_states.clear();
        }
        self.add_accept_state(new_last);

        Ok(())
    }

    pub fn to_dfa(&self) -> Dfa {
        let mut dfa = Dfa::new();
        let mut queue: VecDeque<Vec<u32>> = VecDeque::new();
        let mut states: Vec<Vec<u32>> = Vec::new();
        let mut transitions: HashMap<(Vec<u32>, char), Vec<u32>> = HashMap::new();
        let alphabet: Vec<char> = self
            .alphabet
            .iter()
            .filter(|&s| *s != EMPTY_STRING)
            .cloned()
            .collect();

        // add the epsilon clojure of the start state to the new_states and the queue
        states.push(util::state_epsilon_clojure(self, self.start_state()));
        queue.push_back(states.first().unwrap().clone());

        while let Some(current) = queue.pop_front() {
            // for every symbol in the alphabet
            for symbol in alphabet.iter() {
                // get every state that can be transitioned to
                // from the current set of states
                // and calculate an epsilon clojure on it
                let new_tr = util::set_transitions(self, &current, *symbol);
                let new_tr = util::set_epsilon_clojure(self, &new_tr);

                // insert the transition for it
                transitions.insert((current.clone(), *symbol), new_tr.clone());

                // if it's a new state, add it to the set of state sets
                // and push it to the queue
                if !states.contains(&new_tr) {
                    queue.push_back(new_tr.clone());
                    states.push(new_tr);
                }
            }
        }

        // lookup table used for translating state sets to states
        let mut lookup_table: HashMap<&Vec<u32>, u32> = HashMap::new();
        for (index, state) in states.iter().enumerate() {
            lookup_table.insert(state, index as u32);
            dfa.add_state();

            if state.iter().any(|&state| self.is_accept_state(state)) {
                dfa.add_accept_state(index as u32);
            }
        }
        dfa.remove_state();

        // add the alphabet to the dfa
        alphabet.iter().for_each(|&s| dfa.add_symbol(s));

        // add transitions to the dfa
        for ((source, symbol), destination) in transitions.iter() {
            let source = lookup_table.get(source).unwrap();
            let destination = lookup_table.get(destination).unwrap();

            dfa.add_transition(&(*source, *symbol), *destination)
                .unwrap();
        }

        dfa
    }
}

pub mod test_utils {
    use super::*;

    pub fn prepare_nfa() -> Nfa {
        let mut nfa = Nfa::new();

        nfa.add_state();
        nfa.add_state();
        nfa.add_state();

        nfa.add_symbol('A');
        nfa.add_symbol('B');
        nfa.add_symbol('C');

        nfa.add_transition(&(0, 'A'), 0).unwrap();
        nfa.add_transition(&(0, 'A'), 1).unwrap();
        nfa.add_transition(&(0, 'B'), 3).unwrap();
        nfa.add_transition(&(0, 'C'), 1).unwrap();
        nfa.add_transition(&(1, 'C'), 2).unwrap();
        nfa.add_transition(&(2, 'B'), 3).unwrap();
        nfa.add_transition(&(2, 'B'), 1).unwrap();

        nfa.add_accept_state(3);

        nfa
    }

    pub fn prepare_nfa_pair() -> (Nfa, Nfa) {
        let mut first = Nfa::new();
        let mut second = Nfa::new();

        first.add_state();
        first.add_state();
        first.add_state();
        first.add_symbol('a');
        first.add_symbol('b');
        first.add_transition(&(0, 'a'), 1).unwrap();
        first.add_transition(&(1, EMPTY_STRING), 2).unwrap();
        first.add_transition(&(2, 'b'), 3).unwrap();
        first.add_accept_state(3);

        second.add_state();
        second.add_symbol('a');
        second.add_transition(&(0, 'a'), 1).unwrap();
        second.add_accept_state(1);

        (first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfa_construction() {
        // regex: "a|(ab|b)*
        // {
        //     states: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
        //     alphabet: ['\0', 'a', 'b'],
        //     transition_fn: {
        //         (1, '\0'): [2],
        //         (2, 'a'): [3],
        //         (4, '\0'): [5],
        //         (0, '\0'): [1, 4],
        //         (5, '\0'): [6],
        //         (6, '\0'): [7, 12],
        //         (13, 'b'): [14],
        //         (11, '\0'): [5],
        //         (9, '\0'): [10],
        //         (14, '\0'): [5],
        //         (12, '\0'): [13],
        //         (10, 'b'): [11],
        //         (7, '\0'): [8],
        //         (8, 'a'): [9]
        //     },
        //     accept_states: [3, 4, 11, 14]
        // }
        let nfa = Nfa::from("a|(ab|b)*");
        println!("{:?}", nfa);

        let nfa = match nfa {
            Ok(result) => result,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14], nfa.states);
        assert_eq!(vec![EMPTY_STRING, 'a', 'b'], nfa.alphabet);
        assert_eq!(vec![3, 4, 11, 14], nfa.accept_states);

        let mut map: HashMap<(u32, char), Vec<u32>> = HashMap::new();
        map.insert((0, EMPTY_STRING), vec![1, 4]);
        map.insert((1, EMPTY_STRING), vec![2]);
        map.insert((2, 'a'), vec![3]);
        map.insert((4, EMPTY_STRING), vec![5]);
        map.insert((5, EMPTY_STRING), vec![6]);
        map.insert((6, EMPTY_STRING), vec![7, 12]);
        map.insert((7, EMPTY_STRING), vec![8]);
        map.insert((8, 'a'), vec![9]);
        map.insert((9, EMPTY_STRING), vec![10]);
        map.insert((10, 'b'), vec![11]);
        map.insert((11, EMPTY_STRING), vec![5]);
        map.insert((12, EMPTY_STRING), vec![13]);
        map.insert((13, 'b'), vec![14]);
        map.insert((14, EMPTY_STRING), vec![5]);

        for (key, value) in map.iter() {
            if nfa.transition_fn.get(key).unwrap() != value {
                panic!("Transition is missing");
            }
        }

        let pairs = [
            (0, 'a'),
            (0, 'b'),
            (1, 'a'),
            (1, 'b'),
            (2, EMPTY_STRING),
            (2, 'b'),
            (3, EMPTY_STRING),
            (3, 'a'),
            (3, 'b'),
            (4, 'a'),
            (4, 'b'),
            (5, 'a'),
            (5, 'b'),
            (6, 'a'),
            (6, 'b'),
            (7, 'a'),
            (7, 'b'),
            (8, EMPTY_STRING),
            (8, 'b'),
            (9, 'a'),
            (9, 'b'),
            (10, 'a'),
            (10, EMPTY_STRING),
            (11, 'a'),
            (11, 'b'),
            (12, 'a'),
            (12, 'b'),
            (13, EMPTY_STRING),
            (13, 'a'),
            (14, 'a'),
            (14, 'b'),
        ];

        for i in pairs {
            if nfa.transition_fn.get(&i).is_some() {
                panic!("Some transitions exist that shouldn't");
            }
        }
    }

    #[test]
    fn nfa_adding_state() {
        let nfa = test_utils::prepare_nfa();

        assert_eq!(vec![0, 1, 2, 3], nfa.states);
    }

    #[test]
    fn nfa_removing_state() {
        let mut nfa = test_utils::prepare_nfa();

        nfa.remove_state();
        nfa.remove_state();

        assert_eq!(vec![0, 1], nfa.states);
    }

    #[test]
    fn nfa_adding_symbol() {
        let nfa = test_utils::prepare_nfa();

        assert_eq!(vec![EMPTY_STRING, 'A', 'B', 'C'], nfa.alphabet);
    }

    #[test]
    fn nfa_removing_symbol() {
        let mut nfa = test_utils::prepare_nfa();

        nfa.remove_symbol('B');

        assert_eq!(vec![EMPTY_STRING, 'A', 'C'], nfa.alphabet);
    }

    #[test]
    fn nfa_adding_transition() {
        let nfa = test_utils::prepare_nfa();

        assert_eq!(vec![0, 1], *nfa.transition_fn.get(&(0, 'A')).unwrap());
        assert_eq!(vec![3], *nfa.transition_fn.get(&(0, 'B')).unwrap());
        assert_eq!(vec![1], *nfa.transition_fn.get(&(0, 'C')).unwrap());
        assert_eq!(vec![2], *nfa.transition_fn.get(&(1, 'C')).unwrap());
        assert_eq!(vec![1, 3], *nfa.transition_fn.get(&(2, 'B')).unwrap());
    }

    // #[test]
    // fn nfa_removing_transition() {
    //     let mut nfa = test_utils::prepare_nfa();
    //
    //     nfa.remove_transition(&(0, 'A'), 1);
    //     nfa.remove_transition(&(1, 'C'), 2);
    //     nfa.remove_transition(&(2, 'B'), 1);
    //
    //     assert_eq!(vec![0], *nfa.transition_fn.get(&(0, 'A')).unwrap());
    //     assert_eq!(Option::None, nfa.transition_fn.get(&(1, 'C')));
    //     assert_eq!(vec![3], *nfa.transition_fn.get(&(2, 'B')).unwrap());
    // }

    #[test]
    fn nfa_adding_accept_states() {
        let mut nfa = test_utils::prepare_nfa();

        nfa.add_accept_state(1);
        nfa.add_accept_state(2);

        assert_eq!(vec![1, 2, 3], nfa.accept_states);
    }

    #[test]
    fn nfa_to_dfa() {
        let nfa = Nfa::from("a|(ab|b)*").unwrap();
        let dfa = nfa.to_dfa();

        println!("{:?}", dfa);
        assert!(dfa.is_complete());
    }
}
