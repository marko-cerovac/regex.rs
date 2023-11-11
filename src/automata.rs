use std::collections::{hash_map, HashMap};

pub struct Automata {
    states: Vec<u32>,
    alphabet: Vec<char>,
    transition_fn: HashMap<(u32, char), u32>,
    start_state: u32,
    accept_states: Vec<u32>,
}
