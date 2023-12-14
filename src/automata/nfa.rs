use std::collections::HashMap;
use std::default::Default;

#[derive(Debug)]
pub struct Nfa {
    states: Vec<u32>,
    alphabet: Vec<char>,
    transition_fn: HashMap<(u32, char), Vec<u32>>,
    start_state: u32,
    accept_states: Vec<u32>,
}

impl Default for Nfa {
    fn default() -> Self {
        Nfa {
            states: vec![0],
            alphabet: vec!['\0'],
            transition_fn: HashMap::new(),
            start_state: 0,
            accept_states: Vec::new(),
        }
    }
}

impl Nfa {
    pub fn new() -> Self {
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

    fn add_accept_state(&mut self, state: u32) {
        if self.states.contains(&state) {
            self.accept_states.push(state);
        }
    }

    fn remove_accept_state(&mut self, target: u32) {
        if self.accept_states.contains(&target) {
            if let Some(index) = self.accept_states.iter().position(|e| *e == target) {
                self.accept_states.remove(index);
            }
        }
    }

    fn concat(&mut self, other: & Self) -> Result<(), &'static str> {
        let increment = *self.states.last().unwrap() + 1;

        for _ in other.states.iter() {
            self.add_state()
        }

        for symbol in other.alphabet.iter() {
            if !self.alphabet.contains(symbol) {
                self.alphabet.push(*symbol);
            }
        }

        for entry in other.transition_fn.iter() {
            for state in entry.1 {
                self.add_transition((entry.0.0 + increment, entry.0.1), *state + increment)?;
            }
        }

        for state in other.accept_states.iter() {
            if !self.accept_states.contains(state) {
                self.accept_states.push(*state);
            }
        }

        self.add_transition((increment -1, '\0'), increment)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepare_nfa() -> Nfa {
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

        nfa
    }

    #[test]
    fn nfa_creation() {
        let nfa = Nfa::new();

        assert_eq!(vec![0], nfa.states, "States set should only contain a '0'");
        assert_eq!(vec!['\0'], nfa.alphabet, "Alphabet should only contain an empty string");
        assert!(
            nfa.transition_fn.is_empty(),
            "Transition fn should be empty"
        );
        assert_eq!(0, nfa.start_state, "Start state should be set to zero");
        assert!(
            nfa.accept_states.is_empty(),
            "Accept states set should be empty"
        );
    }

    #[test]
    fn nfa_adding_state() {
        let nfa = prepare_nfa();

        assert_eq!(vec![0, 1, 2, 3], nfa.states);
    }

    #[test]
    fn nfa_removing_state() {
        let mut nfa = prepare_nfa();

        nfa.remove_state();
        nfa.remove_state();

        assert_eq!(vec![0, 1], nfa.states);
    }

    #[test]
    fn nfa_adding_symbol() {
        let nfa = prepare_nfa();

        assert_eq!(vec!['\0', 'A', 'B', 'C'], nfa.alphabet);
    }

    #[test]
    fn nfa_removing_symbol() {
        let mut nfa = prepare_nfa();

        nfa.remove_symbol('B');

        assert_eq!(vec!['\0', 'A', 'C'], nfa.alphabet);
    }

    #[test]
    fn nfa_adding_transition() {
        let nfa = prepare_nfa();

        assert_eq!(vec![0, 1], *nfa.transition_fn.get(&(0, 'A')).unwrap());
        assert_eq!(vec![3], *nfa.transition_fn.get(&(0, 'B')).unwrap());
        assert_eq!(vec![1], *nfa.transition_fn.get(&(0, 'C')).unwrap());
        assert_eq!(vec![2], *nfa.transition_fn.get(&(1, 'C')).unwrap());
        assert_eq!(vec![1, 3], *nfa.transition_fn.get(&(2, 'B')).unwrap());
    }

    #[test]
    fn nfa_removing_transition() {
        let mut nfa = prepare_nfa();

        nfa.remove_transition((0, 'A'), 1);
        nfa.remove_transition((1, 'C'), 2);
        nfa.remove_transition((2, 'B'), 1);

        assert_eq!(vec![0], *nfa.transition_fn.get(&(0, 'A')).unwrap());
        assert_eq!(Option::None, nfa.transition_fn.get(&(1, 'C')));
        assert_eq!(vec![3], *nfa.transition_fn.get(&(2, 'B')).unwrap());
    }

    #[test]
    fn nfa_adding_accept_states() {
        let mut nfa = prepare_nfa();

        nfa.add_accept_state(1);
        nfa.add_accept_state(2);

        assert_eq!(vec![1, 2], nfa.accept_states);
    }

    #[test]
    fn nfa_concat() {
        let mut first = Nfa::new();
        let mut second = Nfa::new();

        first.add_state();
        first.add_state();
        first.add_symbol('A');
        first.add_symbol('B');
        first.add_transition((0, '\0'), 1).unwrap();
        first.add_transition((0, 'A'), 0).unwrap();
        first.add_transition((0, 'B'), 2).unwrap();
        first.add_transition((1, '\0'), 2).unwrap();
        first.add_transition((1, 'A'), 0).unwrap();
        first.add_transition((2, 'A'), 2).unwrap();
        println!("first: {:?}", first.transition_fn);

        second.add_state();
        second.add_state();
        second.add_symbol('A');
        second.add_symbol('B');
        second.add_transition((0, 'A'), 1).unwrap();
        second.add_transition((1, 'B'), 2).unwrap();
        second.add_transition((2, 'B'), 0).unwrap();
        second.add_transition((2, '\0'), 1).unwrap();
        println!("second: {:?}", second.transition_fn);

        first.concat(&second).expect("The concat method crashed");
        assert_eq!(vec![0, 1, 2, 3, 4, 5], first.states, "The number of states is wrong when concatenating");
        assert_eq!(vec!['\0', 'A', 'B'], first.alphabet, "The alphabet symbols don't match");
        assert_eq!(vec![3], *first.transition_fn.get(&(2, '\0')).unwrap(), "An empty string transition is missing between the nfas");
        assert_eq!(vec![4], *first.transition_fn.get(&(3, 'A')).unwrap());
        assert_eq!(vec![5], *first.transition_fn.get(&(4, 'B')).unwrap());
        assert_eq!(vec![4], *first.transition_fn.get(&(5, '\0')).unwrap());
        assert_eq!(vec![3], *first.transition_fn.get(&(5, 'B')).unwrap());
        println!("result: {:?}", first.transition_fn);
    }
}
