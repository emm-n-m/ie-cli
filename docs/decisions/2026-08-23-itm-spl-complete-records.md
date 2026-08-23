# Complete ITM and SPL fixed-size records before serialization

Future structured writes need either exhaustive record parsing or explicit preservation of every
unknown byte. The ITM and SPL read paths fell short of that gate in three concrete ways:

- ITM effects stopped after the resource resref and discarded their final 20 bytes.
- SPL omitted documented header and extended-header fields, including values marked unused.
- SPL read the 16-bit extended-header location, required level, and casting time as 8-bit values.

The parsers now expose every field in the 48-byte ITM/SPL effect record and every field in the
40-byte SPL extended header. Documented but unused SPL values remain raw numeric fields; no meaning
is invented for them. Correcting the three truncated SPL reads is an intentional JSON value fix:
ordinary low-byte values serialize as before, while values above 255 no longer lose data.

ITM V1.1 needs a separate 0x9A-byte header boundary. The parser now rejects shorter V1.1 headers,
labels offset 0x58 as `pickup_sound` instead of the V1-only `description_image`, and preserves the
unknown bytes from 0x80 through 0x99. Its unused requirement range is also copied verbatim because
the V1 stat labels do not apply to that variant; those semantic V1 fields serialize as `null` for
V1.1 instead of inventing stat requirements from unused bytes.

These changes do not add a serializer. They remove known read-side holes so a later serializer can
prove byte-exact round trips over complete fixed-size records. Synthetic goldens now contain a real
ITM effect and a SPL extended header plus feature block, ensuring their nested JSON shapes are pinned.

Layout references:

- [IESDP ITM V1](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/itm_v1.htm)
- [IESDP ITM V1.1](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/itm_v1.1.htm)
- [IESDP SPL V1](https://gibberlings3.github.io/iesdp/file_formats/ie_formats/spl_v1.htm)
