use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Wildcard(Quantifier),
    GroupElement(Quantifier, Vec<Token>),
    Element(Quantifier, char),
}

impl Token {
    pub fn quantifier(&self) -> &Quantifier {
        match self {
            Token::Wildcard(quantifier) => quantifier,
            Token::GroupElement(quantifier, _states) => quantifier,
            Token::Element(quantifier, _value) => quantifier,
        }
    }

    pub fn set_quantifier(&mut self, new_quantifier: Quantifier) {
        match self {
            Token::Wildcard(quantifier) => *quantifier = new_quantifier,
            Token::GroupElement(quantifier, _states) => *quantifier = new_quantifier,
            Token::Element(quantifier, _value) => *quantifier = new_quantifier,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Quantifier {
    ExactlyOne,
    ZeroOrOne,
    ZeroOrMore,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedQuantifier,
    RepeatedQuantifier,
    NoGroupToClose,
    BadEscapeChar,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedQuantifier => {
                write!(f, "Regex should not start from the quantifier")
            }
            ParseError::RepeatedQuantifier => {
                write!(f, "Quantifier must follow an unquantified element or group")
            }
            ParseError::NoGroupToClose => {
                write!(f, "No group to close")
            }
            ParseError::BadEscapeChar => {
                write!(f, "Bad escape character")
            }
        }
    }
}
