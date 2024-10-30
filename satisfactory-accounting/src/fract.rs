use std::fmt;
use std::str::FromStr;

use num_rational::Rational64;
use serde::{Deserialize, Serialize};

/// Custom fraction wrapper which adds features that are useful to us.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Fract {
    /// Inner num::rational.
    ratio: Rational64,
}

impl Fract {
    /// Fraction value of 0.
    pub const ZERO: Fract = Fract {
        ratio: Rational64::ZERO,
    };

    /// Create a new fraction with the given numerator and denominator. Panics if the denominator is
    /// zero.
    pub fn new(numer: i64, denom: i64) -> Self {
        Self {
            ratio: Rational64::new(numer, denom),
        }
    }

    /// Creates an integer.
    pub fn int(numer: i64) -> Self {
        Self::new(numer, 1)
    }

    /// Return true if this is an integer.
    pub fn is_integer(&self) -> bool {
        self.ratio.is_integer()
    }

    /// Gets the numerator.
    pub fn numer(&self) -> i64 {
        *self.ratio.numer()
    }

    /// Gets the denominator.
    pub fn denom(&self) -> i64 {
        *self.ratio.denom()
    }

    // Helper to get numerator and denominator as a tuple of (numer, denom).
    fn pair(&self) -> (i64, i64) {
        self.ratio.into()
    }
}

impl Serialize for Fract {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.is_integer() {
            serializer.serialize_i64(self.numer())
        } else {
            self.pair().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Fract {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, IgnoredAny, Unexpected, Visitor};
        struct FractVisitor;

        impl<'d> Visitor<'d> for FractVisitor {
            type Value = Fract;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "an integer, a string of format \"numer/denom\", or a 2-tuple of (numer, denom)"
                )
            }

            /// If we just get an i64, the value is an integer.
            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Fract::int(value))
            }

            /// If we get a string, parse as numer/denom.
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Fract::from_str(v).map_err(|_| {
                    E::invalid_value(
                        Unexpected::Str(v),
                        &"an integer string or a string in the form numer/denom",
                    )
                })
            }

            /// If we get a pair, assumit it is [numer, denom].
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'d>,
            {
                const WRONG_LEN: &'static str =
                    "a sequence of 2 elements (numerator and denominator)";
                let numer: i64 = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::invalid_length(0, &WRONG_LEN))?;
                let denom: i64 = seq
                    .next_element()?
                    .ok_or_else(|| A::Error::invalid_length(1, &WRONG_LEN))?;
                let mut extra = 0;
                while let Some(_) = seq.next_element::<IgnoredAny>()? {
                    extra += 1;
                }
                if extra > 0 {
                    return Err(A::Error::invalid_length(2 + extra, &WRONG_LEN));
                }
                if denom == 0 {
                    Err(A::Error::invalid_value(
                        Unexpected::Signed(denom),
                        &"a non-zero denominator",
                    ))
                } else {
                    Ok(Fract::new(numer, denom))
                }
            }
        }

        deserializer.deserialize_i64(FractVisitor)
    }
}

impl FromStr for Fract {
    type Err = <Rational64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { ratio: s.parse()? })
    }
}

macro_rules! forward_fmt {
    ($($fmt:path),* $(,)?) => {
        $(
            impl $fmt for Fract {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    <Rational64 as $fmt>::fmt(&self.ratio, f)
                }
            }
        )*
    }
}
forward_fmt!(
    fmt::Debug,
    fmt::Display,
    fmt::Binary,
    fmt::Octal,
    fmt::LowerHex,
    fmt::UpperHex,
    fmt::LowerExp,
    fmt::UpperExp,
);

macro_rules! from_int {
    ($($t:ty),* $(,)?) => {
        $(
            impl From<$t> for Fract {
                fn from(value: $t) -> Self {
                    Self::int(value as i64)
                }
            }
        )*
    }
}

from_int!(i8, i16, i32, i64);

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

    #[test]
    fn equality_check() {
        assert_eq!(Fract::new(-1, 1), Fract::new(1, -1));
        assert_eq!(Fract::new(-2, 2), Fract::new(1, -1));
        assert_eq!(Fract::new(4, 3), Fract::new(-8, -6));
    }

    #[test]
    fn reduction_check() {
        assert_eq!(Fract::new(8, 6).pair(), (4, 3));
        assert_eq!(Fract::new(-8, 6).pair(), (-4, 3));
        assert_eq!(Fract::new(8, -6).pair(), (-4, 3));
        assert_eq!(Fract::new(-8, -6).pair(), (4, 3));
    }

    #[test]
    fn ser_de_as_int() {
        assert_tokens(&Fract::int(10), &[Token::I64(10)]);
        assert_tokens(&Fract::int(-300), &[Token::I64(-300)]);
        assert_tokens(&Fract::new(990, -10), &[Token::I64(-99)]);
        assert_tokens(&Fract::ZERO, &[Token::I64(0)]);
    }

    #[test]
    fn ser_de_as_seq() {
        assert_tokens(
            &Fract::new(8, 6),
            &[
                Token::Tuple { len: 2 },
                Token::I64(4),
                Token::I64(3),
                Token::TupleEnd,
            ],
        );
    }

    #[test]
    fn de_wrong_len() {
        assert_de_tokens_error::<Fract>(
            &[Token::Tuple { len: 0 }, Token::TupleEnd],
            "invalid length 0, expected a sequence of 2 elements (numerator and denominator)",
        );
        assert_de_tokens_error::<Fract>(
            &[Token::Tuple { len: 1 }, Token::I64(3), Token::TupleEnd],
            "invalid length 1, expected a sequence of 2 elements (numerator and denominator)",
        );
        assert_de_tokens_error::<Fract>(
            &[
                Token::Tuple { len: 3 },
                Token::I64(5),
                Token::I64(7),
                Token::I64(8),
                Token::TupleEnd,
            ],
            "invalid length 3, expected a sequence of 2 elements (numerator and denominator)",
        );
        assert_de_tokens_error::<Fract>(
            &[
                Token::Tuple { len: 4 },
                Token::I64(5),
                Token::I64(3),
                Token::I64(7),
                Token::I64(8),
                Token::TupleEnd,
            ],
            "invalid length 4, expected a sequence of 2 elements (numerator and denominator)",
        );
    }

    #[test]
    fn de_from_str() {
        assert_de_tokens(&Fract::int(-154), &[Token::Str("-154")]);
        assert_de_tokens(&Fract::int(5404), &[Token::Str("5404")]);
        assert_de_tokens(&Fract::ZERO, &[Token::Str("0")]);
        assert_de_tokens(&Fract::ZERO, &[Token::Str("0/100")]);
        assert_de_tokens(&Fract::new(3, 7), &[Token::Str("6/14")]);
    }

    #[test]
    fn de_from_str_err() {
        assert_de_tokens_error::<Fract>(
            &[Token::Str("0xffff/0b1110")],
            "invalid value: string \"0xffff/0b1110\", expected an integer string or a string in \
            the form numer/denom",
        );
        assert_de_tokens_error::<Fract>(
            &[Token::Str("9/0")],
            "invalid value: string \"9/0\", expected an integer string or a string in the form \
            numer/denom",
        );
    }
}
