use myrtle_gamedata_raw::tables::operator::OperatorProfession;

pub const PROFESSION_ORDER: &[OperatorProfession] = &[
    OperatorProfession::Vanguard,
    OperatorProfession::Guard,
    OperatorProfession::Defender,
    OperatorProfession::Sniper,
    OperatorProfession::Caster,
    OperatorProfession::Medic,
    OperatorProfession::Supporter,
    OperatorProfession::Specialist,
];

/// Threshold for when a stat is counted as a threat. Grade's lower bound
pub const THREAT_GRADE_RES: &str = "A";
pub const THREAT_GRADE_DEF: &str = "A";
pub const THREAT_GRADE_MOVE_SPEED: &str = "S";
