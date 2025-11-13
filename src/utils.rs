use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STD;
use once_cell::sync::OnceCell;
use regex::Regex;

pub const API_VERSION: &str = "1.0.0.1";

#[macro_export]
macro_rules! lazy_regex {
    ($regex_str:expr) => {{
        static REGEX: ::once_cell::sync::OnceCell<::regex::Regex> = ::once_cell::sync::OnceCell::new();
        REGEX.get_or_init(|| ::regex::Regex::new($regex_str).unwrap())
    }};
}

pub fn generate_obfuscated_data(encryption_key_option: Option<&str>) -> String {
    let encryption_key = encryption_key_option.unwrap_or("fair-maiden");
    let random_text = fastrand::u64(400..600).to_string();
    let key_char_codes: Vec<u32> = encryption_key.chars().map(|c| c as u32).collect();
    let mut encrypted_bytes: Vec<u8> = Vec::with_capacity(random_text.len());

    for (i, text_char) in random_text.chars().enumerate() {
        let text_char_code = text_char as u32;
        let key_char_code = key_char_codes[i % key_char_codes.len()];
        let xor_result = (text_char_code ^ key_char_code) & 0xFF;
        encrypted_bytes.push(xor_result as u8);
    }

    BASE64_STD.encode(encrypted_bytes)
}

pub fn obfuscate_email(email: &str) -> String {
    let mut email_parts = email.split('@');
    let local_part = email_parts.next().unwrap_or("");
    let domain_part = email_parts.next().unwrap_or("");
    format!(
        "{}{}.@{}",
        &local_part[..local_part.len().min(3)],
        "*".repeat(local_part.len() - 3),
        domain_part
    )
}