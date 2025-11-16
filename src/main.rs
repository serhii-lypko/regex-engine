/*
    a|b – Match either “a” or “b
    ? – Zero or one
    + – one or more
    * – zero or more
    {N} – Exactly N number of times (where N is a number)
    {N,} – N or more number of times (where N is a number)
    {N,M} – Between N and M number of times (where N and M are numbers and N < M)

    Square brackets "[]" define a character class:
    A character class matches exactly one character from the set inside.

    "()" does both grouping and capturing simultaneously.
    (com|en|uk) - groups three alternatives. (ab)+ - the + applies to "ab" as a unit,
    not just "b"


    -- -- Examples: -- --

    ((a|b)c)+
    >> ac, bc, acac, bcbc, acaccb, etc.

    ^[a-z]+@(gmail|yahoo)\.(com|en|uk)$
    >> james@gmail.com

    /^([a-z0-9_\.-]+)@([a-z0-9\.-]+)\.([a-z\.]{2,6})$/
    >> some.one@a.cool.domain.com
*/

/*
    ** Basic engine requirements **

    -> Regex provided
    -> Input provided
    Return: true or false if

    Regular expression match from the start

    ELEMENTS:
        Literals      a, 5, #, &, etc
        Wildcard      .
        Groups        ( )
        Escaped       \x

    QUANTIFIERS:
        Optional      ?
        Zero or more  *
        One or more   +

    TODO ->
        - validate if given regex is invalid? (unmatches parentheses etc.)
        - make thread safe?
*/

mod models;
mod parser;

use parser::Parser;

use crate::models::State;
use std::collections::VecDeque;

/// Where (is match, chars consumed)
type MatchResult = (bool, usize);

// abc

// fn match_state_against_index(state: State, i: usize, source: &Vec<char>) -> MatchResult {
//     if i > source.len() {
//         return (false, 0);
//     }

//     match state {
//         State::Wildcard(_) => todo!(),
//         State::GroupElement(quantifier, states) => todo!(),
//         State::Element(_, char) => {
//             if char != source[i] {
//                 return (false, 1);
//             } else {
//                 return (true, 1);
//             }
//         }
//     }
// }

// TODO -> wrapper func for external interface
fn test(states: Vec<State>, source: &[char]) -> (bool, usize) {
    // dbg!(&states);
    // dbg!(&source);
    // println!("-------");

    let mut queue = VecDeque::from(states);

    let mut i = 0;

    while let Some(state) = queue.pop_front() {
        match state.quantifier() {
            models::Quantifier::ExactlyOne => {
                if i >= source.len() {
                    return (false, i);
                }

                match state {
                    State::Wildcard(_) => todo!(),
                    State::GroupElement(_, states) => todo!(),
                    State::Element(_, char) => {
                        if char != source[i] {
                            return (false, i + 1);
                        } else {
                            i = i + 1;
                        }
                    }
                }
            }
            models::Quantifier::ZeroOrOne => match state {
                State::Wildcard(_) => todo!(),
                State::GroupElement(_, states) => {
                    let (is_match, chars_consumed) = test(states.clone(), &source[i..]);

                    if is_match {
                        i = i + chars_consumed;
                    }
                }
                State::Element(_, char) => {
                    if i >= source.len() {
                        return (false, i);
                    }

                    if char == source[i] {
                        i = i + 1;
                    }
                }
            },
            models::Quantifier::ZeroOrMore => match state {
                State::Wildcard(_) => todo!(),
                State::GroupElement(_, states) => loop {
                    let (is_match, chars_consumed) = test(states.clone(), &source[i..]);

                    if is_match {
                        i = i + chars_consumed;
                    }

                    if !is_match || i >= source.len() {
                        break;
                    }
                },
                State::Element(_, char) => {
                    if i >= source.len() {
                        return (false, i);
                    }

                    if char == source[i] {
                        while i < source.len() && char == source[i] {
                            i = i + 1;
                        }
                    }
                }
            },
        }
    }

    return (true, i);
}

fn main() {
    /*
        ? – Zero or one
        + – one or more
        * – zero or more
    */

    // let re = "abc*1";
    // let source = "abcccc1";

    let re = "123(abc)?__(awesome)*__";
    let source = "123__awesomeawesome__";

    // let re = "ab(cde)*__";
    // let re = "ab(cde)*";

    // let re = "a?";
    // let source = "";

    let mut parser = Parser::new(re);
    let parsed_states = parser.parse().unwrap();

    // dbg!(&parsed_states);

    // TODO -> should be part of init stages
    let source: Vec<char> = source.chars().collect();
    let (is_matched, _) = test(parsed_states, &source);

    dbg!(is_matched);
}
