macro_rules! options {
    ($type_name:tt, { $($variant:tt : $variant_str:expr,)* }) => {
        #[derive(core::clone::Clone)]
        pub enum $type_name {
            $($variant),*
        }
        impl $type_name {
            fn string_rep(&self) -> &str {
                match self {
                    $(
                        Self::$variant => $variant_str,
                    )*
                }
            }
        }
        impl core::str::FromStr for $type_name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $variant_str => Ok(Self::$variant),
                    )*
                    _ => Err(())
                }
            }
        }
    };
}

pub(crate) use options;
