//! The module declares structures to hold the address information and
//! a set of functions to parse the raw data.
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_until1};
use nom::character::complete::{alpha0, digit1, multispace0};
use nom::combinator::{map, map_res, opt, recognize, value};
use nom::error::Error;
use nom::multi::{many1, separated_list0};
use nom::sequence::{delimited, pair, separated_pair};
use nom::{Err, IResult};

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct BrojNumber<'a> {
    value: usize,
    extension: Option<&'a str>,
}

impl<'a> From<(usize, Option<&'a str>)> for BrojNumber<'a> {
    fn from((v, e): (usize, Option<&'a str>)) -> Self {
        Self {
            value: v,
            extension: e,
        }
    }
}

impl<'a> From<usize> for BrojNumber<'a> {
    fn from(v: usize) -> Self {
        Self {
            value: v,
            extension: None,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct BrojRange<'a> {
    from: BrojNumber<'a>,
    to: BrojNumber<'a>,
}

impl<'a> From<(usize, usize)> for BrojRange<'a> {
    fn from((from, to): (usize, usize)) -> Self {
        Self {
            from: BrojNumber::from(from),
            to: BrojNumber::from(to),
        }
    }
}

impl<'a> From<((usize, Option<&'a str>), (usize, Option<&'a str>))> for BrojRange<'a> {
    fn from((from, to): ((usize, Option<&'a str>), (usize, Option<&'a str>))) -> Self {
        Self {
            from: BrojNumber::from(from),
            to: BrojNumber::from(to),
        }
    }
}

impl<'a> From<(BrojNumber<'a>, BrojNumber<'a>)> for BrojRange<'a> {
    fn from((from, to): (BrojNumber<'a>, BrojNumber<'a>)) -> Self {
        Self { from, to }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) enum Broj<'a> {
    Bez,
    Number(BrojNumber<'a>),
    Range(BrojRange<'a>),
}

impl<'a> From<BrojNumber<'a>> for Broj<'a> {
    fn from(v: BrojNumber<'a>) -> Self {
        Broj::Number(v)
    }
}

impl<'a> From<BrojRange<'a>> for Broj<'a> {
    fn from(v: BrojRange<'a>) -> Self {
        Broj::Range(v)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct AddressRecord<'a> {
    street: &'a str,
    numbers: Vec<Broj<'a>>,
}

impl<'a> AddressRecord<'a> {
    pub(crate) fn new(street: &'a str, numbers: Vec<Broj<'a>>) -> Self {
        Self { street, numbers }
    }
}

impl<'a> From<(&'a str, Vec<Broj<'a>>)> for AddressRecord<'a> {
    fn from((street, numbers): (&'a str, Vec<Broj<'a>>)) -> Self {
        Self { street, numbers }
    }
}

/// Parser a regular address number with optional extension letter.
fn address_number(input: &str) -> IResult<&str, BrojNumber<'_>> {
    let digit_parser = map_res(digit1, |s: &str| s.parse::<usize>());
    let ext_parser = map(
        recognize(pair(alpha0, opt(pair(tag("/"), digit1)))),
        |x: &str| if !x.is_empty() { Some(x) } else { None },
    );
    map(pair(digit_parser, ext_parser), BrojNumber::from)(input)
}

/// Parse a range of addresses
fn address_number_range(input: &str) -> IResult<&str, BrojRange<'_>> {
    let parser = separated_pair(address_number, tag("-"), address_number);
    map(parser, BrojRange::from)(input)
}

/// Parses an address number, a range of addresses or a special BB case.
fn broj(input: &str) -> IResult<&str, Broj<'_>> {
    let bb_parser = value(Broj::Bez, tag_no_case("bb"));
    let number_parser = map(address_number, Broj::from);
    let range_parser = map(address_number_range, Broj::from);

    alt((bb_parser, range_parser, number_parser))(input)
}

/// Recognizes a list of addresses, ranges of addresses or special BB cases.
fn broj_list(input: &str) -> IResult<&str, Vec<Broj<'_>>> {
    let parser = separated_list0(tag(","), broj);
    delimited(
        multispace0,
        // potentially we can simply skip the second element of the pair (the trailing comma)
        map(pair(parser, opt(tag(","))), |(x, _)| x),
        multispace0,
    )(input)
}

