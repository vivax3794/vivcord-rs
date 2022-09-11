/// Implement `From<...>` for [`Snowflake`][crate::datatypes::Snowflake]
/// This should be used on structs where there is a `id` field
macro_rules! to_snowflake_simple {
    ($st: ty) => {
        impl From<$st> for $crate::datatypes::Snowflake {
            fn from(other: $st) -> Self {
                other.id
            }
        }
    };
}

/// Create trait implementations of [`PartialEq`], [`Eq`], [`PartialOrd`] and [`Ord`] using a given fields.
///
macro_rules! comp_by_field {
    ($st: ty, self.$field:ident ) => {
        impl PartialEq for $st {
            fn eq(&self, other: &$st) -> bool {
                self.$field == other.$field
            }
        }

        impl Eq for $st {}

        impl PartialOrd for $st {
            fn partial_cmp(&self, other: &$st) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $st {
            fn cmp(&self, other: &$st) -> std::cmp::Ordering {
                self.$field.cmp(&other.$field)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    struct TestingStruct {
        field: u8,
    }
    comp_by_field!(TestingStruct, self.field);

    #[test]
    fn test_eq() {
        assert!(TestingStruct { field: 1 } == TestingStruct { field: 1 });
    }

    #[test]
    fn test_ne() {
        assert!(TestingStruct { field: 1 } != TestingStruct { field: 0 });
    }

    #[test]
    fn test_cmp_gt() {
        assert!(TestingStruct { field: 1} > TestingStruct { field: 0}); 
    }
    
    #[test]
    fn test_cmp_lt() {
        assert!(TestingStruct { field: 0} < TestingStruct { field: 1}); 
    }
}
