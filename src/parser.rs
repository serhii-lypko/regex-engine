use std::fmt;

#[derive(Debug)]
pub(crate) struct Parser<'a> {
    re: &'a str,

    // TODO -> would be nice to have stack as separate struct
    stack: Vec<Vec<Token>>,

    output: Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(re: &'a str) -> Self {
        Self {
            re,
            stack: vec![vec![]],
            output: vec![],
        }
    }

    /// Returns array of states that will be used to process an input string.
    pub fn parse(mut self) -> Result<Vec<Token>, ParseError> {
        // Should iterate over the sequance of regex and generate or modify the state

        let chars: Vec<char> = self.re.chars().collect();
        let mut i = 0;

        while i < self.re.len() {
            let char = chars[i];

            match char {
                '.' => {
                    let token = Token::Wildcard(Quantifier::ExactlyOne);
                    self.stack.last_mut().map(|foo| {
                        foo.push(token);
                    });
                    i = i + 1;
                }

                '?' => self.handle_zero_or_quantifier(|last_token| {
                    last_token.set_quantifier(Quantifier::ZeroOrOne);
                    i = i + 1;
                })?,
                '*' => self.handle_zero_or_quantifier(|last_token| {
                    last_token.set_quantifier(Quantifier::ZeroOrMore);
                    i = i + 1;
                })?,
                '+' => {
                    let stack_top = self
                        .stack
                        .last_mut()
                        .ok_or(ParseError::UnexpectedQuantifier)?;

                    let last_element = stack_top
                        .last_mut()
                        .ok_or(ParseError::UnexpectedQuantifier)?;

                    if *last_element.quantifier() != Quantifier::ExactlyOne {
                        return Err(ParseError::RepeatedQuantifier);
                    }

                    let mut one_more = last_element.clone();
                    one_more.set_quantifier(Quantifier::ZeroOrMore);
                    stack_top.push(one_more);

                    i = i + 1;
                }

                '(' => {
                    self.stack.push(vec![]);
                    i = i + 1;
                }
                ')' => {
                    if self.stack.len() <= 1 {
                        return Err(ParseError::NoGroupToClose);
                    }

                    let states = self.stack.pop().ok_or(ParseError::NoGroupToClose)?;
                    let token = Token::GroupElement(Quantifier::ExactlyOne, states);

                    self.stack.last_mut().map(|foo| {
                        foo.push(token);
                    });

                    i = i + 1;
                }

                _ => {
                    //
                }
            }
        }

        dbg!(self.stack);

        Ok(self.output)
    }

    fn handle_zero_or_quantifier<H>(&mut self, handler: H) -> Result<(), ParseError>
    where
        H: FnOnce(&mut Token) -> (),
    {
        let last_element = self.stack.last_mut().and_then(|seq| seq.last_mut());

        match last_element {
            // Check of regex does not start from the quantifier.
            None => return Err(ParseError::UnexpectedQuantifier),
            Some(last_token) => {
                // Checks that expressions like /a++?/, /a*+?/, etc. are not allowed.
                if *last_token.quantifier() != Quantifier::ExactlyOne {
                    return Err(ParseError::RepeatedQuantifier);
                }

                handler(last_token);
                return Ok(());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Wildcard(Quantifier),
    GroupElement(Quantifier, Vec<Token>),
}

impl Token {
    pub fn quantifier(&self) -> &Quantifier {
        match self {
            Token::Wildcard(quantifier) => quantifier,
            Token::GroupElement(quantifier, _states) => todo!(),
        }
    }

    pub fn set_quantifier(&mut self, new_quantifier: Quantifier) {
        match self {
            Token::Wildcard(quantifier) => *quantifier = new_quantifier,
            Token::GroupElement(quantifier, _states) => todo!(),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic() {
        let parser = Parser::new(".+(");
        let res = parser.parse();
    }

    #[test]
    fn should_not_start_from_quanitifer() {
        let parser = Parser::new("?");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));

        let parser = Parser::new("*");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));

        let parser = Parser::new("+");
        let res = parser.parse();
        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));
    }

    #[test]
    fn should_not_have_repeated_quantifiers() {
        let parser = Parser::new("a++?");
        let res = parser.parse();

        // TODO ->
    }

    #[test]
    fn group_should_be_closed() {
        let parser = Parser::new("(.)");
        let res = parser.parse();

        // TODO ->
    }

    // TODO -> test nested groups
}
