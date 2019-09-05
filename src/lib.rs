
pub use cardparse_derive::*;


pub trait CardParse {
    fn cardparse(s: &str) -> Result<Self, failure::Error> where Self: Sized;
}

#[cfg(test)]
mod test {
    use super::CardParse;

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