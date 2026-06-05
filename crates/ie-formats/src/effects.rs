use ie_core::GameVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectOpcodeFamily {
    Item,
    Spell,
    Creature,
}

pub(crate) fn decode_effect_opcode(
    value: u16,
    family: EffectOpcodeFamily,
    variant: GameVariant,
) -> Option<&'static str> {
    match variant {
        GameVariant::Standard => match family {
            EffectOpcodeFamily::Item => decode_item_standard_effect_opcode(value),
            EffectOpcodeFamily::Spell => decode_spell_standard_effect_opcode(value),
            EffectOpcodeFamily::Creature => decode_creature_standard_effect_opcode(value),
        },
        GameVariant::Pst => decode_pst_effect_opcode(value),
    }
}

fn decode_item_standard_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Cure Condition"),
        1 => Some("Cure Poison"),
        2 => Some("Cure Disease"),
        3 => Some("Heal"),
        4 => Some("Drain Level"),
        5 => Some("Drain HP"),
        6 => Some("Drain Magic"),
        7 => Some("Drain Item Charges"),
        8 => Some("Cure Fatigue"),
        9 => Some("Cure Intoxication"),
        16 => Some("Poison"),
        17 => Some("Disease"),
        18 => Some("Fatigue"),
        19 => Some("Intoxication"),
        20 => Some("Sleep"),
        21 => Some("Confusion"),
        22 => Some("Charm"),
        23 => Some("Fear"),
        24 => Some("Power Word: Stun"),
        25 => Some("Hold Monster"),
        26 => Some("Paralyze"),
        27 => Some("Haste"),
        28 => Some("Slow"),
        29 => Some("Protection: Fire"),
        30 => Some("Protection: Cold"),
        31 => Some("Protection: Lightning"),
        32 => Some("Protection: Acid"),
        33 => Some("Protection: Magic"),
        34 => Some("Protection: Magic Fire"),
        35 => Some("Protection: Magic Cold"),
        36 => Some("Invisibility 10' Radius"),
        37 => Some("Invisibility"),
        38 => Some("Detect Invisible"),
        39 => Some("Invisibility Purge"),
        40 => Some("Protection: Cold (Lesser)"),
        41 => Some("Protection: Fire (Lesser)"),
        42 => Some("Protection: Lightning (Lesser)"),
        43 => Some("Protection: Magic Damage (Lesser)"),
        44 => Some("Protection: Acid (Lesser)"),
        45 => Some("Strength"),
        46 => Some("Dexterity"),
        47 => Some("Constitution"),
        48 => Some("Intelligence"),
        49 => Some("Wisdom"),
        50 => Some("Charisma"),
        51 => Some("Damage"),
        52 => Some("Thac0 Bonus"),
        53 => Some("Saving Throw: Death"),
        54 => Some("Saving Throw: Wands"),
        55 => Some("Saving Throw: Polymorph"),
        56 => Some("Saving Throw: Breath"),
        57 => Some("Saving Throw: Spells"),
        _ => None,
    }
}

fn decode_spell_standard_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Cure Condition"),
        1 => Some("Cure Poison"),
        2 => Some("Cure Disease"),
        3 => Some("Berserk"),
        4 => Some("Drain Level"),
        5 => Some("Drain HP"),
        12 => Some("Damage"),
        38 => Some("Silence"),
        55 => Some("Slay"),
        61 => Some("Creature RGB Color Fade"),
        70 => Some("Projectile"),
        101 => Some("Immunity to effect"),
        128 => Some("Confusion"),
        139 => Some("Display String"),
        141 => Some("Lighting Effects"),
        142 => Some("Display Special Effect Icon"),
        174 => Some("Play Sound Effect"),
        215 => Some("Play 3D Effect"),
        318 => Some("Protection from Resource"),
        _ => None,
    }
}

fn decode_creature_standard_effect_opcode(value: u16) -> Option<&'static str> {
    match value {
        0 => Some("Cure Condition"),
        1 => Some("Cure Poison"),
        2 => Some("Cure Disease"),
        3 => Some("Berserk"),
        12 => Some("Damage"),
        16 => Some("Poison"),
        25 => Some("Hold Monster"),
        38 => Some("Silence"),
        45 => Some("Strength"),
        46 => Some("Dexterity"),
        47 => Some("Constitution"),
        48 => Some("Intelligence"),
        49 => Some("Wisdom"),
        50 => Some("Charisma"),
        55 => Some("Slay"),
        128 => Some("Confusion"),
        139 => Some("Display String"),
        142 => Some("Display Special Effect Icon"),
        174 => Some("Play Sound Effect"),
        215 => Some("Play 3D Effect"),
        318 => Some("Protection from Resource"),
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
    use super::{EffectOpcodeFamily, decode_effect_opcode};
    use ie_core::GameVariant;

    #[test]
    fn standard_spell_opcode_labels_remain_unchanged() {
        assert_eq!(
            decode_effect_opcode(101, EffectOpcodeFamily::Spell, GameVariant::Standard),
            Some("Immunity to effect")
        );
        assert_eq!(
            decode_effect_opcode(318, EffectOpcodeFamily::Spell, GameVariant::Standard),
            Some("Protection from Resource")
        );
    }

    #[test]
    fn standard_item_opcode_labels_remain_unchanged() {
        assert_eq!(
            decode_effect_opcode(45, EffectOpcodeFamily::Item, GameVariant::Standard),
            Some("Strength")
        );
        assert_eq!(
            decode_effect_opcode(3, EffectOpcodeFamily::Item, GameVariant::Standard),
            Some("Heal")
        );
    }

    #[test]
    fn pst_opcode_labels_use_torment_table() {
        assert_eq!(
            decode_effect_opcode(0, EffectOpcodeFamily::Item, GameVariant::Pst),
            Some("AC")
        );
        assert_eq!(
            decode_effect_opcode(6, EffectOpcodeFamily::Item, GameVariant::Pst),
            Some("Charisma")
        );
        assert_eq!(
            decode_effect_opcode(10, EffectOpcodeFamily::Spell, GameVariant::Pst),
            Some("Constitution")
        );
        assert_eq!(
            decode_effect_opcode(15, EffectOpcodeFamily::Creature, GameVariant::Pst),
            Some("Dexterity")
        );
        assert_eq!(
            decode_effect_opcode(18, EffectOpcodeFamily::Item, GameVariant::Pst),
            Some("Max HP")
        );
        assert_eq!(
            decode_effect_opcode(19, EffectOpcodeFamily::Spell, GameVariant::Pst),
            Some("Intelligence")
        );
        assert_eq!(
            decode_effect_opcode(44, EffectOpcodeFamily::Creature, GameVariant::Pst),
            Some("Strength")
        );
        assert_eq!(
            decode_effect_opcode(49, EffectOpcodeFamily::Item, GameVariant::Pst),
            Some("Wisdom")
        );
        assert_eq!(
            decode_effect_opcode(45, EffectOpcodeFamily::Item, GameVariant::Pst),
            None
        );
    }
}
