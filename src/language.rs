// pub mod parser;
// use std::collections::HashMap;
//
// use crate::automata::dfa;
//
// pub fn assemble_automata(regex: &Regex, automata: &dfa::Dfa) {
//     let regex = regex.infix_to_postfix(); // pretvori regex u postfix prvo
// }
//
// pub enum Regex {
//     Infix(String),
//     Postfix(String),
// }
//
// impl Regex {
//     fn clone(&self) -> Regex {
//         match self {
//             Regex::Postfix(expression) => Regex::Postfix(expression.clone()),
//             Regex::Infix(expression) => Regex::Infix(expression.clone()),
//         }
//     }
//
//     pub fn infix_to_postfix(&self) -> Regex {
//         match self {
//             Regex::Postfix(_) => self.clone(),
//             Regex::Infix(regex) => {
//                 let mut result = String::new();
//                 let mut stack: Vec<char> = Vec::new();
//                 let mut precedence: HashMap<char, u32> = HashMap::new();
//
//                 result.reserve(regex.len());
//                 precedence.insert('*', 2);
//                 precedence.insert('.', 1);
//                 precedence.insert('|', 0);
//
//                 for symbol in regex.chars() {
//                     match symbol {
//                         '(' => {
//                             stack.push(symbol);
//                         }
//                         ')' => {
//                             while !stack.is_empty() && *stack.last().unwrap() != '(' {
//                                 result.push(stack.pop().unwrap());
//                             }
//                             stack.pop(); // skini '(' sa steka
//                         }
//                         '*' | '.' | '|' => {
//                             let operator_priority = precedence.get(&symbol).unwrap();
//                             let stack_operator_priority =
//                                 precedence.get(stack.last().unwrap()).unwrap();
//                             while !stack.is_empty() && operator_priority <= stack_operator_priority
//                             {
//                                 result.push(stack.pop().unwrap());
//                             }
//                             stack.push(symbol);
//                         }
//                         alpha => result.push(alpha),
//                     }
//                 }
//
//                 while let Some(element) = stack.pop() {
//                     result.push(element);
//                 }
//
//                 Regex::Postfix(result)
//             }
//         }
//     }
// }
