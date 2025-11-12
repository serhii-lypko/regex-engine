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

mod parser;

use parser::Parser;
use regex::Regex;

fn playground() {
    let re = Regex::new(r"([a-z]+)@([a-z]+)\.com").unwrap();
    let hay = "james@gmail.com";

    let matches = re.captures(hay);
    dbg!(matches);

    let result = re.captures_iter(hay);
    for res in result {
        dbg!(res);
    }
}

fn main() {
    //
}
