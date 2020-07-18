use super::code::{
    EncodedNumber,
    PositiveNumber,
    NegativeNumber,
    Modulation,
    Number,
};


pub enum Error {
    BadPrefix,
}

pub trait Modulable<T=Self> {
    fn demodulate_from_string(from: &str) -> Result<T, Error>;
    fn modulate_to_string(&self) -> String;
}

impl Modulable for EncodedNumber {
    fn demodulate_from_string(from: &str) -> Result<EncodedNumber, Error> {
        unimplemented!("Not implemented yet")

        // match &from[0..1] {
        //     "01" => Ok(EncodedNumber {
        //         number: Number::Positive(PositiveNumber {
        //             value: demodulate_number(&from[2..]) as usize,
        //         }),
        //         modulation: Modulation::Demodulated,
        //     }),
        //     "10" => Ok(EncodedNumber {
        //         number: Number::Negative(NegativeNumber {
        //             value: -demodulate_number(&from[2..]),
        //         }),
        //         modulation: Modulation::Demodulated,
        //     }),
        //     _ => Err(Error::BadPrefix)
        // }

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
