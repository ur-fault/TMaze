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
