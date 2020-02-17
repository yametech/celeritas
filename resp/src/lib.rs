extern crate parser;

use parser::Error;

fn example_parser_error() -> parser::Error {
    parser::Error::Unknown("undefine")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(example_parser_error(), parser::Error::Unknown);
    }
}
