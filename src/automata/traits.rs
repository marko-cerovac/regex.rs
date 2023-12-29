pub trait State {
    fn add_state(&mut self);
    fn remove_state(&mut self);
    fn add_accept_state(&mut self, state: u32);
    fn remove_accept_state(&mut self, target: u32);
}

pub trait Alphabet {
    fn add_symbol(&mut self, symbol: char);
    fn remove_symbol(&mut self, symbol: char);
}

pub trait Transition {
    fn add_transition(&mut self, source: &(u32, char), target: u32) -> Result<(), &'static str>;
    // fn remove_transition(&mut self, source: &(u32, char), target: u32);
}
