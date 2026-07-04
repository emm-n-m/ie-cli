# Save Item Write Complete Gate

Status: accepted as the first step toward BG/IWD support.

`save-add-item` now selects save-write behavior through a `GamLayout` descriptor rather
than a single global offset table. Only the PSTEE descriptor is enabled for writes. BG/BG2
and IWDEE are hard-gated until their GAM section offsets and inventory slot maps are
validated against Near Infinity/IESDP and real saves.

The gate applies to the whole `add_item_to_save_gam` path, including explicit `--slot`
selection. This closes the footgun where a non-PST save could bypass the PST-only
auto-slot check and still use a PST-tuned GAM offset fix-up.

The PSTEE descriptor uses the GAM V2 offset fields currently validated for this project:
`0x20`, `0x28`, `0x30`, `0x38`, `0x48`, `0x50`, `0x68`, `0x6C`, and `0x78`. Offset fix-up
skips `0` and `0xFFFFFFFF` sentinels before applying the `>= insertion` shift rule.

`GameVariant` now distinguishes `Iwd` from the BG-family `Standard` variant so future
layout lifting can happen per family without overloading "standard" for every non-PST
game.
