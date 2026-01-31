use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

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
        #[derive(Clone, Debug)]
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

            impl $crate::settings::config_utils::Mergeable<[<Partial $name>]> for $name {
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

            impl TryFrom<$crate::settings::config_utils::Value> for [<Partial $name>] {
                type Error = (String, Self);

                fn try_from(value: $crate::settings::config_utils::Value) -> Result<Self, Self::Error> {
                    match value {
                        $crate::settings::config_utils::Value::Object(map) => {
                            let json_value = ::serde_json::to_value(map)
                                .expect("Failed to convert map to JSON value"); // should not happen
                            ::serde_json::from_value(json_value)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
// Note: order of variants matters for correct deserialization
pub enum Value {
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_deserialize() {
        let json_data = r#"
        {
            "name": "Example",
            "enabled": true,
            "threshold": 10.5,
            "count": 42,
            "items": [1, 2, 3],
            "settings": {
                "option1": "value1",
                "option2": false
            }
        }
        "#;

        let parsed: Value = serde_json::from_str(json_data).unwrap();

        if let Value::Object(map) = parsed {
            assert_eq!(map.get("name"), Some(&Value::String("Example".to_string())));
            assert_eq!(map.get("enabled"), Some(&Value::Bool(true)));
            assert_eq!(map.get("threshold"), Some(&Value::Float(10.5)));
            assert_eq!(map.get("count"), Some(&Value::Int(42)));

            if let Some(Value::List(items)) = map.get("items") {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(3));
            } else {
                panic!("Expected 'items' to be a list");
            }

            if let Some(Value::Object(settings)) = map.get("settings") {
                assert_eq!(
                    settings.get("option1"),
                    Some(&Value::String("value1".to_string()))
                );
                assert_eq!(settings.get("option2"), Some(&Value::Bool(false)));
            } else {
                panic!("Expected 'settings' to be an object");
            }
        } else {
            panic!("Expected top-level value to be an object");
        }
    }
}
