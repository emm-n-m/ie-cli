use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RawDecoded<T> {
    pub raw: T,
    pub decoded: Option<String>,
}

/// Describe a signature mismatch in terms of what was expected and what the file
/// actually starts with.
///
/// A bare "missing X signature" cannot distinguish the two cases a user hits in
/// practice: asking for the wrong resource type, and a shipped file whose
/// signature is corrupt. IWDEE's `#BONECIR.SPL`, for instance, starts with
/// `SPL\x03` — one byte off, with an otherwise coherent spell behind it.
pub(crate) fn signature_mismatch(label: &str, expected: &[u8], found: &[u8]) -> String {
    format!(
        "missing {label} signature: expected {}, found {}",
        printable_signature(expected),
        printable_signature(found)
    )
}

fn printable_signature(bytes: &[u8]) -> String {
    let rendered = bytes
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() || *byte == b' ' {
                (*byte as char).to_string()
            } else {
                format!("\\x{byte:02X}")
            }
        })
        .collect::<String>();
    format!("\"{rendered}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_mismatch_renders_non_printable_bytes_as_escapes() {
        let message = signature_mismatch("SPL", b"SPL ", &[b'S', b'P', b'L', 0x03]);
        assert_eq!(
            message,
            "missing SPL signature: expected \"SPL \", found \"SPL\\x03\""
        );
    }

    #[test]
    fn signature_mismatch_keeps_printable_signatures_readable() {
        let message = signature_mismatch("AREA", b"AREA", b"ITM ");
        assert_eq!(
            message,
            "missing AREA signature: expected \"AREA\", found \"ITM \""
        );
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RawDecodedFlags<T = u32> {
    pub raw: T,
    pub decoded: Vec<String>,
}
