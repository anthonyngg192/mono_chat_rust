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
        #[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, OptionalStruct)]
        #[optional_derive(Serialize, Deserialize, JsonSchema, Debug, Default, Clone)]
        #[optional_name = $name]
        #[opt_skip_serializing_none]
        #[opt_some_priority]
        $item
    };
}

pub fn if_false(t: &bool) -> bool {
    !t
}

pub fn if_zero_u32(t: &u32) -> bool {
    t == &0
}

#[macro_export]
#[cfg(debug_assertions)]
macro_rules! query {
    ( $self: ident, $type: ident, $collection: expr, $($rest:expr),+ ) => {
        Ok($self.$type($collection, $($rest),+).await.unwrap())
    };
}
