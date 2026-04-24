# ARE Read And Link Enrichment

M6 adds read-only `ARE` support for area inspection, starting with the actor-placement workflow needed for PSTEE `AR0202.ARE`.

The first slice parses only the `AREA`/`V1.0` header and actor table. Actor output includes placement coordinates, dialog, script slots, CRE references, embedded CRE offsets/sizes, and preserved unknown actor tail bytes. Unknown byte ranges inside the actor record are preserved verbatim rather than dropped, so future round-trip write support and diagnostics against unknown PST-specific fields do not need a reparse.

Regions, doors, spawn points, containers, WED/TIS data, and write support remain out of scope until a concrete use case needs them. Their offsets and counts are still surfaced in a `deferred_sections` JSON block — not as decoded records — so the output advertises what was skipped and gives future decoders a sanity-check anchor without having to re-read the header.

`V1.0` covers BG1/BG2/IWD and all Enhanced Editions (including PSTEE). Legacy Planescape: Torment ships an incompatible `V1.1` layout and is explicitly rejected at parse time; adding it would be a separate decision.

Linked resources are exposed as enrichment metadata, not recursive decoded payloads. ARE parsing stays byte-focused; the CLI/IO layer resolves actor CRE, DLG, and BCS links through the existing locator. CRE links may include display-name strrefs resolved through TLK. Missing linked resources are represented with `exists: false` and do not fail the ARE dump.

