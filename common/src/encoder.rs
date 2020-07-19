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

impl Modulable for EncodedNumber {
    fn demodulate_from_string(input: &str) -> Result<EncodedNumber, Error> {
        fn demodulate_number(from: &str) -> Result<isize, Error> {
            match from.find('0') {
                Some(width) => {
                    isize::from_str_radix(&from[width..], 2).map_err(|err| Error::ParseIntError(err))
                },
                None => Err(Error::BadWidthCode),
            }
        }
        match &input[0..2] {
            "01" => Ok(EncodedNumber {
                number: Number::Positive(PositiveNumber {
                    value: demodulate_number(&input[2..])? as usize,
                }),
                modulation: Modulation::Demodulated,
            }),
            "10" => Ok(EncodedNumber {
                number: Number::Negative(NegativeNumber {
                    value: -demodulate_number(&input[2..])?,
                }),
                modulation: Modulation::Demodulated,
            }),
            _ => Err(Error::BadPrefix)
        }

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
enum ListVal {
    Number(EncodedNumber),
    Cons(Box<ConsList>),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum ConsList {
    Nil,
    Cons(ListVal, ListVal),
}

impl Modulable for ConsList {
    fn demodulate_from_string(_from: &str) -> Result<ConsList, Error> {
        unimplemented!("Not implemented yet")
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
}
