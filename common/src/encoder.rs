use super::code::{
    EncodedNumber,
    Modulation,
    Number,
    PositiveNumber,
    NegativeNumber,
};

use std::num::ParseIntError;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    BadPrefix,
    BadWidthCode,
    ParseIntError(ParseIntError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadPrefix => write!(f, "Error::BadPrefix"),
            Error::BadWidthCode => write!(f, "Error::BadWidthCode"),
            Error::ParseIntError(_) => write!(f, "Error::ParseIntError"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ParseIntError(e) => Some(e),
            _ => None,
        }
    }
}

pub trait Modulable<T=Self> {
    fn demodulate_from_string(from: &str) -> Result<T, Error>;
    fn modulate_to_string(&self) -> String;
}

fn demodulate_number_from_string_helper(input: &str) -> Result<(EncodedNumber, usize), Error> {
    fn demodulate_number(from: &str) -> Result<(isize, usize), Error> {
        match from.find('0') {
            Some(0) => Ok((0, 1)),
            Some(width) => {
                isize::from_str_radix(&from[width+1..width*4+width+1], 2)
                    .map_err(|err| Error::ParseIntError(err))
                    .map(|num| (num, width * 5 + 1))
                    },
            None => Err(Error::BadWidthCode),
        }
    }

    match &input[0..2] {
        "01" => {
            let (val, consumed) = demodulate_number(&input[2..])?;

            Ok((EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: val as usize,
                    }),
                    modulation: Modulation::Demodulated,
                },
                consumed + 2))
        },
        "10" => {
            let (val, consumed) = demodulate_number(&input[2..])?;

            Ok((EncodedNumber {
                    number: Number::Negative(NegativeNumber {
                        value: -val,
                    }),
                    modulation: Modulation::Demodulated,
                },
                consumed + 2))
        },
        _ => Err(Error::BadPrefix)
    }
}

impl Modulable for EncodedNumber {

    fn demodulate_from_string(input: &str) -> Result<EncodedNumber, Error> {
        demodulate_number_from_string_helper(input).map(|val| val.0)
    }

