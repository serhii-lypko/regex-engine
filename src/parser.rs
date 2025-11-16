use std::iter::Peekable;
use std::mem;
use std::str::Chars;

use crate::models::{ParseError, Quantifier, State};

#[derive(Debug)]
pub(crate) struct Parser<'a> {
    /// The Peekable iterator is shared across all recursive calls when grouping
    chars: Peekable<Chars<'a>>,
    stack: Vec<StackItem>,
}

impl<'a> Parser<'a> {
    pub fn new(re: &'a str) -> Self {
        Self {
            chars: re.chars().peekable(),
            stack: vec![],
        }
    }

    /// Returns an array of states
    pub fn parse(&mut self) -> Result<Vec<State>, ParseError> {
        while let Some(char) = self.chars.next() {
            match char {
                '.' => {
                    let token = State::Wildcard(Quantifier::ExactlyOne);
                    self.stack.push(StackItem::State(token));
                }

                '?' => self.handle_base_quantifiers(Quantifier::ZeroOrOne)?,
                '*' => self.handle_base_quantifiers(Quantifier::ZeroOrMore)?,

                '+' => self.handle_plus_quantifier()?,

                '(' => self.stack.push(StackItem::GroupOpen),
                ')' => {
                    let mut group_tokens: Vec<State> = Vec::new();

                    loop {
                        match self.stack.pop() {
                            Some(StackItem::State(token)) => group_tokens.push(token),
                            Some(StackItem::GroupOpen) => {
                                // TODO -> use VecDeque with push front?
                                group_tokens.reverse();

                                let group =
                                    State::GroupElement(Quantifier::ExactlyOne, group_tokens);
                                self.stack.push(StackItem::State(group));
                                break;
                            }
                            None => return Err(ParseError::NoGroupToClose),
                        }
                    }
                }

                '\\' => match self.chars.next() {
                    Some(char) => {
                        let token = State::Element(Quantifier::ExactlyOne, char);
                        self.stack.push(StackItem::State(token));
                    }
                    None => return Err(ParseError::BadEscapeChar),
                },

                _ => {
                    let token = State::Element(Quantifier::ExactlyOne, char);
                    self.stack.push(StackItem::State(token));
                }
            }
        }

        let stack = mem::take(&mut self.stack);
        let result: Vec<State> = stack
            .into_iter()
            .map(|stack_item| stack_item.into())
            .collect();

        Ok(result)
    }

    fn handle_base_quantifiers(&mut self, new_quantifier: Quantifier) -> Result<(), ParseError> {
        match self.stack.last_mut() {
            Some(stack_item) => match stack_item {
                StackItem::State(token) => {
                    if *token.quantifier() != Quantifier::ExactlyOne {
                        return Err(ParseError::RepeatedQuantifier);
                    }

                    token.set_quantifier(new_quantifier);
                }
                StackItem::GroupOpen => return Err(ParseError::UnexpectedQuantifier),
            },
            None => return Err(ParseError::UnexpectedQuantifier),
        }

        Ok(())
    }

    fn handle_plus_quantifier(&mut self) -> Result<(), ParseError> {
        let last_token = self
            .stack
            .last_mut()
            .ok_or(ParseError::UnexpectedQuantifier)?;

        match last_token {
            StackItem::State(token) => {
                if *token.quantifier() != Quantifier::ExactlyOne {
                    return Err(ParseError::RepeatedQuantifier);
                }

                let mut one_more = token.clone();
                one_more.set_quantifier(Quantifier::ZeroOrMore);
                self.stack.push(StackItem::State(one_more));

                Ok(())
            }
            StackItem::GroupOpen => Err(ParseError::UnexpectedQuantifier),
        }
    }
}

#[derive(Debug)]
enum StackItem {
    State(State),
    GroupOpen,
}

