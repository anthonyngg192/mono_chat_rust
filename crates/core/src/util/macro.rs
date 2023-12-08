pub use validator::Validate;

#[macro_export]
macro_rules! auto_derived {
    ( $( $item:item )+ ) => {
        $(
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            #[cfg_attr(feature = "schemas", derive(JsonSchema))]
            #[derive(Debug, Clone)]
            $item
        )+
    };
}

#[macro_export]
macro_rules! auto_derived_partial {
    ( $item:item, $name:expr ) => {
        #[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct, Default)]
        #[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

/// Utility function to check if an u32 is zero
pub fn if_zero_u32(t: &u32) -> bool {
    t == &0
}
