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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConsList {
    Nil,
    Cons(ListVal, ListVal),
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
        "" => Ok((ConsList::Nil, 2)),
        _ => {
            let (left, left_consumed) = demodulate_list_val(&from)?;
            let (right, right_consumed) = demodulate_list_val(&from[left_consumed..])?;

            Ok((ConsList::Cons(left, right), left_consumed + right_consumed))
        }
    }
}



impl Modulable for ConsList {
    fn demodulate_from_string(input: &str) -> Result<ConsList, Error> {
        demodulate_list_from_string_helper(input).map(|val| val.0)
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::code::{
        PositiveNumber,
        NegativeNumber,
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
}
