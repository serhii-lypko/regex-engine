use std::fmt;

#[derive(Debug)]
pub struct Parser<'a> {
    re: &'a str,
    stack: Vec<Vec<Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(re: &'a str) -> Self {
        Self {
            re,
            stack: vec![vec![]],
        }
    }

    /// Returns array of states that will be used to process an input string.
    pub fn parse(&mut self) -> Result<(), ParseError> {
        // Should iterate over the sequance of regex and generate or modify the state

        for char in self.re.chars() {
            match char {
                '.' => {
                    let token = Token::Wildcard(Quantifier::ExactlyOne);
                    self.stack.last_mut().map(|foo| {
                        foo.push(token);
                    });
                }

                // Quantifiers
                '?' => {
                    let last_element = self.stack.last_mut().and_then(|seq| seq.last_mut());

                    match last_element {
                        // Check of regex does not start from the quantifier.
                        None => return Err(ParseError::UnexpectedQuantifier),
                        Some(last_token) => {
                            // Checks that expressions like /a++?/, /a*+?/, etc. are not allowed.
                            if *last_token.quantifier() != Quantifier::ExactlyOne {
                                return Err(ParseError::RepeatedQuantifier);
                            }

                            last_token.set_quantifier(Quantifier::ZeroOrOne);

                            dbg!(&self.stack);

                            return Ok(());
                        }
                    }
                }
                '+' => todo!(),
                '*' => todo!(),

                _ => todo!(),
            }
        }

        Ok(())
    }
}

// TODO -> not sure if such name is appropriate?
#[derive(Debug)]
enum Token {
    Wildcard(Quantifier),
}

impl Token {
    pub fn quantifier(&self) -> &Quantifier {
        match self {
            Token::Wildcard(quantifier) => quantifier,
        }
    }

    pub fn set_quantifier(&mut self, new_quantifier: Quantifier) {
        match self {
            Token::Wildcard(quantifier) => *quantifier = new_quantifier,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Quantifier {
    ExactlyOne,
    ZeroOrOne,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedQuantifier,
    RepeatedQuantifier,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic() {
        let mut parser = Parser::new(".?");
        let res = parser.parse();

        //
    }

    #[test]
    fn should_not_start_from_quanitifer() {
        let mut parser = Parser::new("?");
        let res = parser.parse();

        assert_eq!(res, Err(ParseError::UnexpectedQuantifier));

        // TODO -> more cases
    }

    #[test]
    fn should_not_have_repeated_quantifiers() {
        let mut parser = Parser::new("a++?");
        let res = parser.parse();

        // TODO ->
    }
}
