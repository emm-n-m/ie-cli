# ITM requirement widths and enum corrections

## Decision

Export the ITM requirement fields at offsets `0x28`, `0x2A`, `0x2C`, `0x2E`, and `0x30` as
individual unsigned bytes. Export the interleaved kit-usability bytes at `0x29`, `0x2B`, `0x2D`,
and `0x2F` as `kit_usability`, and offset `0x31` as `weapon_proficiency`.

Use the IESDP ITM V1 tables for item categories and ability type, location, and target values. In
particular, ability type `3` is `Magical`, not `Innate`, and target `5` is `Caster`.

## Reason

IESDP defines these as one-byte values interleaved with independent fields. Reading two bytes per
requirement folded the adjacent field into the value: stock BGEE `SW1H01.ITM` consequently reported
minimum Constitution `22784` (`0x5900`) instead of `0`, where `0x59` is its weapon proficiency.

The real-resource pass also showed stock scroll abilities carrying values that the old ad hoc enum
maps mislabeled or left null. The correction changes erroneous numeric values and labels and adds
two JSON fields. This is an intentional pre-release interface correction: keeping the old values
would preserve a stable but false output.
The exact-value ITM golden and the env-gated stock-resource expectation both pin the new shape and
semantics.
