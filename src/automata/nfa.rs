mod operators;

use crate::util;
use crate::language::EMPTY_STRING;
use std::collections::HashMap;
use std::default::Default;

#[allow(dead_code)] // TODO: remove
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

#[allow(dead_code)] // TODO: remove
impl Nfa {
    pub fn from(expression: &str) -> Result<Self, &'static str> {
        util::check_for_correctness(expression)?;

        enum OnReturn {
            Union,
            Bracket,
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
                            let root = new_last.0.last_added_state();
                            operators::concat(&mut new_last.0, last.0, root)?;
                            break;
                        }
                        OnReturn::Finished => return Ok(last.0.clone()),
                    }
                },
                '*' => {
                    operators::kleene_star(&mut current.0)?;
                }
                '|' => {
                    stack.push((Nfa::new(), OnReturn::Union));
                }
                alpha => {
                    current.0.add_symbol(alpha);
                    current.0.push_symbol(alpha)?;
                }
            }
        }

        loop {
            let current = stack.pop();

            if current.is_none() {
                break;
            }
            let current = current.unwrap();

            match current.1 {
                OnReturn::Bracket => {
                    return Err(err_msg);
                }
                OnReturn::Union => {
                    let new_last = stack.last_mut().expect(err_msg);
                    operators::union(&mut new_last.0, current.0)?;
                }
                OnReturn::Finished => return Ok(current.0),
            }
        }

        Err(err_msg)
    }

    fn new() -> Self {
        Nfa::default()
    }

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

    #[inline]
    pub fn states_iter(&self) -> impl Iterator<Item = &u32> {
        self.states.iter()
    }

    #[inline]
    pub fn states_iter_mut(&mut self) -> impl Iterator<Item = &mut u32> {
        self.states.iter_mut()
    }

    #[inline]
    pub fn alphabet_iter(&self) -> impl Iterator<Item = &char> {
        self.alphabet.iter()
    }

    #[inline]
    pub fn alphabet_iter_mut(&mut self) -> impl Iterator<Item = &mut char> {
        self.alphabet.iter_mut()
    }

    #[inline]
    pub fn transitions_iter(&self) -> impl Iterator<Item = (&(u32, char), &Vec<u32>)> {
        self.transition_fn.iter()
    }

    #[inline]
    pub fn transitions_iter_mut(&mut self) -> impl Iterator<Item = (&(u32, char), &mut Vec<u32>)> {
        self.transition_fn.iter_mut()
    }

    #[inline]
    pub fn accept_states_iter(&self) -> impl Iterator<Item = &u32> {
        self.accept_states.iter()
    }

    #[inline]
    pub fn accept_states_iter_mut(&mut self) -> impl Iterator<Item = &mut u32> {
        self.accept_states.iter_mut()
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

    fn add_transition(&mut self, source: (u32, char), target: u32) -> Result<(), &'static str> {
        if !self.states.contains(&source.0) {
            return Err("Source state is not a valid state");
        }
        if !self.alphabet.contains(&source.1) {
            return Err("Transition symbol is not in the alphabet");
        }
        if !self.states.contains(&target) {
            return Err("Destination state is not a valid state");
        }

        match self.transition_fn.get_mut(&source) {
            Some(destinations) => {
                if !destinations.contains(&target) {
                    destinations.push(target);
                    destinations.sort();
                }
            }
            None => {
                self.transition_fn.insert(source, vec![target]);
            }
        }

        Ok(())
    }

    fn remove_transition(&mut self, source: (u32, char), target: u32) {
        if let Some(destinations) = self.transition_fn.get_mut(&source) {
            if let Some(index) = destinations.iter().position(|&e| e == target) {
                destinations.remove(index);

                if destinations.is_empty() {
                    self.transition_fn.remove(&source);
                }
            }
        }
    }

    fn get_transition(&self, key: (u32, char)) -> Option<&Vec<u32>> {
        self.transition_fn.get(&key)
    }

    fn add_accept_state(&mut self, state: u32) {
        if self.states.contains(&state) {
            self.accept_states.push(state);
            self.accept_states.sort();
        }
    }

    fn remove_accept_state(&mut self, target: u32) {
        if self.accept_states.contains(&target) {
            if let Some(index) = self.accept_states.iter().position(|e| *e == target) {
                self.accept_states.remove(index);
            }
        }
    }

    // metoda inkrementuje nazive stanja za neki broj increment
    fn increment_states(&mut self, increment: u32) {
        let mut lookup_table: HashMap<(u32, char), Vec<u32>> = HashMap::new();

        // inkrementuj imena u skupovima stanja i finalnih stanja
        self.states.iter_mut().for_each(|e| *e += increment);
        self.accept_states.iter_mut().for_each(|e| *e += increment);

        // iskopiraj trenutnu tabelu tranzicija u privremenu lookup tablue
        // (zbog borrow checker-a)
        // u novoj tabeli uvecaj stanja u k,v parovima za inkrement
        self.transition_fn.iter().for_each(|(key, value)| {
            lookup_table.insert((key.0 + increment, key.1), value.clone());
        });

        for entry in lookup_table.iter_mut() {
            entry.1.iter_mut().for_each(|e| *e += increment);
        }
        // pomjeri lookup tabelu u svoju tablue
        self.transition_fn = lookup_table;
    }

    fn push_symbol(&mut self, symbol: char) -> Result<(), &'static str> {
        if !self.alphabet.contains(&symbol) {
            return Err("The symbol is not in the alphabet");
        }

        let prev_last = self.last_added_state();
        self.add_state();
        let new_last = self.last_added_state();
        self.add_transition((prev_last, symbol), new_last)?;
        self.accept_states.clear();
        self.add_accept_state(new_last);

        Ok(())
    }

    // pub fn to_dfa(&self) -> Dfa {
    //     let dfa = Dfa::new();
    //
    //     let powerset = util::create_power_set(&self.states);
    //
    //     dfa
    // }
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

        nfa.add_transition((0, 'A'), 0).unwrap();
        nfa.add_transition((0, 'A'), 1).unwrap();
        nfa.add_transition((0, 'B'), 3).unwrap();
        nfa.add_transition((0, 'C'), 1).unwrap();
        nfa.add_transition((1, 'C'), 2).unwrap();
        nfa.add_transition((2, 'B'), 3).unwrap();
        nfa.add_transition((2, 'B'), 1).unwrap();

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
        first.add_transition((0, 'a'), 1).unwrap();
        first.add_transition((1, EMPTY_STRING), 2).unwrap();
        first.add_transition((2, 'b'), 3).unwrap();
        first.add_accept_state(3);

        second.add_state();
        second.add_symbol('a');
        second.add_transition((0, 'a'), 1).unwrap();
        second.add_accept_state(1);

        (first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;

    #[test]
    fn nfa_construction() {
        // regex: "a|(ab|b)*"
        // {
        //   states: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        //   alphabet: ['\0', 'a', 'b'],
        //   transition_fn: {
        //     (8, '\0'): [4],
        //     (4, '\0'): [5],
        //     (3, '\0'): [4],
        //     (0, '\0'): [1, 3],
        //     (5, '\0'): [6, 9],
        //     (6, 'a'): [7],
        //     (9, 'b'): [10],
        //     (7, 'b'): [8],
        //     (1, 'a') : [2],
        //     (10, '\0'): [4]}
        //     accept_states: [2, 3, 8, 10]
        // }
        let nfa = Nfa::from("a|(ab|b)*");

        let nfa = match nfa {
            Ok(result) => result,
            Err(e) => panic!("{}", e),
        };

        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10], nfa.states);
        assert_eq!(vec![EMPTY_STRING, 'a', 'b'], nfa.alphabet);
        assert_eq!(vec![2, 3, 8, 10], nfa.accept_states);

        let mut map: HashMap<(u32, char), Vec<u32>> = HashMap::new();
        map.insert((10, EMPTY_STRING), vec![4]);
        map.insert((8, EMPTY_STRING), vec![4]);
        map.insert((4, EMPTY_STRING), vec![5]);
        map.insert((3, EMPTY_STRING), vec![4]);
        map.insert((0, EMPTY_STRING), vec![1, 3]);
        map.insert((5, EMPTY_STRING), vec![6, 9]);
        map.insert((6, 'a'), vec![7]);
        map.insert((9, 'b'), vec![10]);
        map.insert((7, 'b'), vec![8]);
        map.insert((1, 'a'), vec![2]);

        for (key, value) in map.iter() {
            if nfa.transition_fn.get(key).unwrap() != value {
                panic!("Transition is missing");
            }
        }

        let pairs = [
            (0, 'a'),
            (0, 'b'),
            (1, EMPTY_STRING),
            (1, 'b'),
            (2, EMPTY_STRING),
            (2, 'a'),
            (2, 'b'),
            (3, 'a'),
            (3, 'b'),
            (4, 'a'),
            (4, 'b'),
            (5, 'a'),
            (5, 'b'),
            (6, EMPTY_STRING),
            (6, 'b'),
            (7, EMPTY_STRING),
            (7, 'a'),
            (8, 'a'),
            (8, 'b'),
            (9, EMPTY_STRING),
            (9, 'a'),
            (10, 'a'),
            (10, 'b'),
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

    #[test]
    fn nfa_removing_transition() {
        let mut nfa = test_utils::prepare_nfa();

        nfa.remove_transition((0, 'A'), 1);
        nfa.remove_transition((1, 'C'), 2);
        nfa.remove_transition((2, 'B'), 1);

        assert_eq!(vec![0], *nfa.transition_fn.get(&(0, 'A')).unwrap());
        assert_eq!(Option::None, nfa.transition_fn.get(&(1, 'C')));
        assert_eq!(vec![3], *nfa.transition_fn.get(&(2, 'B')).unwrap());
    }

    #[test]
    fn nfa_adding_accept_states() {
        let mut nfa = test_utils::prepare_nfa();

        nfa.add_accept_state(1);
        nfa.add_accept_state(2);

        assert_eq!(vec![1, 2, 3], nfa.accept_states);
    }
}
