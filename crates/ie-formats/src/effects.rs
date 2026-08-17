use ie_core::GameVariant;

pub(crate) fn decode_effect_opcode(value: u16, variant: GameVariant) -> Option<&'static str> {
    match variant {
        GameVariant::Standard | GameVariant::Iwd => decode_standard_effect_opcode(value),
        GameVariant::Pst => decode_pst_effect_opcode(value),
    }
}

fn decode_standard_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("AC vs. Damage Type Modifier"),
        1 => Some("Attacks Per Round Modifier"),
        2 => Some("Cure Sleep"),
        3 => Some("Berserk"),
        4 => Some("Cure Berserk"),
        5 => Some("Charm Specific Creature"),
        6 => Some("Charisma"),
        10 => Some("Constitution"),
        12 => Some("Damage"),
        15 => Some("Dexterity"),
        18 => Some("Max HP"),
        19 => Some("Intelligence"),
        33 => Some("Save vs. Death Modifier"),
        34 => Some("Save vs. Wands Modifier"),
        35 => Some("Save vs. Polymorph Modifier"),
        36 => Some("Save vs. Breath Modifier"),
        37 => Some("Save vs. Spell Modifier"),
        38 => Some("Silence"),
        44 => Some("Strength"),
        45 => Some("Stun"),
        46 => Some("Cure Stun"),
        47 => Some("Cure Invisibility"),
        48 => Some("Cure Silence"),
        49 => Some("Wisdom Modifier"),
        55 => Some("Slay"),
        61 => Some("Creature RGB Color Fade"),
        70 => Some("Projectile"),
        101 => Some("Immunity to effect"),
        128 => Some("Confusion"),
        139 => Some("Display String"),
        141 => Some("Lighting Effects"),
        142 => Some("Display Special Effect Icon"),
        146 => Some("Cast Spell at Creature"),
        147 => Some("Learn Spell"),
        174 => Some("Play Sound Effect"),
        206 => Some("Protection from Spell"),
        215 => Some("Play 3D Effect"),
        318 => Some("Protection from Resource"),
        _ => None,
    }
}

pub(crate) fn decode_effect_target_type(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("None"),
        1 => Some("Self"),
        2 => Some("Projectile Target"),
        3 => Some("Party"),
        4 => Some("Everyone"),
        5 => Some("Everyone Except Party"),
        6 => Some("Caster Group"),
        7 => Some("Target Group"),
        8 => Some("Everyone Except Self"),
        9 => Some("Original Caster"),
        _ => None,
    }
}

pub(crate) fn decode_effect_timing(value: u8) -> Option<&'static str> {
    match value {
        0 => Some("Instant/Limited"),
        1 => Some("Instant/Permanent"),
        2 => Some("Instant/While Equipped"),
        3 => Some("Delay/Limited"),
        4 => Some("Delay/Permanent"),
        5 => Some("Delay/While Equipped"),
        6 => Some("Limited After Duration"),
        7 => Some("Permanent After Duration"),
        8 => Some("Equipped After Duration"),
        9 => Some("Instant/Permanent (After Death)"),
        10 => Some("Instant/Limited (Ticks)"),
        _ => None,
    }
}

fn decode_pst_effect_opcode(value: u16) -> Option<&'static str> {
    // Partial PSTEE table, anchored to the dedicated IESDP PSTEE opcode list:
    // https://gibberlings3.github.io/iesdp/opcodes/pstee.htm
    //
    // Important: do not fall back to the standard BG/EE labels for unknown PST
    // values. A wrong-but-plausible label is worse than `null`.
    match value {
        0 => Some("AC"),
        6 => Some("Charisma"),
        10 => Some("Constitution"),
        15 => Some("Dexterity"),
        18 => Some("Max HP"),
        19 => Some("Intelligence"),
        44 => Some("Strength"),
        49 => Some("Wisdom"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_effect_opcode, decode_effect_target_type, decode_effect_timing};
    use ie_core::GameVariant;

    #[test]
    fn standard_spell_opcode_labels_remain_unchanged() {
        assert_eq!(
            decode_effect_opcode(101, GameVariant::Standard),
            Some("Immunity to effect")
        );
        assert_eq!(
            decode_effect_opcode(318, GameVariant::Standard),
            Some("Protection from Resource")
        );
    }

    #[test]
    fn standard_opcode_labels_do_not_depend_on_container_family() {
        assert_eq!(
            decode_effect_opcode(3, GameVariant::Standard),
            Some("Berserk")
        );
        assert_eq!(
            decode_effect_opcode(12, GameVariant::Standard),
            Some("Damage")
        );
    }

    #[test]
    fn standard_stat_modifier_opcodes_are_decoded() {
        for (opcode, label) in [
            (6, "Charisma"),
            (10, "Constitution"),
            (15, "Dexterity"),
            (18, "Max HP"),
            (19, "Intelligence"),
            (44, "Strength"),
        ] {
            assert_eq!(
                decode_effect_opcode(opcode, GameVariant::Standard),
                Some(label)
            );
            assert_eq!(decode_effect_opcode(opcode, GameVariant::Iwd), Some(label));
        }
    }

    #[test]
    fn standard_save_and_spell_opcodes_are_decoded() {
        for (opcode, label) in [
            (33, "Save vs. Death Modifier"),
            (34, "Save vs. Wands Modifier"),
            (35, "Save vs. Polymorph Modifier"),
            (36, "Save vs. Breath Modifier"),
            (37, "Save vs. Spell Modifier"),
            (146, "Cast Spell at Creature"),
            (147, "Learn Spell"),
            (206, "Protection from Spell"),
        ] {
            assert_eq!(
                decode_effect_opcode(opcode, GameVariant::Standard),
                Some(label)
            );
        }
    }

    #[test]
    fn embedded_effect_target_and_timing_labels_follow_eff_v1() {
        assert_eq!(decode_effect_target_type(0), Some("None"));
        assert_eq!(decode_effect_target_type(1), Some("Self"));
        assert_eq!(decode_effect_timing(2), Some("Instant/While Equipped"));
    }

    #[test]
    fn pst_opcode_labels_use_torment_table() {
        assert_eq!(decode_effect_opcode(0, GameVariant::Pst), Some("AC"));
        assert_eq!(decode_effect_opcode(6, GameVariant::Pst), Some("Charisma"));
        assert_eq!(
            decode_effect_opcode(10, GameVariant::Pst),
            Some("Constitution")
        );
        assert_eq!(
            decode_effect_opcode(15, GameVariant::Pst),
            Some("Dexterity")
        );
        assert_eq!(decode_effect_opcode(18, GameVariant::Pst), Some("Max HP"));
        assert_eq!(
            decode_effect_opcode(19, GameVariant::Pst),
            Some("Intelligence")
        );
        assert_eq!(decode_effect_opcode(44, GameVariant::Pst), Some("Strength"));
        assert_eq!(decode_effect_opcode(49, GameVariant::Pst), Some("Wisdom"));
        assert_eq!(decode_effect_opcode(45, GameVariant::Pst), None);
    }
}
