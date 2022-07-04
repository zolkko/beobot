use nom::IResult;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::sequence::separated_pair;
use chrono::NaiveTime;
use nom::combinator::{map, map_res};
use nom::{Err, error::Error};


#[derive(Eq, PartialEq, Debug)]
pub(crate) struct TimeRange {
    from: NaiveTime,
    to: NaiveTime,
}

impl TimeRange {
    pub(crate) fn new(from: NaiveTime, to: NaiveTime) -> Self {
        Self { from, to }
    }

    pub(crate) fn parse(input: &str) -> Result<Self, Err<Error<&str>>> {
        let (_, result) = parse_interval(input)?;
        Ok(result)
    }
}

impl From<(NaiveTime, NaiveTime)> for TimeRange {
    fn from((from, to): (NaiveTime, NaiveTime)) -> Self {
        Self { from, to }
    }
}

fn digit_parse(input: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse::<u32>)(input)
}

fn parse_time(input: &str) -> IResult<&str, NaiveTime> {
    map(separated_pair(digit_parse, tag(":"), digit_parse), |(hh, mm)| NaiveTime::from_hms(hh, mm, 0))(input)
}

fn parse_interval(input: &str) -> IResult<&str, TimeRange> {
    map(separated_pair(parse_time, tag("-"), parse_time), TimeRange::from)(input)
}

#[cfg(test)]
mod tests {

    use super::*;


    #[test]
    fn test_parse_time() {
        let (_, time) = parse_time("12:00").expect("can parse time");
        assert_eq!(time, NaiveTime::from_hms(12, 00, 00));
    }

    #[test]
    fn test_parse_interval() {
        let time_range = TimeRange::parse("12:00-13:15").expect("can parse time interval");
        assert_eq!(time_range, TimeRange::new(NaiveTime::from_hms(12, 00, 00), NaiveTime::from_hms(13, 15, 00)))
    }
}