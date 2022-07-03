//! To simplify text processing and models all the input text from users and
//! data obtained from web sites will be transliterated into Latin script and to upper case register.
use itertools::Itertools;
use std::collections::HashMap;

macro_rules! smap {
    ($map:ident, $to:expr, $($from:expr),+ $(,)? ) => {
        $(
            $map.insert($from, CharOrString::from($to));
        )+
    };
}

#[derive(Hash, PartialEq, Debug, Eq)]
enum CharOrString {
    Char(char),
    String(String),
}

impl From<char> for CharOrString {
    fn from(v: char) -> Self {
        CharOrString::Char(v)
    }
}

impl From<String> for CharOrString {
    fn from(v: String) -> Self {
        CharOrString::String(v)
    }
}

impl From<&'_ str> for CharOrString {
    fn from(v: &'_ str) -> Self {
        CharOrString::String(v.to_owned())
    }
}

#[derive(Debug)]
pub(crate) struct Mapper {
    map: HashMap<char, CharOrString>,
}

impl Mapper {
    pub(crate) fn new() -> Self {
        let mut map = HashMap::new();

        smap![map, 'A', 'А', 'а'];
        smap![map, 'B', 'Б', 'б'];
        smap![map, 'V', 'В', 'в'];
        smap![map, 'G', 'Г', 'г'];
        smap![map, 'D', 'Д', 'д'];
        smap![map, 'Đ', 'Ђ', 'ђ'];
        smap![map, 'E', 'Е', 'е'];
        smap![map, 'Ž', 'Ж', 'ж'];
        smap![map, 'Z', 'З', 'з'];
        smap![map, 'I', 'И', 'и'];
        smap![map, 'J', 'Ј', 'ј'];
        smap![map, 'K', 'К', 'к'];
        smap![map, 'L', 'Л', 'л'];
        smap![map, "Lj", 'Љ', 'љ'];
        smap![map, 'M', 'М', 'м'];
        smap![map, 'N', 'Н', 'н'];
        smap![map, "Nj", 'Њ', 'њ'];
        smap![map, 'O', 'О', 'о'];
        smap![map, 'P', 'П', 'п'];
        smap![map, 'R', 'Р', 'р'];
        smap![map, 'S', 'С', 'с'];
        smap![map, 'T', 'Т', 'т'];
        smap![map, 'Ć', 'Ћ', 'ћ'];
        smap![map, 'U', 'У', 'у'];
        smap![map, 'F', 'Ф', 'ф'];
        smap![map, 'H', 'Х', 'х'];
        smap![map, 'C', 'Ц', 'ц'];
        smap![map, 'Č', 'Ч', 'ч'];
        smap![map, "Dž", 'Џ', 'џ'];
        smap![map, 'Š', 'Ш', 'ш'];

        Self { map }
    }

    pub(crate) fn transoform(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| {
                if let Some(mapped_value) = self.map.get(&c) {
                    match mapped_value {
                        CharOrString::Char(rc) => rc.to_uppercase().to_string(),
                        CharOrString::String(rs) => rs.to_uppercase(),
                    }
                } else {
                    c.to_uppercase().to_string()
                }
            })
            .join("")
    }
}

impl Default for Mapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mapper() {
        let mapper = Mapper::new();

        let output = mapper.transoform("simple text");
        assert_eq!(&output, "SIMPLE TEXT");

        let output = mapper.transoform("");
        assert_eq!(&output, "");

        let output =
            mapper.transoform("У служби грађана - Званична презентација Владе Републике Србије");
        assert_eq!(
            &output,
            "U SLUŽBI GRAĐANA - ZVANIČNA PREZENTACIJA VLADE REPUBLIKE SRBIJE"
        );
    }
}
