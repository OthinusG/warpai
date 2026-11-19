use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use std::{fmt, marker::PhantomData};

pub(crate) fn bounded_sequence<'de, D, T, const MAX: usize>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct Bounded<T, const MAX: usize>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>, const MAX: usize> Visitor<'de> for Bounded<T, MAX> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "at most {MAX} terminal entries")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let count = sequence.size_hint().unwrap_or_default();
            if count > MAX {
                return Err(A::Error::invalid_length(count, &self));
            }
            let mut entries = Vec::with_capacity(count);
            while let Some(entry) = sequence.next_element()? {
                if entries.len() == MAX {
                    return Err(A::Error::invalid_length(MAX + 1, &self));
                }
                entries.push(entry);
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_seq(Bounded::<T, MAX>(PhantomData))
}