/// Recognizes a pair of an address and the list of addresses' numbers.
fn address_number_pair(input: &str) -> IResult<&str, AddressRecord<'_>> {
    let take_pp = take_until1(":");
    map(separated_pair(take_pp, tag(":"), broj_list), |(a, b)| {
        AddressRecord::new(a.trim(), b)
    })(input)
}

/// Parse addresses info (row).
fn addresses(input: &str) -> IResult<&str, Vec<AddressRecord<'_>>> {
    many1(address_number_pair)(input)
}

#[derive(Eq, PartialEq, Debug)]
#[repr(transparent)]
pub(crate) struct Addresses<'a> {
    items: Vec<AddressRecord<'a>>,
}

impl<'a> Addresses<'a> {
    #[inline(always)]
    pub(crate) fn parse(input: &'a str) -> Result<Addresses<'a>, Err<Error<&str>>> {
        match addresses(input) {
            Ok((_, items)) => Ok(Self { items }),
            Err(err) => Err(err),
        }
    }
}

impl<'a> IntoIterator for Addresses<'a> {
    type Item = <Vec<AddressRecord<'a>> as IntoIterator>::Item;
    type IntoIter = <Vec<AddressRecord<'a>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_can_parse_complicated_address() {
        let res = address_number("36A/1").expect("parse the compilcated regular address");
        assert_eq!(res, ("", BrojNumber::from((36, Some("A/1")))));
    }

    #[test]
    fn test_can_parse_a_range_of_addresses() {
        let res = address_number_range("123-321").expect("parse the range of addresses");
        assert_eq!(res, ("", BrojRange::from((123, 321))))
    }

    #[test]
    fn test_can_parse_one_of() {
        let res = broj("BB").expect("can recognize BB");
        assert_eq!(res, ("", Broj::Bez));

        let res = broj("123A").expect("can recognize an address number");
        assert_eq!(res, ("", Broj::from(BrojNumber::from((123, Some("A"))))));

        let res = broj("123A-321B").expect("can recognize an addresses range");
        assert_eq!(
            res,
            (
                "",
                Broj::from(BrojRange::from(((123, Some("A")), (321, Some("B")))))
            )
        );
    }

    #[test]
    fn test_can_parse_numbers_sequences() {
        let res = broj_list("BB,123,123-321").expect("parse the sequence of numbers");
        assert_eq!(
            res,
            (
                "",
                vec![
                    Broj::Bez,
                    Broj::Number(BrojNumber::from(123)),
                    Broj::Range(BrojRange::from((123, 321))),
                ]
            )
        )
    }

    #[test]
    fn test_ignores_trailing_whitespaces() {
        let res =
            broj_list("   BB,BB   ").expect("rejects whitespaces before and after the sequence");
        assert_eq!(res, ("", vec![Broj::Bez, Broj::Bez,]))
    }

    #[test]
    fn test_can_recognize_trailing_comma() {
        let res = broj_list("BB,")
            .expect("parse the sequence of addresses followed by the trailing comma");
        assert_eq!(res, ("", vec![Broj::Bez,]));
    }

