use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(from = "String")]
pub struct Snowflake(pub u64);

impl From<String> for Snowflake {
    fn from(raw: String) -> Self {
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