use serde::{Deserialize, Serialize};

/// Holds a discord id
/// 
/// Discord ids actually contain a timestamp of creation.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
// Discord api gives the number id as a string
#[serde(from = "&str")]
pub struct Snowflake(pub u64);
// TODO: Add support for timestamp


impl From<&str> for Snowflake {
    fn from(raw: &str) -> Self {
       Self(raw.parse().unwrap()) 
    }
}

impl From<u64> for Snowflake {
    fn from(value: u64) -> Self {
       Self(value) 
    }
}

impl From<Snowflake> for u64 {
    fn from(snow: Snowflake) -> Self {
       snow.0 
    }
}

/// Implement `From<...>` for [`Snowflake`]
/// This should be used on structs where there is a `id` field
/// 
/// # Important
/// This macro is used internally, but must be exposed because rust :P
/// You very likely wont need it
/// 
/// # Example
/// ```
/// # use vivcord::{to_snowflake_simple, datatypes::Snowflake};
/// struct SomeData {
///     id: Snowflake
/// }
/// to_snowflake_simple!(SomeData);
/// 
/// let data = SomeData {id: Snowflake(123)};
/// assert_eq!(Snowflake::from(data).0, 123);
/// ```
#[macro_export]
macro_rules! to_snowflake_simple {
    ($st: ty) => {
        impl From<$st> for $crate::datatypes::Snowflake {
            fn from(other: $st) -> Self {
                other.id
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::Snowflake;

    #[test]
    fn test_from_string() {
        let snow = Snowflake::from("123");

        assert_eq!(snow.0, 123);
    }

    #[test]
    fn test_from_u64() {
        let snow = Snowflake::from(123);

        assert_eq!(snow.0, 123);
    }

    #[test]
    fn test_from_macro() {
        struct MyData {
            id: Snowflake
        }
        to_snowflake_simple!(MyData);

        let instance = MyData {id: Snowflake(123)};
        let snow = Snowflake::from(instance);

        assert_eq!(snow.0, 123);
    }
}