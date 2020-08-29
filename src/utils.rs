pub enum QNameCategory<'a> {
    None,
    UnprefixedName(&'a str),
    PrefixedName(&'a str, &'a str),
}

pub fn classify_qname(s: &str) -> QNameCategory {
    todo!()
}

macro_rules! impl_enum_from_variant {
    ($enum_type:ident, $variant_name:ident, $variant_type:path) => {
        impl From<$variant_type> for $enum_type {
            fn from(v: $variant_type) -> $enum_type {
                $enum_type::$variant_name(v)
            }
        }
    };
}
