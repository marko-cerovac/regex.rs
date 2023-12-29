pub trait StateIter {
    fn is_empty(&self) -> bool;
    fn states_iter(&self) -> impl Iterator<Item = &u32>;
    fn states_iter_mut(&mut self) -> impl Iterator<Item = &mut u32>;
    fn accept_states_iter(&self) -> impl Iterator<Item = &u32>;
    fn accept_states_iter_mut(&mut self) -> impl Iterator<Item = &mut u32>;
}

pub trait AlphabetIter {
    fn alphabet_iter(&self) -> impl Iterator<Item = &char>;
    fn alphabet_iter_mut(&mut self) -> impl Iterator<Item = &mut char>;
}

// pub trait TransitionIter {
//     type Target;
//
//     fn transitions_iter(&self) -> impl Iterator<Item = (&(u32, char), Self::Target)>;
//     fn transitions_iter_mut(&mut self) -> impl Iterator<Item = (&(u32, char), Self::Target)>;
// }
