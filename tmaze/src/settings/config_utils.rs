use std::fmt::Display;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

pub trait Mergeable<O> {
    fn merge(&mut self, other: &O);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Segment {
    Key(String),
    Index(usize),
}

impl Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Key(key) => write!(f, "{}", key),
            Segment::Index(index) => write!(f, "[{}]", index),
        }
    }
}

pub type Path = Vec<Segment>;

#[derive(Clone, Debug)]
pub struct ConvertError {
    // TODO: Add source file/line info
    pub path: Path,
    pub detail: String,
}

impl std::fmt::Display for ConvertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path_str = self.path.iter().map(Segment::to_string).collect::<Vec<_>>();
        let path_str = path_str.join(".");
        write!(f, "'{}': {}", path_str, self.detail)
    }
}

pub struct ConvertContext {
    pub path: Path,
    pub errors: Vec<ConvertError>,
}

impl ConvertContext {
    pub fn new() -> Self {
        Self {
            path: vec![],
            errors: vec![],
        }
    }

    pub fn at<T>(&mut self, segment: Segment, inside: impl FnOnce(&mut Self) -> T) -> T {
        self.push(segment);
        let t = inside(self);
        self.pop();
        t
    }

    pub fn push(&mut self, segment: Segment) {
        self.path.push(segment);
    }

    pub fn push_key(&mut self, key: &str) {
        self.path.push(Segment::Key(key.to_string()));
    }

    pub fn push_index(&mut self, index: usize) {
        self.path.push(Segment::Index(index));
    }

    pub fn pop(&mut self) {
        self.path.pop();
    }

    pub fn err(&mut self, detail: String) {
        self.errors.push(ConvertError {
            path: self.path.clone(),
            detail,
        });
    }

    pub fn errors(self) -> Vec<ConvertError> {
        self.errors
    }
}

pub trait LenientConvert: Sized {
    fn convert(value: Value, context: &mut ConvertContext) -> Option<Self>;
}

pub trait ConfigValue<O>: Mergeable<O> + LenientConvert + Default {}
impl<T, O> ConfigValue<O> for T where T: Mergeable<O> + LenientConvert + Default {}

#[macro_export]
macro_rules! config {
    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         // this branch must be 1st so that #[nest] is parsed as special attribute
         { #[nest] $(#[$attr:meta])* $field:ident : $type:ty, $($rest:tt)* }
    ) => {
        ::paste::paste! {
            config!{ @step
                $name
                [ $($fields)* $field : [<Partial $type>] = ::std::default::Default::default(), ]
                [ $($rfields)* pub $field : $type, ]
                [ $($pfields)* $(#[$attr])* pub $field : Option<[<Partial $type>]>, ]
                { $($rest)* }
            }
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
            [ $($fields)* $field : $type = ::std::default::Default::default(), ]
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
            [ $($fields)* $field : $type = ($def), ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* $(#[$attr])* pub $field : Option<$type>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:ident : $type:ty = $def_vals:expr),* ,]
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
                    $($fields : $def_vals,)*
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

            impl $crate::settings::config_utils::LenientConvert for [<Partial $name>] {
                fn convert(
                    value: super::config_utils::Value,
                    context: &mut super::config_utils::ConvertContext,
                ) -> Option<Self> {
                    let super::config_utils::Value::Object(mut map) = value else {
                        context.err("expected an object".to_string());
                        return None;
                    };

                    Some(Self {
                        $(
                            $fields: {
                                if let Some(value) = map.remove(stringify!($fields)) {
                                    context.at(
                                        $crate::settings::config_utils::Segment::Key(stringify!($fields).to_string()),
                                        |ctx| <$type as $crate::settings::config_utils::LenientConvert>::convert(value, ctx)
                                    )
                                } else {
                                    None
                                }
                            },
                        )*
                    })
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

#[macro_export]
macro_rules! impl_lenient_prims {
    ($($t:ty => $($variant:ident)+),* $(,)?) => {
        $(impl $crate::settings::config_utils::LenientConvert for $t {
            fn convert(value: super::config_utils::Value, context: &mut super::config_utils::ConvertContext) -> Option<Self> {
                match value {
                    $(super::config_utils::Value::$variant(v) => Some(v as $t),)+
                    _ => {
                        context.err(format!("expected one of: {}", stringify!($($variant),+)));
                        None
                    }
                }
            }
        })*
    };
}

#[macro_export]
macro_rules! impl_lenient_deserialize  {
    ($($t:ty)*) => {
        $(impl $crate::settings::config_utils::LenientConvert for $t {
            fn convert(value: super::config_utils::Value, context: &mut super::config_utils::ConvertContext) -> Option<Self> {
                let json_value = ::serde_json::to_value(&value)
                    .expect("Failed to convert Value to JSON value"); // should not happen
                match ::serde_json::from_value::<$t>(json_value) {
                    Ok(v) => Some(v),
                    Err(e) => {
                        context.err(format!("deserialization error: {}", e));
                        None
                    }
                }
            }
        })*
    }
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
