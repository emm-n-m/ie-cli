use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RawDecoded<T> {
    pub raw: T,
    pub decoded: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RawDecodedFlags<T = u32> {
    pub raw: T,
    pub decoded: Vec<String>,
}
