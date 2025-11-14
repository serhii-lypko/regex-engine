use std::iter::Peekable;
use std::str::Chars;

use crate::models::{ParseError, Quantifier, Token};

#[derive(Debug)]
pub(crate) struct Parser<'a> {
    /// The Peekable iterator is shared across all recursive calls when grouping
    chars: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(re: &'a str) -> Self {
        Self {
            chars: re.chars().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut output: Vec<Token> = vec![];

        while let Some(char) = self.chars.next() {
            match char {
                '.' => output.push(Token::Wildcard(Quantifier::ExactlyOne)),

                '?' => match output.last_mut() {
                    Some(token) => {
                        if *token.quantifier() != Quantifier::ExactlyOne {
                            return Err(ParseError::RepeatedQuantifier);
                        }

                        token.set_quantifier(Quantifier::ZeroOrOne);
                    }
                    None => return Err(ParseError::UnexpectedQuantifier),
                },
                '*' => match output.last_mut() {
                    Some(token) => {
                        if *token.quantifier() != Quantifier::ExactlyOne {
                            return Err(ParseError::RepeatedQuantifier);
                        }

                        token.set_quantifier(Quantifier::ZeroOrMore);
                    }
                    None => return Err(ParseError::UnexpectedQuantifier),
                },
                '+' => match output.last_mut() {
                    Some(token) => {
                        if *token.quantifier() != Quantifier::ExactlyOne {
                            return Err(ParseError::RepeatedQuantifier);
                        }

                        token.set_quantifier(Quantifier::ExactlyOne);

                        let mut one_more = token.clone();
                        one_more.set_quantifier(Quantifier::ZeroOrMore);
                        output.push(one_more);
                    }
                    None => return Err(ParseError::UnexpectedQuantifier),
                },

                '\\' => match self.chars.next() {
                    Some(char) => output.push(Token::Element(Quantifier::ExactlyOne, char)),
                    None => return Err(ParseError::BadEscapeChar),
                },

                // TODO -> handle ParseError::NoGroupToClose? like "abc)" -> pass flag is_group?
                '(' => output.push(Token::GroupElement(Quantifier::ExactlyOne, self.parse()?)),
                ')' => return Ok(output),

                _ => output.push(Token::Element(Quantifier::ExactlyOne, char)),
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_characters() {
        let mut parser = Parser::new("abc");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0], Token::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], Token::Element(Quantifier::ExactlyOne, 'b'));
        assert_eq!(res[2], Token::Element(Quantifier::ExactlyOne, 'c'));
    }

    #[test]
    fn test_wildcard() {
        let mut parser = Parser::new("a.c");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0], Token::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], Token::Wildcard(Quantifier::ExactlyOne));
        assert_eq!(res[2], Token::Element(Quantifier::ExactlyOne, 'c'));
    }

    #[test]
    fn test_zero_or_one_quantifier() {
        let mut parser = Parser::new("a?");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0], Token::Element(Quantifier::ZeroOrOne, 'a'));
    }

    #[test]
    fn test_zero_or_more_quantifier() {
        let mut parser = Parser::new("a*");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0], Token::Element(Quantifier::ZeroOrMore, 'a'));
    }

    #[test]
    fn test_one_or_more_quantifier() {
        let mut parser = Parser::new("a+");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], Token::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], Token::Element(Quantifier::ZeroOrMore, 'a'));
    }

    #[test]
    fn test_wildcard_with_quantifiers() {
        let mut parser = Parser::new(".*");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        assert_eq!(res[0], Token::Wildcard(Quantifier::ZeroOrMore));
    }

    #[test]
    fn test_escape_special_chars() {
        let mut parser = Parser::new(r"\+\?\*");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 3);
        assert_eq!(res[0], Token::Element(Quantifier::ExactlyOne, '+'));
        assert_eq!(res[1], Token::Element(Quantifier::ExactlyOne, '?'));
        assert_eq!(res[2], Token::Element(Quantifier::ExactlyOne, '*'));
    }

    #[test]
    fn test_escape_regular_chars() {
        let mut parser = Parser::new(r"\a\b");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], Token::Element(Quantifier::ExactlyOne, 'a'));
        assert_eq!(res[1], Token::Element(Quantifier::ExactlyOne, 'b'));
    }

    #[test]
    fn test_simple_group() {
        let mut parser = Parser::new("(ab)");
        let res = parser.parse().unwrap();

        assert_eq!(res.len(), 1);
        match &res[0] {
            Token::GroupElement(q, tokens) => {
                assert_eq!(*q, Quantifier::ExactlyOne);
                assert_eq!(tokens.len(), 2);
                assert_eq!(tokens[0], Token::Element(Quantifier::ExactlyOne, 'a'));
                assert_eq!(tokens[1], Token::Element(Quantifier::ExactlyOne, 'b'));
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
            Token::GroupElement(q, _) => {
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
            Token::GroupElement(q, outer_tokens) => {
                assert_eq!(*q, Quantifier::ExactlyOne);
                assert_eq!(outer_tokens.len(), 3); // a, (bc), d

                assert_eq!(outer_tokens[0], Token::Element(Quantifier::ExactlyOne, 'a'));

                match &outer_tokens[1] {
                    Token::GroupElement(_, inner_tokens) => {
                        assert_eq!(inner_tokens.len(), 2);
                        assert_eq!(inner_tokens[0], Token::Element(Quantifier::ExactlyOne, 'b'));
                        assert_eq!(inner_tokens[1], Token::Element(Quantifier::ExactlyOne, 'c'));
                    }
                    _ => panic!("Expected nested GroupElement"),
                }

                assert_eq!(outer_tokens[2], Token::Element(Quantifier::ExactlyOne, 'd'));
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
            (Token::GroupElement(_, g1), Token::GroupElement(_, g2)) => {
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
            Token::GroupElement(_, tokens) => {
                assert_eq!(tokens.len(), 0);
            }
            _ => panic!("Expected GroupElement"),
        }
    }

    #[test]
    fn test_unclosed_group_completes_at_end() {
        let mut parser = Parser::new("(abc");
        let res = parser.parse().unwrap();

        dbg!(res);

        // TODO

        // assert_eq!(res.len(), 3);
    }
}
