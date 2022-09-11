use serde::{Deserialize, Serialize};

/// Holds a discord id
/// 
/// Discord ids actually contain a timestamp of creation.
#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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