use std::str;

// Devuelve el código fuente de una página web como String
pub fn get_source(url: String) -> Result<String, ureq::Error> {
    let source = ureq::get(&url).call()?.into_string()?;
    Ok(source)
}

// Decodifica un String de base64 a un String
pub fn decode_base64(input: String) -> String {
    let bytes = base64::decode(input).unwrap();
    let decoded_string = match str::from_utf8(&bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    decoded_string.to_string()
}


