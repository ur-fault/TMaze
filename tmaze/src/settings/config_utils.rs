pub trait Mergeable<O> {
    fn merge(&mut self, other: &O);
}

#[macro_export]
macro_rules! config {
    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         // this branch must be 1st so that #[nest] is parsed as special attribute
         { #[nest] $(#[$attr:meta])* $field:ident : $type:ty, $($rest:tt)* }
    ) => {
        config!{ @step
            $name
            [ $($fields)* $field = ::std::default::Default::default(), ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* $(#[$attr])* pub $field : Option<[<Partial $type>]>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { $(#[$attr:meta])* $field:ident : $type:ty, $($rest:tt)* }
    ) => {
        config!{ @step
            $name
            [ $($fields)* $field = ::std::default::Default::default(), ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* $(#[$attr])* pub $field : Option<$type>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { $(#[$attr:meta])* $field:ident : $type:ty = $def:expr, $($rest:tt)* }
    ) => {
        config!{ @step
            $name
            [ $($fields)* $field = ($def), ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* $(#[$attr])* pub $field : Option<$type>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:ident = $def_vals:expr),* ,]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { }
    ) => {
        pub struct $name {
            $($rfields)*
        }

        impl ::std::default::Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $fields : $def_vals,
                    )*
                }
            }
        }

        ::paste::paste! {
            #[derive(Default, Clone, Serialize, Deserialize)]
            pub struct [<Partial $name>] {
                $($pfields)*
            }

            impl Mergeable<[<Partial $name>]> for $name {
                fn merge(&mut self, other: &[<Partial $name>]) {
                    $(
                        if let Some(value) = &other.$fields {
                            self.$fields.merge(value);
                        }
                    )*
                }
            }

            impl From<&[<Partial $name>]> for $name {
                fn from(partial: &[<Partial $name>]) -> Self {
                    let mut config = Self::default();
                    config.merge(partial);
                    config
                }
            }

            impl TryFrom<Value> for [<Partial $name>] {
                type Error = (String, Self);

                fn try_from(value: Value) -> Result<Self, Self::Error> {
                    match value {
                        Value::Object(map) => {
                            let json_value = serde_json::to_value(map)
                                .expect("Failed to convert map to JSON value"); // should not happen
                            serde_json::from_value(json_value)
                                .map_err(|e| (e.to_string(), Self::default()))
                        }
                        _ => Err(("Expected an object for partial config".to_string(), Self::default())),
                    }
                }
            }
        }
    };

    ($(pub struct $name:ident { $($body:tt)* })*) => {
        $(config!{ @step $name [] [] [] { $($body)* } })*
    };
}

#[macro_export]
macro_rules! impl_merge_prims {
    ($($t:ty)*) => {
        $(impl Mergeable<$t> for $t where $t: Clone {
            fn merge(&mut self, other: &$t) {
                *self = other.clone();
            }
        })*
    };
}
