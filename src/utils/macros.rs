#[macro_export]
macro_rules! lazy_regex {
    ($regex_str:expr) => {{
        static REGEX: ::once_cell::sync::OnceCell<::regex::Regex> =
            ::once_cell::sync::OnceCell::new();
        REGEX.get_or_init(|| ::regex::Regex::new($regex_str).unwrap())
    }};
}
