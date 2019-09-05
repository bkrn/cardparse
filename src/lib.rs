
pub use cardparse_derive::*;

#[derive(Debug)]
pub enum ParseError {}

pub trait CardParse {
    fn cardparse(s: &str) -> Result<Self, ParseError> where Self: Sized;
}

pub mod prelude {
    pub use crate::{CardParse, ParseError};
}

#[cfg(test)]
mod test {
    use super::prelude::*;

    #[derive(CardParse)]
    struct TLE {
        #[location(line=1,start=1,end=12)]
        field_one: String,
        #[location(line=2,start=6,end=12)]
        field_two: String,
    }

    #[test]
    fn tle() {
        let tle = TLE::cardparse("Some String it is\nAnd also some other string");
        assert!(tle.is_ok());
        let tle = tle.unwrap();
        assert_eq!(tle.field_one, "Some String ");
        assert_eq!(tle.field_two, "lso som");
        
    }
}