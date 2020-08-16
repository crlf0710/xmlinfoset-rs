pub enum QNameCategory<'a> {
    None,
    UnprefixedName(&'a str),
    PrefixedName(&'a str, &'a str),
}

pub fn classify_qname(s: &str) -> QNameCategory {
    todo!()
}
