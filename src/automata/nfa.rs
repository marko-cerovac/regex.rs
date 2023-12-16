use std::collections::HashMap;
use std::default::Default;

const EMPTY_STRING: char = '\0';

#[allow(dead_code)] // TODO: remove
#[derive(Debug)]
pub struct Nfa {
    states: Vec<u32>,
    alphabet: Vec<char>,
    transition_fn: HashMap<(u32, char), Vec<u32>>,
    // start_state: u32,
    accept_states: Vec<u32>,
}

impl Default for Nfa {
    fn default() -> Self {
        Nfa {
            states: vec![0],
            alphabet: vec![EMPTY_STRING],
            transition_fn: HashMap::new(),
            // start_state: 0,
            accept_states: Vec::new(),
        }
    }
}

#[allow(dead_code)] // TODO: remove
impl Nfa {
    pub fn construct(regex: &str) -> Self {
        let mut nfa = Nfa::new();

        for token in regex.chars() {
            match token {
                '(' => todo!("Implemet"),
                ')' => todo!("Implemet"),
                '|' => todo!("Implemet"),
                '*' => todo!("Implemet"),
                symbol => {
                    let source_state = nfa.last_added_state();
                    nfa.add_state();
                    nfa.add_symbol(symbol);
                    let target_state = nfa.last_added_state();

                    nfa.add_transition((source_state, symbol), target_state)
                        .unwrap();
                }
            }
        }
        nfa.add_accept_state(nfa.last_added_state());
        nfa
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
    fn last_added_state(&self) -> u32 {
        *self.states.last().unwrap()
    }

    #[inline]
    fn start_state(&self) -> u32 {
        *self.states.first().unwrap()
    }

    fn num_states(&self) -> usize {
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

    // konkatenacija krugog automata stanja na trenutni
    fn concat(&mut self, root: u32, mut other: Self) -> Result<(), &'static str> {
        if !self.states.contains(&root) {
            return Err("Can not add to a state that doesn't exist");
        }

        // dodaj alfabet na svoj
        for symbol in other.alphabet.iter() {
            if !self.alphabet.contains(symbol) {
                self.alphabet.push(*symbol);
            }
        }

        // uvecaj imena njegovih stanja za broj svojih stanja + 1
        let increment = self.last_added_state() + 1;
        other.increment_states(increment);

        // dodaj sebi nova stanja
        for _ in 0..other.num_states() {
            self.add_state()
        }

        // dodaj njegove tranzicije u svoju tabelu
        for entry in other.transition_fn.iter() {
            for state in entry.1 {
                self.add_transition((entry.0 .0, entry.0 .1), *state)?;
            }
        }

        // zaboravi svoja finalna stanja
        // dodaj njegova finalna stanja u svoja
        self.accept_states.clear();
        other
            .accept_states
            .iter()
            .for_each(|&e| self.accept_states.push(e));

        // dodaj tranziciju sa praznim stringom izmedju
        self.add_transition((root, EMPTY_STRING), increment)?;

        Ok(())
    }

    fn kleene_star(&mut self) -> Result<(), &'static str> {
        // dodaj novo finalno stanje na pocetku
        self.increment_states(1);
        self.states.insert(0, 0);
        self.add_accept_state(0);

        // dodaj praznu tranziciju od svih finalnih stanja
        // u prethodno prvo stanje
        let temp = self.accept_states.clone();
        temp.iter().for_each(|e| {
            self.add_transition((*e, EMPTY_STRING), 1).unwrap();
        });

        Ok(())
    }

    fn union(&mut self, mut other: Self) -> Result<(), &'static str> {
        self.increment_states(1);
        self.states.insert(0, 0);

        other.increment_states(
            u32::try_from(self.num_states())
                .expect("Failed conversion from usize to u32 when creating a union"),
        );

        let other_start_state = other.start_state();

        for _ in 0..other.num_states() {
            self.add_state();
        }

        // dodaj tranzicije na svoje
        for entry in other.transition_fn {
            self.transition_fn.insert(entry.0, entry.1);
        }

        // dodaj finalna stanja na svoja
        other.accept_states.iter().for_each(|&e| self.add_accept_state(e));

        // dodaj alfabet na svoj
        for symbol in other.alphabet.iter() {
            if !self.alphabet.contains(symbol) {
                self.alphabet.push(*symbol);
            }
        }

        // povezi novo pocetno stanje sa
        // proslim pocetnim stanjima
        self.add_transition((0, EMPTY_STRING), 1)?;
        self.add_transition((0, EMPTY_STRING), other_start_state)?;

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

        nfa.add_accept_state(3);