    fn modulate_to_string(&self) -> String {
        fn modulate_number(num_as_bin: &str, prefix: &str) -> String {
            let quads = (num_as_bin.len() as f32 / 4.0).ceil() as usize;

            // prefix + 1 bit per each 4 bits of value + 1 bit for '0' spacer + 4-bit aligned number
            let mut val = String::with_capacity(prefix.len() + quads + 1 + quads*4);
            val.push_str(prefix);
            val.push_str(&"1".repeat(quads));
            val.push_str("0");
            val.push_str(&"0".repeat(quads*4 - num_as_bin.len()));
            val.push_str(&num_as_bin);

            val
        }

        match self.modulation {
            Modulation::Demodulated =>
                match &self.number {
                    Number::Positive(e) => e.value.to_string(),
                    Number::Negative(e) => e.value.to_string(),
                }
            Modulation::Modulated =>
                match &self.number {
                    Number::Positive(e) if e.value == 0 =>
                        String::from("010"),
                    Number::Negative(e) if e.value == 0 =>
                        String::from("010"),
                    Number::Positive(e) => modulate_number(format!("{:b}",e.value).as_str(), "01"),
                    Number::Negative(e) => modulate_number(format!("{:b}",e.value.abs()).as_str(), "10")
                }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ListVal {
    Number(EncodedNumber),
    Cons(Box<ConsList>),
}

impl ListVal {
    pub fn as_encoded_number(&self) -> &EncodedNumber {
        match self {
            ListVal::Number(n) => n,
            _ => unreachable!(),
        }
    }

    pub fn as_cons(&self) -> &ConsList {
        match self {
            ListVal::Cons(l) => l.as_ref(),
            _ => unreachable!(),
        }
    }

    pub fn as_tuple(&self) -> (&EncodedNumber, &EncodedNumber) {
        match self {
            ListVal::Cons(c) => match c.as_ref() {
                ConsList::Cons(ListVal::Number(l), ListVal::Number(r))
                    => (l, r),
                _ => unreachable!(),
            }
            _ => unreachable!(),
        }
    }
}

impl PrettyPrintable for ListVal {
    fn to_pretty_string(&self) -> String {
        match self {
            ListVal::Cons(c) => c.as_ref().to_pretty_string(),
            ListVal::Number(n) => n.to_pretty_string(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConsList {
    Nil,
    Cons(ListVal, ListVal),
}

impl ConsList {
    pub fn is_nil(&self) -> bool {
        match self {
            ConsList::Nil => true,
            _ => false,
        }

    }
    pub fn car(&self) -> &ListVal {
        match self {
            ConsList::Cons(a, _) => a,
            _ => unreachable!(),
        }
    }

    pub fn cdr(&self) -> &ListVal {
        match self {
            ConsList::Cons(_, a) => a,
            _ => unreachable!(),
        }
    }
}

fn demodulate_list_from_string_helper(input: &str) -> Result<(ConsList, usize), Error> {
    fn demodulate_list_val(from: &str) -> Result<(ListVal, usize), Error> {
        match &from[0..2] {
            "00" => Ok((ListVal::Cons(Box::new(ConsList::Nil)), 2)),
            "01" | "10" => demodulate_number_from_string_helper(from)
                               .map(|(val, consumed)| (ListVal::Number(val), consumed)),
            "11" => demodulate_list_from_string_helper(from)
                               .map(|(val, consumed)| (ListVal::Cons(Box::new(val)), consumed)),
            _ => unreachable!(),
        }
    }

    if &input[0..2] != "11" {
        return Err(Error::BadPrefix);
    }

    let from = &input[2..];

    match from {
        "" => Ok((ConsList::Nil, 2)),  // 2 is for prefix
        _ => {
            let (left, left_consumed) = demodulate_list_val(&from)?;
            let (right, right_consumed) = demodulate_list_val(&from[left_consumed..])?;

            Ok((ConsList::Cons(left, right), left_consumed + right_consumed + 2)) // 2 is for prefix
        }
    }
}

impl Modulable for ConsList {
    fn demodulate_from_string(input: &str) -> Result<ConsList, Error> {
        demodulate_list_from_string_helper(input).map(|val| {
            let (value, consumed) = val;
            assert_eq!(consumed, input.len());
            value
        })
    }

    fn modulate_to_string(&self) -> String {
        fn modulate_val(val: &ListVal) -> String {
            match val {
                ListVal::Number(num) => num.modulate_to_string(),
                ListVal::Cons(c) => match c.as_ref() {
                    ConsList::Nil => String::from("00"),
                    _ => c.as_ref().modulate_to_string(),
                }
            }
        }

        match self {
            ConsList::Nil => String::from("11"),
            ConsList::Cons(v1, v2) => format!("11{}{}", modulate_val(v1), modulate_val(v2)),
        }
    }
}

pub trait PrettyPrintable {
    fn to_pretty_string(&self) -> String;
}

pub fn is_proper_list(value: &ConsList) -> bool {
    match value {
        ConsList::Nil => true,
        ConsList::Cons(_, ListVal::Cons(tail)) => is_proper_list(tail.as_ref()),
        _ => false,
    }
}

impl PrettyPrintable for ConsList {
    fn to_pretty_string(&self) -> String {
        fn print_proper_list(val: &ConsList) -> String{
            match val {
                ConsList::Nil => String::new(),
                ConsList::Cons(ListVal::Number(l), ListVal::Cons(r)) => {
                    format!("{} {}", l.to_pretty_string(), print_proper_list(r.as_ref()))
                }
                ConsList::Cons(ListVal::Cons(l), ListVal::Cons(r)) => {
                    format!("{} {}", l.as_ref().to_pretty_string(), print_proper_list(r.as_ref()))
                },
                _ => unreachable!(),
            }
        }
        let is_list = is_proper_list(self);
        match self {
            ConsList::Nil => "nil".to_string(),
            ConsList::Cons(ListVal::Number(l), ListVal::Number(r)) if !is_list
                => format!("({} . {})", l.to_pretty_string(), r.to_pretty_string()),
            ConsList::Cons(ListVal::Number(l), ListVal::Cons(r)) if !is_list
                => format!("({} . {})", l.to_pretty_string(), r.as_ref().to_pretty_string()),
            ConsList::Cons(ListVal::Cons(l), ListVal::Cons(r)) if !is_list
                => format!("({} . {})", l.as_ref().to_pretty_string(), r.as_ref().to_pretty_string()),
            ConsList::Cons(ListVal::Cons(l), ListVal::Number(r)) if !is_list
                => format!("({} . {})", l.as_ref().to_pretty_string(), r.to_pretty_string()),
            ConsList::Cons(_, _) if is_list => format!("({})", print_proper_list(self).trim()),
            ConsList::Cons(a, b)  => unreachable!("FAIL\nA {:?}\nB: {:?}",a ,b),
        }
    }
}

impl PrettyPrintable for EncodedNumber {
    fn to_pretty_string(&self) -> String {
        match self {
            EncodedNumber { number: Number::Positive ( PositiveNumber { value }), modulation: Modulation::Demodulated }
              => value.to_string(),
            EncodedNumber { number: Number::Negative ( NegativeNumber { value }), modulation: Modulation::Demodulated }
              => value.to_string(),
            EncodedNumber { number: Number::Positive ( PositiveNumber { value }), modulation: Modulation::Modulated }
              => format!("[{}]", value.to_string()),
            EncodedNumber { number: Number::Negative ( NegativeNumber { value }), modulation: Modulation::Modulated }
              => format!("[{}]", value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::{
        PositiveNumber,
        NegativeNumber,
        make_dem_number,
    };

    #[test]
    fn encode_mod_0() {
        let num1 = EncodedNumber {
            number: Number::Positive(PositiveNumber {
                value: 0,
            }),
            modulation: Modulation::Modulated,
        };
        assert_eq!(num1.modulate_to_string(), "010");
        let num2 = EncodedNumber {
            number: Number::Negative(NegativeNumber {
                value: 0,
            }),
            modulation: Modulation::Modulated,
        };
        assert_eq!(num2.modulate_to_string(), "010");
        assert_eq!(num1.modulate_to_string(), num2.modulate_to_string());
    }

    #[test]
    fn encode_mod_1() {
        let num1 = EncodedNumber {
            number: Number::Positive(PositiveNumber {
                value: 1,
            }),
            modulation: Modulation::Modulated,
        };
        assert_eq!(num1.modulate_to_string(), "01100001");
        let num2 = EncodedNumber {
            number: Number::Negative(NegativeNumber {
                value: -1,
            }),
            modulation: Modulation::Modulated,
        };
        assert_eq!(num2.modulate_to_string(), "10100001");
    }

    #[test]
    fn encode_mod_255() {
        let num1 = EncodedNumber {
            number: Number::Positive(PositiveNumber {
                value: 255,
            }),
            modulation: Modulation::Modulated,
        };
        assert_eq!(num1.modulate_to_string(), "0111011111111");
        let num2 = EncodedNumber {
            number: Number::Negative(NegativeNumber {
                value: -255,
            }),
            modulation: Modulation::Modulated,
        };
        assert_eq!(num2.modulate_to_string(), "1011011111111");
    }

    #[test]
    fn modulate_list_msg35() {
        assert_eq!((ConsList::Nil).modulate_to_string(), "11");
        assert_eq!(ConsList::Cons(
                ListVal::Cons(Box::new(ConsList::Nil)),
                ListVal::Cons(Box::new(ConsList::Nil)),
            ).modulate_to_string(), "110000");
        assert_eq!(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 0,
                    }),
                    modulation: Modulation::Modulated,
                }),
                ListVal::Cons(Box::new(ConsList::Nil)),
            ).modulate_to_string(), "1101000");
        assert_eq!(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Modulated,
                }),
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Modulated,
                }),
            ).modulate_to_string(), "110110000101100010");
        assert_eq!(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Modulated,
                }),
                ListVal::Cons(Box::new(ConsList::Cons(
                    ListVal::Number(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 2,
                        }),
                        modulation: Modulation::Modulated,
                    }),
                    ListVal::Cons(Box::new(ConsList::Nil)))))
            ).modulate_to_string(), "1101100001110110001000");
    }

    #[test]
    fn dem_numbers() {
        assert_eq!(EncodedNumber::demodulate_from_string("010"),
                   Ok(EncodedNumber {
                       number: Number::Positive(PositiveNumber {
                           value: 0,
                       }),
                       modulation: Modulation::Demodulated,
                   }));
        assert_eq!(EncodedNumber::demodulate_from_string("01100001"),
                   Ok(EncodedNumber {
                       number: Number::Positive(PositiveNumber {
                           value: 1,
                       }),
                       modulation: Modulation::Demodulated,
                   }));
        assert_eq!(EncodedNumber::demodulate_from_string("10100001"),
                   Ok(EncodedNumber {
                       number: Number::Negative(NegativeNumber {
                           value: -1,
                       }),
                       modulation: Modulation::Demodulated,
                   }));
        assert_eq!(EncodedNumber::demodulate_from_string("0111011111111"),
                   Ok(EncodedNumber {
                       number: Number::Positive(PositiveNumber {
                           value: 255,
                       }),
                       modulation: Modulation::Demodulated,
                   }));
        assert_eq!(EncodedNumber::demodulate_from_string("1011011111111"),
                   Ok(EncodedNumber {
                       number: Number::Negative(NegativeNumber {
                           value: -255,
                       }),
                       modulation: Modulation::Demodulated,
                   }));
    }

    #[test]
    fn dem_lists() {
        assert_eq!(ConsList::demodulate_from_string("11"),
                   Ok(ConsList::Nil));
        assert_eq!(ConsList::demodulate_from_string("110000"),
                   Ok(ConsList::Cons(
                       ListVal::Cons(Box::new(ConsList::Nil)),
                       ListVal::Cons(Box::new(ConsList::Nil)),
                     )));
        assert_eq!(ConsList::demodulate_from_string("110110000101100010"),
                   Ok(ConsList::Cons(
                       ListVal::Number(EncodedNumber {
                           number: Number::Positive(PositiveNumber {
                               value: 1,
                           }),
                           modulation: Modulation::Demodulated,
                       }),
                       ListVal::Number(EncodedNumber {
                           number: Number::Positive(PositiveNumber {
                               value: 2,
                           }),
                           modulation: Modulation::Demodulated,
                       }),
                     )));
        assert_eq!(ConsList::demodulate_from_string("1101100001110110001000"),
                   Ok(ConsList::Cons(
                       ListVal::Number(EncodedNumber {
                           number: Number::Positive(PositiveNumber {
                               value: 1,
                           }),
                           modulation: Modulation::Demodulated,
                       }),
                       ListVal::Cons(Box::new(ConsList::Cons(
                           ListVal::Number(EncodedNumber {
                               number: Number::Positive(PositiveNumber {
                                   value: 2,
                               }),
                               modulation: Modulation::Demodulated,
                           }),
                           ListVal::Cons(Box::new(ConsList::Nil)))))
                      )));
    }

    #[test]
    fn is_list() {
        assert_eq!(is_proper_list(&ConsList::Nil), true);
        assert_eq!(is_proper_list(&ConsList::Cons(
                                       ListVal::Cons(Box::new(ConsList::Nil)),
                                       ListVal::Cons(Box::new(ConsList::Nil)),
            )), true);
        assert_eq!(is_proper_list(&ConsList::Cons(
                                       ListVal::Number(EncodedNumber {
                                           number: Number::Positive(PositiveNumber {
                                               value: 2,
                                           }),
                                           modulation: Modulation::Demodulated,
                                       }),
                                       ListVal::Cons(Box::new(ConsList::Nil)),
            )), true);
        assert_eq!(is_proper_list(&ConsList::Cons(
                                       ListVal::Number(EncodedNumber {
                                           number: Number::Positive(PositiveNumber {
                                               value: 2,
                                           }),
                                           modulation: Modulation::Demodulated,
                                       }),
                                       ListVal::Cons(Box::new(ConsList::Cons(
                                           ListVal::Number(EncodedNumber {
                                               number: Number::Positive(PositiveNumber {
                                                   value: 2,
                                               }),
                                               modulation: Modulation::Demodulated,
                                           }),
                                           ListVal::Cons(Box::new(ConsList::Nil)))))
            )), true);
        assert_eq!(is_proper_list(&ConsList::Cons(
                                       ListVal::Number(EncodedNumber {
                                           number: Number::Positive(PositiveNumber {
                                               value: 2,
                                           }),
                                           modulation: Modulation::Demodulated,
                                       }),
                                       ListVal::Number(EncodedNumber {
                                               number: Number::Positive(PositiveNumber {
                                                   value: 2,
                                               }),
                                               modulation: Modulation::Demodulated,
                                           })
            )), false);
        assert_eq!(is_proper_list(&ConsList::Cons(
                                       ListVal::Number(EncodedNumber {
                                           number: Number::Positive(PositiveNumber {
                                               value: 2,
                                           }),
                                           modulation: Modulation::Demodulated,
                                       }),
                                       ListVal::Cons(Box::new(ConsList::Cons(
                                           ListVal::Number(EncodedNumber {
                                               number: Number::Positive(PositiveNumber {
                                                   value: 2,
                                               }),
                                               modulation: Modulation::Demodulated,
                                           }),
                                           ListVal::Number(EncodedNumber {
                                               number: Number::Positive(PositiveNumber {
                                                   value: 2,
                                               }),
                                               modulation: Modulation::Demodulated,
                                           }))))
            )), false);
    }

    #[test]
    fn number_pretty_print() {
        assert_eq!(EncodedNumber {
            number: Number::Positive(PositiveNumber {
                value: 1,
            }),
            modulation: Modulation::Demodulated,
        }.to_pretty_string(), "1");

        assert_eq!(EncodedNumber {
            number: Number::Negative(NegativeNumber {
                value: -1,
            }),
            modulation: Modulation::Demodulated,
        }.to_pretty_string(), "-1");

        assert_eq!(EncodedNumber {
            number: Number::Positive(PositiveNumber {
                value: 1,
            }),
            modulation: Modulation::Modulated,
        }.to_pretty_string(), "[1]");

        assert_eq!(EncodedNumber {
            number: Number::Negative(NegativeNumber {
                value: -1,
            }),
            modulation: Modulation::Modulated,
        }.to_pretty_string(), "[-1]");
    }

    #[test]
    fn list_pretty_print() {
        assert_eq!(ConsList::Nil.to_pretty_string(), "nil");
        assert_eq!(ConsList::Cons(ListVal::Cons(Box::new(ConsList::Nil)),
                                  ListVal::Cons(Box::new(ConsList::Nil))).to_pretty_string(), "(nil)");
        assert_eq!(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            }),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Demodulated,
                }),
                ListVal::Cons(Box::new(ConsList::Cons(
                    ListVal::Number(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
                        }),
                        modulation: Modulation::Demodulated,
                    }),
                    ListVal::Cons(Box::new(ConsList::Nil))))))))).to_pretty_string(), "(1 2 3)");

        assert_eq!(ConsList::Cons(
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                }),
                ListVal::Cons(Box::new(ConsList::Nil))))),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Demodulated,
                }),
                ListVal::Cons(Box::new(ConsList::Cons(
                    ListVal::Number(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
                        }),
                        modulation: Modulation::Demodulated,
                    }),
                    ListVal::Cons(Box::new(ConsList::Nil))))))))).to_pretty_string(), "((1) 2 3)");

        assert_eq!(ConsList::Cons(
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 1,
                    }),
                    modulation: Modulation::Demodulated,
                }),
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 10,
                    }),
                    modulation: Modulation::Demodulated,
                })))),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Demodulated,
                }),
                ListVal::Cons(Box::new(ConsList::Cons(
                    ListVal::Number(EncodedNumber {
                        number: Number::Positive(PositiveNumber {
                            value: 3,
                        }),
                        modulation: Modulation::Demodulated,
                    }),
                    ListVal::Cons(Box::new(ConsList::Nil))))))))).to_pretty_string(), "((1 . 10) 2 3)");

        assert_eq!(ConsList::Cons(
            ListVal::Number(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: 1,
                }),
                modulation: Modulation::Demodulated,
            }),
            ListVal::Cons(Box::new(ConsList::Cons(
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 2,
                    }),
                    modulation: Modulation::Demodulated,
                }),
                ListVal::Number(EncodedNumber {
                    number: Number::Positive(PositiveNumber {
                        value: 3,
                    }),
                    modulation: Modulation::Demodulated,
                }))))).to_pretty_string(), "(1 . (2 . 3))");
    }

    #[test]
    fn dem_lists_pretty_print_regression1() {
        let regression_lst = ConsList::demodulate_from_string("1101100001110101111011110000100000000110110000111110111100001110000001101100001110111001000000001111011100001000011011101000000000110000110000").unwrap();
        assert_eq!(regression_lst.to_pretty_string(), "(1 0 (256 1 (448 1 64) (16 128) nil) nil)");
    }

    #[test]
    fn dem_lists_regression2() {
        let modul1 = "11011000011101100001111101111000010000000011010111101111000100000000011011000011101110010000000011110111000010000110111010000000001111011100110010011011010001101110000100001101110001000000000111101100010111101110000100001101110100000000011111111011000011101011110111000101110011011011111101000010101111011100110001111011010001101110000100001101110001000000011010110111001000000110110000100111111010111110100001010000000111111010110110000111111011000101101101011011111011000100101111010110101101011011000010011010110111001000000110110000100110000000000";
        let dem1 = ConsList::demodulate_from_string(&modul1);
        let modul2 = "11011000011101100001111101111000010000000011010111101111000100000000011011000011101110010000000011110111000010000110111010000000001111011100110010011011010001101110000100001101110001000000000111101100001111101110000100001101110100000000011111111011000011101011110111000101111011011011111101000010101111011100110010011011010001101110000100001101110001000000011010110111001000000110110000100110000111111010110110000111111011000101111101011011111011000010101111010110101101011011000010011010110111001000000110110000100110000000000";
        let dem2 = ConsList::demodulate_from_string(&modul2);
        assert_ne!(dem1, dem2);
    }

    #[test]
    fn lists_basic_ops() {
        assert_eq!(ConsList::Nil.is_nil(), true);
        assert_eq!(ConsList::Cons(ListVal::Number(make_dem_number(1)), ListVal::Cons(Box::new(ConsList::Nil))).is_nil(), false);
    }
}
