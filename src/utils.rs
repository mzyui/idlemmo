use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STD;

pub fn generate_obfuscated_data(key: Option<&str>) -> String {
    let key = key.unwrap_or("fair-maiden");
    let text = fastrand::u64(400..600).to_string();
    let key_codes: Vec<u32> = key.chars().map(|c| c as u32).collect();
    let mut encrypted: Vec<u8> = Vec::with_capacity(text.len());

    for (i, ch) in text.chars().enumerate() {
        let t = ch as u32;
        let k = key_codes[i % key_codes.len()];
        let xor_val = (t ^ k) & 0xFF;
        encrypted.push(xor_val as u8);
    }

    BASE64_STD.encode(encrypted)
}

pub fn obfuscate_email(email: &str) -> String {
    let mut parts = email.split('@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    format!("{}...@{}", &local[..local.len().min(3)], domain)
}
