#[macro_use] extern crate failure;
pub use cardparse_derive::*;

#[derive(Debug,Fail)]
pub enum ParseError {
    #[fail(display = "Field '{}' could not be parsed from '{}' because the source was too long", field, source_line)]
    SourceTooLong{field: String, start: usize, end: Option<usize>, line: usize, source_line: String},
    #[fail(display = "Field '{}' could not be parsed from '{}' because the source was too short", field, source_line)]
    SourceTooShort{field: String, start: usize, end: Option<usize>, line: usize, source_line: String},
}

pub trait CardParse {
    fn cardparse(s: &str) -> Result<Self, ParseError> where Self: Sized;
}

pub mod prelude {
    pub use crate::{CardParse, ParseError};
}

#[cfg(test)]
mod test {
    use super::prelude::*;

    #[derive(CardParse,Debug)]
    struct Simple {
        #[location(line=1,start=1,end=12)]
        field_one: String,
        #[location(line=2,start=6,end=12)]
        field_two: String,
    }

    #[derive(CardParse,Debug)]
    struct FirstNoEnd {
        #[location(line=1,start=1)]
        field_one: String,
        #[location(line=2,start=6,max=24)]
        field_two: String,
        #[location(line=2,start=1,end=5)]
        field_three: String,
    }

    #[test]
    fn simple_test() {
        let simple = Simple::cardparse("Some String it is\nAnd also some other string");
        assert!(simple.is_ok());
        let simple = simple.unwrap();
        assert_eq!(simple.field_one, "Some String ");
        assert_eq!(simple.field_two, "lso som");
        
    }

    #[test]
    fn first_no_end_test() {
        let first_no_end = FirstNoEnd::cardparse("Some String it is\nAnd also some other string");
        assert!(first_no_end.is_ok());
        let first_no_end = first_no_end.unwrap();
        assert_eq!(first_no_end.field_one, "Some String it is");
        assert_eq!(first_no_end.field_two, "lso some other stri");
        assert_eq!(first_no_end.field_three, "And a");
    }

     #[test]
    fn first_no_end_failure_test() {
        let first_no_end = FirstNoEnd::cardparse("Some String it is\nAnd");
        assert!(first_no_end.is_err());
        assert_eq!(
            format!("{}", first_no_end.unwrap_err()), 
            "Field 'field_two' could not be parsed from 'Some String it is\nAnd' because the source was too short"
        );
    }
}