macro_rules! simple_enum {
    ($(
        $(#[$attr:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident),*
        }
    )*) => {$(
        $(#[$attr])*
        $vis enum $name {
            $($variant),*
        }
        
        impl $name {
            pub const NUM: usize = [$(Self::$variant),*].len();
            pub const ALL: [Self; Self::NUM] = [$(Self::$variant),*];
    
            pub const fn try_index(index: usize) -> Option<Self> {
                $(#[allow(non_upper_case_globals, unused)]
                const $variant: usize = $name::$variant as usize;)*
                #[allow(non_upper_case_globals)]
                match index {
                    $($variant => Option::Some(Self::$variant),)*
                    _ => Option::None
                }
            }

            pub fn index(index: usize) -> Self {
                Self::try_index(index).unwrap_or_else(|| panic!("Index {} is out of range.", index))
            }

            pub const fn index_const(index: usize) -> Self {
                if let Some(value) = Self::try_index(index) {
                    value
                } else {
                    [/* Index is out of range */][index]
                }
            }
        }
    )*};
}
pub(crate) use simple_enum;

macro_rules! enum_char_conv {
    ($(
        $enum:ident, $error:ident {
            $($variant:ident = $char:expr),*
        }
    )*) => {$(
        impl From<$enum> for char {
            fn from(value: $enum) -> Self {
                match value {
                    $($enum::$variant => $char),*
                }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum $error {
            InvalidValue
        }

        impl std::convert::TryFrom<char> for $enum {
            type Error = $error;

            fn try_from(value: char) -> Result<Self, Self::Error> {
                match value {
                    $($char => Ok(Self::$variant),)*
                    _ => Err($error::InvalidValue)
                }
            }
        }

        impl std::str::FromStr for $enum {
            type Err = $error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use std::convert::TryInto;
                
                let mut chars = s.chars();
                let c = chars.next().ok_or($error::InvalidValue)?;
                if chars.next().is_none() {
                    c.try_into()
                } else {
                    Err($error::InvalidValue)
                }
            }
        }

        impl std::fmt::Display for $enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                let c: char = (*self).into();
                c.fmt(f)
            }
        }
    )*};
}
pub(crate) use enum_char_conv;
