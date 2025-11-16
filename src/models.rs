use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum State {
    Wildcard(Quantifier),
    GroupElement(Quantifier, Vec<State>),
    Element(Quantifier, char),
}

impl State {
    pub fn quantifier(&self) -> &Quantifier {
        match self {
            State::Wildcard(quantifier) => quantifier,
            State::GroupElement(quantifier, _states) => quantifier,
            State::Element(quantifier, _value) => quantifier,
        }
    }

    pub fn set_quantifier(&mut self, new_quantifier: Quantifier) {
        match self {
            State::Wildcard(quantifier) => *quantifier = new_quantifier,
            State::GroupElement(quantifier, _states) => *quantifier = new_quantifier,
            State::Element(quantifier, _value) => *quantifier = new_quantifier,
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
