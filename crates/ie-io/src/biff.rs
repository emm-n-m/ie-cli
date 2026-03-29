use crate::IoError;
use crate::bytes::{read_u16_le, read_u32_le};
use flate2::read::ZlibDecoder;
use std::fs;
use std::io::Read;
use std::path::Path;

const BIFF_SIGNATURE: &[u8; 8] = b"BIFFV1  ";
const BIF_SIGNATURE: &[u8; 8] = b"BIF V1.0";
const BIFC_SIGNATURE: &[u8; 8] = b"BIFCV1.0";

pub(crate) fn read_resource(path: &Path, locator: u32) -> Result<Vec<u8>, IoError> {
    let bytes = fs::read(path).map_err(IoError::FileIo)?;
    let archive = match detect_archive_type(&bytes)? {
        ArchiveType::Biff => bytes,
        ArchiveType::Bif => decompress_bif(&bytes)?,
        ArchiveType::Bifc => decompress_bifc(&bytes)?,
    };

    extract_resource(&archive, locator)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveType {
    Biff,
    Bif,
    Bifc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArchiveEntry {
    offset: usize,
    size: usize,
    tile_count: Option<usize>,
    type_code: u16,
}

impl ArchiveEntry {
    fn byte_len(self) -> Result<usize, IoError> {
        match self.tile_count {
            Some(tile_count) => tile_count
                .checked_mul(self.size)
                .ok_or_else(|| IoError::InvalidBiff("TIS resource size overflow".to_string())),
            None => Ok(self.size),
        }
    }

    fn is_tileset(self) -> bool {
        self.tile_count.is_some()
    }
}

fn detect_archive_type(bytes: &[u8]) -> Result<ArchiveType, IoError> {
    let signature = bytes
        .get(..8)
        .ok_or_else(|| IoError::InvalidBiff("BIFF archive is too small".to_string()))?;

    if signature == BIFF_SIGNATURE {
        Ok(ArchiveType::Biff)
    } else if signature == BIF_SIGNATURE {
        Ok(ArchiveType::Bif)
    } else if signature == BIFC_SIGNATURE {
        Ok(ArchiveType::Bifc)
    } else {
        Err(IoError::InvalidBiff(
            "unsupported BIFF archive signature".to_string(),
        ))
    }
}

fn decompress_bif(bytes: &[u8]) -> Result<Vec<u8>, IoError> {
    let name_length = read_u32_le(bytes, 8)? as usize;
    let size_offset = 12usize
        .checked_add(name_length)
        .ok_or_else(|| IoError::InvalidBiff("BIF header offset overflow".to_string()))?;
    let uncompressed_size = read_u32_le(bytes, size_offset)? as usize;
    let compressed_size = read_u32_le(bytes, size_offset + 4)? as usize;
    let compressed_offset = size_offset
        .checked_add(8)
        .ok_or_else(|| IoError::InvalidBiff("BIF data offset overflow".to_string()))?;
    let compressed_end = compressed_offset
        .checked_add(compressed_size)
        .ok_or_else(|| IoError::InvalidBiff("BIF compressed size overflow".to_string()))?;
    let compressed = bytes
        .get(compressed_offset..compressed_end)
        .ok_or_else(|| {
            IoError::InvalidBiff("BIF compressed payload points outside the file".to_string())
        })?;

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(uncompressed_size);
    decoder
        .read_to_end(&mut decompressed)
        .map_err(IoError::FileIo)?;

    if decompressed.len() != uncompressed_size {
        return Err(IoError::InvalidBiff(format!(
            "BIF decompressed to {} bytes, expected {uncompressed_size}",
            decompressed.len()
        )));
    }

    Ok(decompressed)
}

fn decompress_bifc(bytes: &[u8]) -> Result<Vec<u8>, IoError> {
    let total_uncompressed_size = read_u32_le(bytes, 8)? as usize;
    let mut offset = 12usize;
    let mut decompressed = Vec::with_capacity(total_uncompressed_size);

    while decompressed.len() < total_uncompressed_size {
        let block_uncompressed_size = read_u32_le(bytes, offset)? as usize;
        let block_compressed_size = read_u32_le(bytes, offset + 4)? as usize;
        offset = offset
            .checked_add(8)
            .ok_or_else(|| IoError::InvalidBiff("BIFC block header overflow".to_string()))?;

        let block_end = offset
            .checked_add(block_compressed_size)
            .ok_or_else(|| IoError::InvalidBiff("BIFC block size overflow".to_string()))?;
        let block = bytes.get(offset..block_end).ok_or_else(|| {
            IoError::InvalidBiff("BIFC compressed block points outside the file".to_string())
        })?;

        let start_len = decompressed.len();
        let mut decoder = ZlibDecoder::new(block);
        decoder
            .read_to_end(&mut decompressed)
            .map_err(IoError::FileIo)?;

        if decompressed.len() - start_len != block_uncompressed_size {
            return Err(IoError::InvalidBiff(format!(
                "BIFC block decompressed to {} bytes, expected {block_uncompressed_size}",
                decompressed.len() - start_len
            )));
        }

        offset = block_end;
    }

    if decompressed.len() != total_uncompressed_size {
        return Err(IoError::InvalidBiff(format!(
            "BIFC decompressed to {} bytes, expected {total_uncompressed_size}",
            decompressed.len()
        )));
    }

    Ok(decompressed)
}

fn extract_resource(bytes: &[u8], locator: u32) -> Result<Vec<u8>, IoError> {
    let entry = parse_entry(bytes, locator)?;
    let byte_len = entry.byte_len()?;
    let end = entry
        .offset
        .checked_add(byte_len)
        .ok_or_else(|| IoError::InvalidBiff("resource offset overflow".to_string()))?;
    let resource_bytes = bytes.get(entry.offset..end).ok_or_else(|| {
        IoError::InvalidBiff("resource data points outside the BIFF archive".to_string())
    })?;

    if entry.is_tileset() {
        let tile_count = entry.tile_count.ok_or_else(|| {
            IoError::InvalidBiff("TIS entry is missing tile count metadata".to_string())
        })?;
        let mut output = Vec::with_capacity(24 + resource_bytes.len());
        output.extend_from_slice(b"TIS V1  ");
        output.extend_from_slice(&(tile_count as u32).to_le_bytes());
        output.extend_from_slice(&(entry.size as u32).to_le_bytes());
        output.extend_from_slice(&0x18u32.to_le_bytes());
        output.extend_from_slice(&0x40u32.to_le_bytes());
        output.extend_from_slice(resource_bytes);
        return Ok(output);
    }

    Ok(resource_bytes.to_vec())
}

fn parse_entry(bytes: &[u8], locator: u32) -> Result<ArchiveEntry, IoError> {
    if bytes.get(..8) != Some(BIFF_SIGNATURE) {
        return Err(IoError::InvalidBiff(
            "invalid decompressed BIFF signature".to_string(),
        ));
    }

    let file_count = read_u32_le(bytes, 8)? as usize;
    let tileset_count = read_u32_le(bytes, 12)? as usize;
    let entry_offset = read_u32_le(bytes, 16)? as usize;
    let mut cursor = entry_offset;
    let locator = locator & 0x000F_FFFF;

    for _ in 0..file_count {
        let entry = parse_file_entry(bytes, cursor)?;
        if entry.0 == locator {
            return Ok(entry.1);
        }
        cursor = cursor
            .checked_add(16)
            .ok_or_else(|| IoError::InvalidBiff("file entry cursor overflow".to_string()))?;
    }

    for _ in 0..tileset_count {
        let entry = parse_tileset_entry(bytes, cursor)?;
        if entry.0 == locator {
            return Ok(entry.1);
        }
        cursor = cursor
            .checked_add(20)
            .ok_or_else(|| IoError::InvalidBiff("tileset entry cursor overflow".to_string()))?;
    }

    Err(IoError::ResourceNotFound(format!(
        "BIFF locator 0x{locator:05X}"
    )))
}

fn parse_file_entry(bytes: &[u8], offset: usize) -> Result<(u32, ArchiveEntry), IoError> {
    let locator = read_u32_le(bytes, offset)? & 0x000F_FFFF;
    let data_offset = read_u32_le(bytes, offset + 4)? as usize;
    let size = read_u32_le(bytes, offset + 8)? as usize;
    let type_code = read_u16_le(bytes, offset + 12)?;

    Ok((
        locator,
        ArchiveEntry {
            offset: data_offset,
            size,
            tile_count: None,
            type_code,
        },
    ))
}

fn parse_tileset_entry(bytes: &[u8], offset: usize) -> Result<(u32, ArchiveEntry), IoError> {
    let locator = read_u32_le(bytes, offset)? & 0x000F_FFFF;
    let data_offset = read_u32_le(bytes, offset + 4)? as usize;
    let tile_count = read_u32_le(bytes, offset + 8)? as usize;
    let tile_size = read_u32_le(bytes, offset + 12)? as usize;
    let type_code = read_u16_le(bytes, offset + 16)?;

    Ok((
        locator,
        ArchiveEntry {
            offset: data_offset,
            size: tile_size,
            tile_count: Some(tile_count),
            type_code,
        },
    ))
}