impl From<StackItem> for State {
    fn from(value: StackItem) -> Self {
        match value {
            StackItem::State(token) => token,
            StackItem::GroupOpen => panic!("Unclosed group found: {}", ParseError::NoGroupToClose),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO -> more cases for unclosing group

    #[test]
    fn generic() {
        let mut parser = Parser::new("a?(b.*c)+d");
        let res = parser.parse();

        dbg!(res);
    }

    #[test]
    fn test_simple_characters() {
        let mut parser = Parser::new("abc");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0], State::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], State::Element(Quantifier::ExactlyOne, 'b'));
        assert_eq!(res[2], State::Element(Quantifier::ExactlyOne, 'c'));
    }

    #[test]
    fn test_wildcard() {
        let mut parser = Parser::new("a.c");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0], State::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], State::Wildcard(Quantifier::ExactlyOne));
        assert_eq!(res[2], State::Element(Quantifier::ExactlyOne, 'c'));
    }

    #[test]
    fn test_zero_or_one_quantifier() {
        let mut parser = Parser::new("a?");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0], State::Element(Quantifier::ZeroOrOne, 'a'));
    }

    #[test]
    fn test_zero_or_more_quantifier() {
        let mut parser = Parser::new("a*");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0], State::Element(Quantifier::ZeroOrMore, 'a'));
    }

    #[test]
    fn test_one_or_more_quantifier() {
        let mut parser = Parser::new("a+");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], State::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], State::Element(Quantifier::ZeroOrMore, 'a'));
    }

    #[test]
    fn test_wildcard_with_quantifiers() {
        let mut parser = Parser::new(".*");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0], State::Wildcard(Quantifier::ZeroOrMore));
    }

    #[test]
    fn test_escape_special_chars() {
        let mut parser = Parser::new(r"\+\?\*");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0], State::Element(Quantifier::ExactlyOne, '+'));
        assert_eq!(res[1], State::Element(Quantifier::ExactlyOne, '?'));
        assert_eq!(res[2], State::Element(Quantifier::ExactlyOne, '*'));
    }

    #[test]
    fn test_escape_regular_chars() {
        let mut parser = Parser::new(r"\a\b");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], State::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], State::Element(Quantifier::ExactlyOne, 'b'));
    }

    #[test]
    fn test_simple_group() {
        let mut parser = Parser::new("(ab)");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        match &res[0] {
            State::GroupElement(q, tokens) => {
                assert_eq!(*q, Quantifier::ExactlyOne);
                assert_eq!(tokens.len(), 2);
                assert_eq!(tokens[0], State::Element(Quantifier::ExactlyOne, 'a'));
                assert_eq!(tokens[1], State::Element(Quantifier::ExactlyOne, 'b'));
            }
            _ => panic!("Expected GroupElement"),
        }
    }

    #[test]
    fn test_group_with_quantifier() {
        let mut parser = Parser::new("(ab)?");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        match &res[0] {
            State::GroupElement(q, _) => {
                assert_eq!(*q, Quantifier::ZeroOrOne);
            }
            _ => panic!("Expected GroupElement"),
        }
    }

    #[test]
    fn test_nested_groups() {
        let mut parser = Parser::new("(a(bc)d)");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        match &res[0] {
            State::GroupElement(q, outer_tokens) => {
                assert_eq!(*q, Quantifier::ExactlyOne);
                assert_eq!(outer_tokens.len(), 3); // a, (bc), d

                assert_eq!(outer_tokens[0], State::Element(Quantifier::ExactlyOne, 'a'));

                match &outer_tokens[1] {
                    State::GroupElement(_, inner_tokens) => {
                        assert_eq!(inner_tokens.len(), 2);
                        assert_eq!(inner_tokens[0], State::Element(Quantifier::ExactlyOne, 'b'));
                        assert_eq!(inner_tokens[1], State::Element(Quantifier::ExactlyOne, 'c'));
                    }
                    _ => panic!("Expected nested GroupElement"),
                }

                assert_eq!(outer_tokens[2], State::Element(Quantifier::ExactlyOne, 'd'));
            }
            _ => panic!("Expected GroupElement"),
        }
    }

    #[test]
    fn test_multiple_groups() {
        let mut parser = Parser::new("(ab)(cd)");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 2);
        match (&res[0], &res[1]) {
            (State::GroupElement(_, g1), State::GroupElement(_, g2)) => {
                assert_eq!(g1.len(), 2);
                assert_eq!(g2.len(), 2);
            }
            _ => panic!("Expected two GroupElements"),
        }
    }

    #[test]
    fn test_complex_pattern() {
        let mut parser = Parser::new("a+b*(cd)?");
        let res = parser.parse().unwrap();

        // a+ produces 2 tokens: a, a*
        // b* produces 1 token: b*
        // (cd)? produces 1 token: group?
        assert_eq!(res.len(), 4);
    }

    // Error cases

    #[test]
    fn test_should_not_start_with_quantifier_question() {
        let mut parser = Parser::new("?");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));
    }

    #[test]
    fn test_should_not_start_with_quantifier_star() {
        let mut parser = Parser::new("*");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));
    }

    #[test]
    fn test_should_not_start_with_quantifier_plus() {
        let mut parser = Parser::new("+");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));
    }

    #[test]
    fn test_repeated_quantifier_star_question() {
        let mut parser = Parser::new("a*?");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::RepeatedQuantifier));
    }

    #[test]
    fn test_repeated_quantifier_plus_star() {
        let mut parser = Parser::new("a+*");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::RepeatedQuantifier));
    }

    #[test]
    fn test_repeated_quantifier_question_plus() {
        let mut parser = Parser::new("a?+");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::RepeatedQuantifier));
    }

    #[test]
    fn test_bad_escape_at_end() {
        let mut parser = Parser::new(r"ab\");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::BadEscapeChar));
    }

    #[test]
    fn test_empty_string() {
        let mut parser = Parser::new("");
        let res = parser.parse().unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn test_empty_group() {
        let mut parser = Parser::new("()");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        match &res[0] {
            State::GroupElement(_, tokens) => {
                assert_eq!(tokens.len(), 0);
            }
            _ => panic!("Expected GroupElement"),
        }
    }
}
