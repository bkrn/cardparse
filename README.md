[![Build Status](https://travis-ci.org/bkrn/cardparse.svg?branch=master)](https://travis-ci.org/bkrn/cardparse)
[![codecov](https://codecov.io/gh/bkrn/cardparse/branch/master/graph/badge.svg)](https://codecov.io/gh/bkrn/cardparse)

# CardParse

Derive a trait that allows creating a struct from a fixed width text data source by specifying the location of data as field attributes in the struct



```rust
use cardparse::prelude::*;

static TLE_STRING: &str = r#"ISS (ZARYA)
1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927
2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537"#;

#[allow(dead_code)]
#[derive(cardparse::CardParse)]
struct TwoLineElement {
    // If end is missing then read to end of line
    // but only as far as max - otherwise return
    // Err(cardparse::ParseError::SourceTooShort{..})
    #[location(line=1,start=1,max=24)]
    name: String,
    #[location(line=2,start=1,end=1)]
    line_number_one: String,
    #[location(line=2,start=3,end=7)]
    satellite_catalog_number_one: String,
    #[location(line=2,start=8,end=8)]
    classification: String,
    #[location(line=2,start=10,end=11)]
    launch_year: String,
    #[location(line=2,start=12,end=14)]
    launch_number: String,
    #[location(line=2,start=15,end=17)]
    launch_piece: String,
    #[location(line=2,start=19,end=20)]
    epoch_year: String,
    #[location(line=2,start=21,end=32)]
    epoch_day: String,
    #[location(line=2,start=34,end=43)]
    ballistic_coefficient: String,
    #[location(line=2,start=45,end=52)]
    second_derivative_of_mean_motion: String,
    #[location(line=2,start=54,end=61)]
    drag_term: String,
    #[location(line=2,start=63,end=63)]
    ephemeris_type: String,
    #[location(line=2,start=65,end=68)]
    element_set_number: String,
    #[location(line=2,start=69,end=69)]
    check_sum: String,
    #[location(line=3,start=01,end=01)]
    line_number_two: String,
    #[location(line=3,start=03,end=07)]
    satellite_catalog_number_two: String,
    #[location(line=3,start=09,end=16)]
    inclination: String,
    #[location(line=3,start=18,end=25)]
    right_ascension_of_ascending_node: String,
    #[location(line=3,start=27,end=33)]
    eccentricity: String,
    #[location(line=3,start=35,end=42)]
    argument_of_perigee: String,
    #[location(line=3,start=44,end=51)]
    mean_anomaly: String,
    #[location(line=3,start=53,end=63)]
    mean_motion: String,
    #[location(line=3,start=64,end=68)]
    revolution_number_at_epoch: String,
    #[location(line=3,start=69,end=69)]
    checksum_two: String,
}

fn main() {
   let tle = TwoLineElement::cardparse(TLE_STRING).unwrap();
   assert_eq!(tle.name, "ISS (ZARYA)");
}