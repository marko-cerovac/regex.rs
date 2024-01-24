fn main() {
    let dfa = fmsi::Dfa::from("a|(ab|b)*").unwrap();
    let nfa = fmsi::Nfa::from("a|(ab|b)*").unwrap();

    println!("------------------- Nfa -------------------");
    println!("{:?}", nfa);
    print!("\n\n\n");
    println!("------------------- Non-minimized Dfa -------------------");
    let non_min_dfa = nfa.to_dfa();
    println!("{:?}", non_min_dfa);
    print!("\n\n\n");
    println!("------------------- Dfa -------------------");
    println!("{:?}", dfa);
    print!("\n\n\n");
    println!("------------------- Regex -------------------");
    let regex = dfa.to_regex();
    println!("{}", regex);
    print!("\n\n\n");

    let nfa = fmsi::Nfa::from("a*b*").unwrap();
    let dfa = fmsi::Dfa::from("a*b*").unwrap();
    println!("------------------- Nfa -------------------");
    println!("{:?}", nfa);
    print!("\n\n\n");
    println!("------------------- Non-minimized Dfa -------------------");
    let non_min_dfa = nfa.to_dfa();
    println!("{:?}", non_min_dfa);
    print!("\n\n\n");
    println!("------------------- Dfa -------------------");
    println!("{:?}", dfa);
    print!("\n\n\n");
    println!("------------------- Regex -------------------");
    let regex = dfa.to_regex();
    println!("{}", regex);
    print!("\n\n\n");
}
