use serde::de;
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

/// Deserialize a string or a list of strings, adapted from
/// https://stackoverflow.com/a/43627388/108112
pub fn string_or_seq<'d, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'d>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'d> de::Visitor<'d> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'d>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}
