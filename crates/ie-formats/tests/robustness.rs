//! Deterministic adversarial-input coverage for every decoded resource family.
//!
//! This is intentionally dependency-free so it runs in the normal CI suite.
//! It does not replace a coverage-guided fuzzer, but it makes the primary
//! parser invariant permanent: malformed bytes may return an error, never
//! panic or abort the process.

use ie_core::{
    GameVariant, ResolverBundle, ResourceBytes, ResourceMetadata, ResourceType, SourceKind,
};
use ie_formats::decode_to_json;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

const CASES_PER_TYPE: usize = 256;
const MAX_INPUT_LENGTH: usize = 4096;

#[test]
fn decoders_do_not_panic_on_deterministic_adversarial_inputs() {
    let formats = [
        (ResourceType::Itm, "FUZZ.ITM", Some((*b"ITM ", *b"V1  "))),
        (ResourceType::Spl, "FUZZ.SPL", Some((*b"SPL ", *b"V1  "))),
        (ResourceType::Cre, "FUZZ.CRE", Some((*b"CRE ", *b"V1.0"))),
        (ResourceType::Chr, "FUZZ.CHR", Some((*b"CHR ", *b"V2.0"))),
        (ResourceType::Sto, "FUZZ.STO", Some((*b"STOR", *b"V1.0"))),
        (ResourceType::Dlg, "FUZZ.DLG", Some((*b"DLG ", *b"V1.0"))),
        (ResourceType::Are, "FUZZ.ARE", Some((*b"AREA", *b"V1.0"))),
        (ResourceType::Bcs, "FUZZ.BCS", None),
    ];

    let mut random = XorShift64(0x6A09_E667_F3BC_C909);
    for (resource_type, resource_name, header) in formats {
        for case_index in 0..CASES_PER_TYPE {
            let length = (random.next() as usize) % (MAX_INPUT_LENGTH + 1);
            let mut bytes = (0..length).map(|_| random.next() as u8).collect::<Vec<_>>();

            // Most completely random binary inputs stop at the signature. Give
            // sufficiently long cases a real signature/version so randomized
            // offsets, counts, tables, and nested structures are exercised.
            if let Some((signature, version)) = header.filter(|_| bytes.len() >= 8) {
                bytes[0..4].copy_from_slice(&signature);
                bytes[4..8].copy_from_slice(&version);
            }

            let resource = ResourceBytes {
                metadata: ResourceMetadata {
                    source_path: PathBuf::from("<adversarial>"),
                    source_kind: SourceKind::LooseFile,
                    resource_type,
                    resource_name: resource_name.to_string(),
                    game_variant: GameVariant::Standard,
                },
                bytes,
            };

            let outcome = catch_unwind(AssertUnwindSafe(|| {
                decode_to_json(
                    &resource,
                    ResolverBundle {
                        strref: None,
                        ids: None,
                        links: None,
                    },
                )
            }));

            assert!(
                outcome.is_ok(),
                "{} decoder panicked on deterministic adversarial case {case_index} ({} bytes)",
                resource_type.as_str(),
                resource.bytes.len()
            );
        }
    }
}

struct XorShift64(u64);

impl XorShift64 {
    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }
}
