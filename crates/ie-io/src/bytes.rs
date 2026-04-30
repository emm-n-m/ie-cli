use crate::IoError;

pub(crate) fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16, IoError> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| IoError::UnexpectedEof("u16".to_string()))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

pub(crate) fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32, IoError> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| IoError::UnexpectedEof("u32".to_string()))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}