    #[test]
    fn test_reject_simple_comma() {
        let res = broj_list("   ,   ");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse() {
        let res = address_number_pair("  AUTOPUT ZA NOVI SAD  : BB,284,294-296F,").unwrap();
        assert_eq!(
            res,
            (
                "",
                AddressRecord::new(
                    "AUTOPUT ZA NOVI SAD",
                    vec![
                        Broj::Bez,
                        Broj::Number(BrojNumber::from(284)),
                        Broj::Range(BrojRange::from(((294, None), (296, Some("F"))))),
                    ]
                )
            )
        );
    }

    #[test]
    fn test_parse_all_addresses() {
        let res = addresses("AUTOPUT ZA NOVI SAD: BB,284,294-296F,  BATAJNI??KI DRUM: BB,261-265,269,283-293,299,303-303A,").expect("parse the address row");

        assert_eq!(
            res,
            (
                "",
                vec![
                    AddressRecord::new(
                        "AUTOPUT ZA NOVI SAD",
                        vec![
                            Broj::Bez,
                            Broj::from(BrojNumber::from(284)),
                            Broj::from(BrojRange::from(((294, None), (296, Some("F"))))),
                        ]
                    ),
                    AddressRecord::new(
                        "BATAJNI??KI DRUM",
                        vec![
                            Broj::Bez,
                            Broj::from(BrojRange::from((261, 265))),
                            Broj::from(BrojNumber::from(269)),
                            Broj::from(BrojRange::from((283, 293))),
                            Broj::from(BrojNumber::from(299)),
                            Broj::from(BrojRange::from(((303, None), (303, Some("A"))))),
                        ]
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_full_row_test() {
        static TEST_INPUT: &str = "AUTOPUT ZA NOVI SAD: BB,284,294-296F,  BATAJNI??KI DRUM: BB,261-265,269,283-293,299,303-303A,  BATAJNI??KI DRUM 14 DEO: 14,  NIKOLE SUKNJAREVI??A PRIKE: 2-18,1-17, NASELJE BATAJNICA:   1 SREMSKOG ODREDA: 2-90,1-89,  AERODROMSKA: 68A-80,84-88I,98,1-1A,5-13,23A,  BANOVA??KA: 4A-12,20,24,28,34,1-1A,  BATAJNI??KIH ??RTAVA: 2-16,1-13,  BATINSKE BITKE: 2-60,1-59,  BE??EJSKA: 22-26,30-32,42-44,  BIHA??KA: 2-28,1-7,  BOJ??INSKA: 1-15,  BOSANSKE KRAJINE: 2-86,1-73,  BRA??E BARI??I??A: 2-18,1-3,7-19,  BRA??E MIHAJLOVI??-TRIPI??: 6-106,43-45,49-51,  BRA??E NE??TINAC: 2-12,1-11,  BRA??E RADI??I??: 4A-4V,1-41,  BRA??E SAVI??A: 2-54,1-73,  BRA??E SMILJANI??A: 4-6,14-64,68-72,3-11,15-61A,65A-71,75B-83F/3,  BRA??E VOJINOVI??A: 1-1A,5-7A,11-11A,15-15A,19-19A,  BRANISLAVA BARI??I??A: 2-92,1-43,47,53-57,  BRILETOVA: 2-6,1-5,  BRODSKA: 2-18,1-19,  CARICE JELENE : 2-26,1-27,  DALMATINSKE ZAGORE : 8-16,20-160,1-145B,  DALMATINSKIH BRIGADA: 4A-6A,10-12,20,24,36A/1-56,60,19-21A,25-33,39-43,47,51,57-57,61-85,89-91,97-103,  DESPOTA IVANI??A : 10-12,18-24,  DIMITRIJA LAZAROVA RA??E: 2-28,1-33,37-41,  DISKONT PKB NOVA 21: 2-14,17,  ??OR??A BO??KOVI??A - BATE: 6,12-16B,26-36A,42-54,3-19B,23-39,43-63B,  DRAGE MIHAJLOVI??A: 2-58,1-47,51-53,  ??UR??A BAL??I??A : 2-6,10-20,3,9,  ISLAMA GR??KOG: 4,8,12-18,17,21-31,  IVANA DELNEGRA-ENGLEZA : 2-42,1-17,  IVANA SENKOVI??A: 2-78,1-73,  JOVANA BRANKOVI??A : 2-118,122-152,156-166C,170,174-176D,180-182,1-137,141-155,161-171,  KARLOV??I??KA: 2-6,1-5,  KATICE OPA??I??: 2-18,22-40,44-46,50,72,76,94-94D,98-104D,1,5-11,17-17,25A-69B,  KESARA HRELJE : 2,12-24,28,34-40,  KESARA NOVAKA : 2-14A,  KESARA PRELJUBA : 4-8,12,20,24-26,30-36,3-9,13-25,  KESARA VOJIHNE : 4-6,3-23,27-33,  KLISINA NOVA  8: 2,3,7-17,  KLISINA NOVA  9: 2A,6,10,14,18-20,3-5,9-9A,13-17,  KNEZA PASKA??A : 2-14,18,1-5,  KRALJA MIHAILA ZETSKOG : 2-4,8-24,30-32,48-52,1-11,45-47,51-67O,73-83,87,  KRALJA RADOSLAVA : 38-120,126-148,152-178,53-81,85-85,99-99N,105-181,  KRALJA STEFANA TOMA??A : 40-42,48-58,64-66,67-89,  KRALJA URO??A PRVOG : 2-16G,1,9A,  KRALJA VLADISLAVA : 22-42,46-50B,54-102,106,110-116,120-150,13-29,33-35,39-43,47-61,65-73,77-117,121-129,133-139,  KULSKA: 23-29E,  MAJKE JUGOVI??A: 16-16A,30-36,11-11E,99N,  MAJORA ZORANA RADOSAVLJEVI??A : 2-50,116-226,236-258B,262-290,372-374,382,1-49,117-143,149-277,281,  MAKSIMA BRANKOVI??A : 2-26,30,38-56,1-3,7-47,  MALA: 2-10,1,  MARKA PERI??INA-KAMENJARA : 2-8A,16,24-26,32,42-70,1,25,39-43,  MATROZOVA: BB,  MIHALJEVA??KA: 2-20,1-19,  MILICE RAKI?? : 2-96,3-21,39-79,83-117,  MITROVA??KA: 2-26,1-27,  MRCINI??TE NOVA 28: 2-10,14-16,24-36,3-27,  NATALIJE DUBAJI??: 2-6A,1-11,  NIKICE POPOVI??A: 2-18,1-13,  NOVAKA ATANACKOVI??A: 2-6,1-3,  NOVOSADSKA : 10-98,1-41,45-47,51-61D,65-75??,81D-81E,97G-99J,103A-109V,  OFICIRSKA KOLONIJA : 4-10,14-16,1-9,13-17,  PALI??KA: 2-52,1-83,  PE??INA??KA: 2-76,1-39,  PILOTSKA: 2-20,1-19,  PRIMO??TENSKA: 3,11,19-21,  PUKOVNIKA MILENKA PAVLOVI??A : 2-142,160-162,180,1-9A,13-127,143-159A,175,  RATARSKA: 2-42,1-39,  ROMSKA: 2,14-16,23,  SAVE GRKINI??A: 2-30,1-33,43,  SAVE RADOVANOVI??A: 2-2A,6-8A,12-12A,16,20-20A,1-5,15-17,  SEVASTOKRATORA BRANKA : 2-90,1-89,  SEVASTOKRATORA DEJANA : 2-36,40,1,9-43,47-49,  SEVASTOKRATORA VLATKA : 2-68,1-79,  ??IMANOVA??KA: 2-80,1-55,  ??IROKI PUT: 2-16A,36,1-19,31E-31K,  ??KOLSKA: 2-6,1-5,  SLOBODANA MACURE : 2-4,8-12,1-15,33-37,41-69,  SREMSKOG FRONTA: 2-20,1-9,13-25,  STANKA TI??ME: 2-84G,31A-47,71B-85V,  STEVANA DUBAJI??A : 2A-42,46-48,52-68,74-82,1-17,21-29,33-73,79-81,85-91,  STEVE STANKOVI??A: 2-18,1-11,  STOJANA BO??KOVI??A : 1,5-17,  SUNCOKRETA: 2-6,10-14,20,24,28-30,  SVETISLAVA VULOVI??A : 4,10,14-18,27-33,37,  SVETOG RAFAILA ??I??ATOVA??KOG : 2-12,1-15,  SVETOG SERAFIMA SAROVSKOG : 2-12,1-15,  SVILAJSKA: 2-12,1-9,  TITELSKA: 10-12,18-20,  VASILIJA RANKOVI??A-BA??E: 2-12,1-5,9-19,  VERE MI????EVI??: 2-30,5-15,19-33,  VOJVO??ANSKIH BRIGADA: 2-34,44-134,1-37,41-87,91-139A,143-145Z,  VOJVODE JAK??E : 2-10,  VOJVODE NOVAKA : 2-6,10,44-46E,1-29F,33-39V,55M-55N,61B,  VOJVODE VOJISLAVA VOJINOVI??A : 2-16,1-9,  VOJVODE VRATKA : BB,4-28,1A-37,  ??ARKA BOKUNA: 2-104,108-110,1-11,49,61-103,121-129,  ??ARKA OBRE??KOG: 2-14,18-20,30,34-34,38-40,44,1A-29E,33-41B,  ??IKE MARKOVI??A: 2-10,1-13,  ??UPANA PRIBILA : 2-36,1-31, NASELJE ZEMUN:   BATAJNI??KI DRUM 13 DEO: 301,  KLISINA NOVA 10: 8-10,  TEMERINSKA 1 DEO: 1,";
        let res = addresses(TEST_INPUT);
        assert!(res.is_ok())
    }
}
