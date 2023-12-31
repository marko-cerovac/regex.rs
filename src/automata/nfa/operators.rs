use super::Nfa;
use crate::automata::traits::*;
use crate::language::EMPTY_STRING;

/// Concatenates the second Nfa on to the first
/// and ads an epsilon transition in between.
/// In the process of doing so, the method
/// function consumes the second Nfa.
///
/// The function assumes that the alphabets are identical.
pub fn concat(first: &mut Nfa, mut second: Nfa) -> Result<(), &'static str> {
    // copy the missing symbols in the alphabet
    for symbol in second.alphabet.iter() {
        if !first.alphabet.contains(symbol) {
            first.alphabet.push(*symbol);
        }
    }

    // increment the state names in the second alphabet
    // to avoid name clashes
    let increment = first.last_added_state() + 1;
    second.increment_states(increment);

    // add the new states
    (0..second.num_states()).for_each(|_| first.add_state());

    // concatenate the transition table of the second
    // onto the first
    for entry in second.transition_fn.iter() {
        for state in entry.1 {
            first.add_transition(&(entry.0 .0, entry.0 .1), *state)?;
        }
    }

    if first.accept_states.is_empty() {
        first.add_transition(&(increment - 1, EMPTY_STRING), increment)?
    } else {
        // concect first's accept states with second's start state
        for accept_state in first.accept_states.clone() {
            first.add_transition(&(accept_state, EMPTY_STRING), increment)?;
        }
    }

    // forget first's final states
    // copy second's final states
    first.accept_states.clear();
    second
        .accept_states
        .iter()
        .for_each(|&e| first.accept_states.push(e));

    Ok(())
}

/// Applies the kleene star operator to a given Nfa
pub fn kleene_star(nfa: &mut Nfa) -> Result<(), &'static str> {
    // add a new final state at the beggining
    nfa.increment_states(1);
    nfa.states.insert(0, 0);
    nfa.add_accept_state(0);

    // add an epsilon transition from every
    // final state to the previous first state
    let accept_states = nfa.accept_states.clone();
    accept_states.iter().for_each(|e| {
        nfa.add_transition(&(*e, EMPTY_STRING), 1).unwrap();
    });

    Ok(())
}

/// Applies the union operator on two given operators.
/// The union is applied on to the first nfa while
/// the second gets consumed.
pub fn union(first: &mut Nfa, mut second: Nfa) -> Result<(), &'static str> {
    // add a new state to the beggining of the first
    first.increment_states(1);
    first.states.insert(0, 0);

    // increment the states in the second nfa
    second.increment_states(
        u32::try_from(first.num_states())
            .expect("Failed conversion from usize to u32 when creating a union"),
    );

    let other_start_state = second.start_state();

    // add enough states to the first
    (0..second.num_states()).for_each(|_| first.add_state());

    // add second's transitions to first
    for entry in second.transition_fn {
        first.transition_fn.insert(entry.0, entry.1);
    }

    // add second's accept states to first
    second
        .accept_states
        .iter()
        .for_each(|&e| first.add_accept_state(e));

    // ad second's alphabet to first
    for symbol in second.alphabet.iter() {
        if !first.alphabet.contains(symbol) {
            first.alphabet.push(*symbol);
        }
    }

    // conncect new first transition with the
    // two branches
    first.add_transition(&(0, EMPTY_STRING), 1)?;
    first.add_transition(&(0, EMPTY_STRING), other_start_state)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::nfa::test_utils;

    #[test]
    fn nfa_concat() {
        let mut first = Nfa::new();
        let mut second = Nfa::new();

        first.add_state();
        first.add_state();
        first.add_symbol('A');
        first.add_symbol('B');
        first.add_transition(&(0, EMPTY_STRING), 1).unwrap();
        first.add_transition(&(0, 'A'), 0).unwrap();
        first.add_transition(&(0, 'B'), 2).unwrap();
        first.add_transition(&(1, EMPTY_STRING), 2).unwrap();
        first.add_transition(&(1, 'A'), 0).unwrap();
        first.add_transition(&(2, 'A'), 2).unwrap();
        first.add_accept_state(2);
        println!("first: {:?}", first.transition_fn);

        second.add_state();
        second.add_state();
        second.add_symbol('A');
        second.add_symbol('B');
        second.add_transition(&(0, 'A'), 1).unwrap();
        second.add_transition(&(1, 'B'), 2).unwrap();
        second.add_transition(&(2, 'B'), 0).unwrap();
        second.add_transition(&(2, EMPTY_STRING), 1).unwrap();
        second.add_accept_state(2);
        println!("second: {:?}", second.transition_fn);

        concat(&mut first, second).expect("The concat method crashed");

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
        let mut nfa = test_utils::prepare_nfa();
        kleene_star(&mut nfa).expect("Kleene star method failed");

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
        let (mut first, second) = test_utils::prepare_nfa_pair();
        union(&mut first, second).unwrap();

        assert_eq!(vec![0, 1, 2, 3, 4, 5, 6], first.states);
        assert_eq!(
            vec![1, 5],
            *first.transition_fn.get(&(0, EMPTY_STRING)).unwrap()
        );
        assert_eq!(vec![4], *first.transition_fn.get(&(3, 'b')).unwrap());
        assert_eq!(vec![6], *first.transition_fn.get(&(5, 'a')).unwrap());
        assert_eq!(vec![4, 6], first.accept_states);
    }
}
