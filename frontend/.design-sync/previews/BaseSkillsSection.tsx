import { BaseSkillsSection } from "frontend";

// The collapsible RIIC/base-skill ladder on the Information tab, fed by
// `operator.baseSkills` from `/api/operators/<id>` (see InfoContent.tsx). It
// sorts by unlock order, so the Elite badge on each icon steps E0 -> E1 -> E2.
// Payloads below are the live ones for Młynar, Texas and Lancet-2.
const MLYNAR_BASE_SKILLS = [
    {
        "buffId": "control_mp_cost[008]",
        "buffName": "Self-Absorbed",
        "description": "When this Operator is assigned to the Control Center, increases Morale of all Operators in the Control Center by <@cc.vup>+0.05</> per hour.",
        "roomType": "CONTROL",
        "targets": [],
        "skillIcon": "bskill_ctrl_cost",
        "unlockElite": 0,
        "unlockLevel": 1
    },
    {
        "buffId": "control_mp_lonely[000]",
        "buffName": "Business Is Business",
        "description": "When this Operator is assigned to the Control Center, working Operators in <$cc.c.room1><@cc.kw>certain facilities</></> will recover <@cc.vup>+0.1</> Morale per hour, and <$cc.c.skill><@cc.kw>some skills</></> in the Control Center will provide additional Morale recovery for Operators working in <$cc.c.room2><@cc.kw>other facilities</></>",
        "roomType": "CONTROL",
        "targets": [],
        "skillIcon": "bskill_ctrl_lonely",
        "unlockElite": 2,
        "unlockLevel": 1
    }
];

const TEXAS_BASE_SKILLS = [
    {
        "buffId": "trade_ord_spd&cost_P[000]",
        "buffName": "Feud",
        "description": "When this Operator is assigned to the same Trading Post as <@cc.kw>Lappland</>, Morale consumed each hour <@cc.vdown>+0.3</>, and order acquisition efficiency <@cc.vup>+65%</>",
        "roomType": "TRADING",
        "targets": [],
        "skillIcon": "bskill_tra_texas1",
        "unlockElite": 0,
        "unlockLevel": 1
    },
    {
        "buffId": "trade_ord_limit&cost_P[010]",
        "buffName": "Tacit Understanding",
        "description": "When this Operator is assigned to the same Trading Post as <@cc.kw>Exusiai</>, Morale consumed each hour <@cc.vup>-0.3</>",
        "roomType": "TRADING",
        "targets": [],
        "skillIcon": "bskill_tra_texas2",
        "unlockElite": 2,
        "unlockLevel": 1
    }
];

const LANCET_BASE_SKILLS = [
    {
        "buffId": "power_rec_spd[000]",
        "buffName": "Backup Energy",
        "description": "When this Operator is assigned to a Power Plant, increases the drone recovery rate by <@cc.vup>+10%</>",
        "roomType": "POWER",
        "targets": [],
        "skillIcon": "bskill_pow_spd1",
        "unlockElite": 0,
        "unlockLevel": 1
    },
    {
        "buffId": "dorm_rec_single[000]",
        "buffName": "Medical Service",
        "description": "When this Operator is assigned to a Dormitory, restores <@cc.vup>+0.65</> Morale per hour to another Operator assigned to that Dormitory whose Morale is not full (Only the strongest effect of this type takes place)",
        "roomType": "DORMITORY",
        "targets": [],
        "skillIcon": "bskill_dorm_single2",
        "unlockElite": 0,
        "unlockLevel": 30
    }
];

export const SixStarLadder = () => <BaseSkillsSection server="en" skills={MLYNAR_BASE_SKILLS} />;

export const TradingPostPair = () => <BaseSkillsSection server="en" skills={TEXAS_BASE_SKILLS} />;

export const RobotSupport = () => <BaseSkillsSection server="en" skills={LANCET_BASE_SKILLS} />;