        nfa
    }

    #[test]
    fn nfa_creation() {
        let nfa = Nfa::new();

        assert_eq!(vec![0], nfa.states, "States set should only contain a '0'");
        assert_eq!(
            vec![EMPTY_STRING],
            nfa.alphabet,
            "Alphabet should only contain an empty string"
        );
        assert!(
            nfa.transition_fn.is_empty(),
            "Transition fn should be empty"
        );
        // assert_eq!(0, nfa.start_state, "Start state should be set to zero");
        assert!(
            nfa.accept_states.is_empty(),
            "Accept states set should be empty"
        );
    }

    #[test]
    fn nfa_construction() {
        let nfa = Nfa::construct("abcde");

        assert_eq!(vec![0, 1, 2, 3, 4, 5], nfa.states);
        assert_eq!(vec![1], *nfa.get_transition((0, 'a')).unwrap());
        assert_eq!(vec![2], *nfa.get_transition((1, 'b')).unwrap());
        assert_eq!(vec![3], *nfa.get_transition((2, 'c')).unwrap());
        assert_eq!(vec![4], *nfa.get_transition((3, 'd')).unwrap());
        assert_eq!(vec![5], *nfa.get_transition((4, 'e')).unwrap());
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

        assert_eq!(vec![EMPTY_STRING, 'A', 'B', 'C'], nfa.alphabet);
    }

    #[test]
    fn nfa_removing_symbol() {
        let mut nfa = prepare_nfa();

        nfa.remove_symbol('B');

        assert_eq!(vec![EMPTY_STRING, 'A', 'C'], nfa.alphabet);
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

        assert_eq!(vec![1, 2, 3], nfa.accept_states);
    }

    #[test]
    fn nfa_concat() {
        let mut first = Nfa::new();
        let mut second = Nfa::new();

        first.add_state();
        first.add_state();
        first.add_symbol('A');
        first.add_symbol('B');
        first.add_transition((0, EMPTY_STRING), 1).unwrap();
        first.add_transition((0, 'A'), 0).unwrap();
        first.add_transition((0, 'B'), 2).unwrap();
        first.add_transition((1, EMPTY_STRING), 2).unwrap();
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
        second.add_transition((2, EMPTY_STRING), 1).unwrap();
        println!("second: {:?}", second.transition_fn);

        first
            .concat(*first.states.last().unwrap(), second)
            .expect("The concat method crashed");

        assert_eq!(
            vec![0, 1, 2, 3, 4, 5],
            first.states,
            "The number of states is wrong when concatenating"
        );
        assert_eq!(
            vec![EMPTY_STRING, 'A', 'B'],
            first.alphabet,
            "The alphabet symbols don't match"
        );
        assert_eq!(
            vec![3],
            *first.transition_fn.get(&(2, EMPTY_STRING)).unwrap(),
            "An empty string transition is missing between the nfas"
        );
        assert_eq!(vec![4], *first.transition_fn.get(&(3, 'A')).unwrap());
        assert_eq!(vec![5], *first.transition_fn.get(&(4, 'B')).unwrap());
        assert_eq!(
            vec![4],
            *first.transition_fn.get(&(5, EMPTY_STRING)).unwrap()
        );
        assert_eq!(vec![3], *first.transition_fn.get(&(5, 'B')).unwrap());
        println!("result: {:?}", first.transition_fn);
    }

    #[test]
    fn nfa_kleene_star() {
        let mut nfa = prepare_nfa();
        nfa.kleene_star().expect("Kleene star method failed");

        assert_eq!(vec![0, 1, 2, 3, 4], nfa.states, "States set is not correct");
        assert_eq!(
            vec![0, 4],
            nfa.accept_states,
            "Accept states is not correct"
        );

        assert_eq!(vec![1], *nfa.transition_fn.get(&(0, EMPTY_STRING)).unwrap());
        assert_eq!(vec![1], *nfa.transition_fn.get(&(4, EMPTY_STRING)).unwrap());
        assert_eq!(vec![1, 2], *nfa.transition_fn.get(&(1, 'A')).unwrap());
        assert_eq!(vec![2, 4], *nfa.transition_fn.get(&(3, 'B')).unwrap());
    }

    #[test]
    fn nfa_union() {
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

        first.union(second).unwrap();
        
        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6], first.states);
        assert_eq!(vec![1, 5], *first.transition_fn.get(&(0, EMPTY_STRING)).unwrap());
        assert_eq!(vec![4], *first.transition_fn.get(&(3, 'b')).unwrap());
        assert_eq!(vec![6], *first.transition_fn.get(&(5, 'a')).unwrap());
        assert_eq!(vec![4, 6], first.accept_states);
    }
}
