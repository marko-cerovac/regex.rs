pub const EMPTY_STRING: char = '\0';
// use crate::util::check_for_correctness;
//
// pub enum Token {
//     Union,
//     OpeningBracket,
//     ClosingBracket,
//     KleeneStar,
//     Symbol(char),
// }
//
// pub fn tokenize(expression: &str) -> Result<Vec<Token>, &'static str> {
//     check_for_correctness(expression)?;
//
//     let mut result: Vec<Token> = Vec::with_capacity(expression.len());
//
//     for token in expression.chars() {
//         match token {
//             '(' => result.push(Token::OpeningBracket),
//             ')' => result.push(Token::ClosingBracket),
//             '*' => result.push(Token::KleeneStar),
//             '|' => result.push(Token::Union),
//             alpha => result.push(Token::Symbol(alpha)),
//         }
//     }
//
//     Ok(result)
// }
