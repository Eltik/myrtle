import { BaseSkill } from "frontend";
import type { ReactNode } from "react";

// BaseSkill is one line of an operator's RIIC kit: the skill's sprite, its
// name, and the game's own rich-text description (`<@cc.vup>` / `<@cc.vdown>` /
// `<@cc.kw>` tags render in the game's colours). A skill the operator has not
// promoted far enough to unlock greys out and shows its E/Lv requirement.
// `RoomPopover` stacks these under each seated operator; `BaseSkillsSection`
// on the operator page lists a whole kit. Skills below are the live
// `/api/operators/<id>` kits, ids checked against `/api/operators/index`.

type Skill = {
    buffId: string;
    buffName: string;
    description: string;
    roomType: string;
    efficiency: number;
    targets: string[];
    skillIcon: string;
    unlockElite: number;
    unlockLevel: number;
    slot: number;
    unlocked: boolean;
    live: boolean;
};

const skill = (fields: Omit<Skill, "unlocked" | "live"> & Partial<Pick<Skill, "unlocked" | "live">>): Skill => ({ unlocked: true, live: true, ...fields });

// Jaye - the per-order trading skill every LMD build is priced around.
const STREET_ECONOMICS = skill({
    buffId: "trade_ord_limit_diff[000]",
    buffName: "Street Economics",
    description: "When this Operator is assigned to a Trading Post, increases order acquisition efficiency by <@cc.vup>+4%</> for every difference of <@cc.vup>1</> order(s) between the current number of orders and the maximum number of orders",
    roomType: "TRADING",
    efficiency: 0,
    targets: [],
    skillIcon: "bskill_tra_limit_diff",
    unlockElite: 0,
    unlockLevel: 1,
    slot: 0,
});

const BASIC_NEEDS = skill({
    buffId: "trade_ord_limit_count[000]",
    buffName: "Basic Needs",
    description: "When this Operator is assigned to a Trading Post, reduces the order limit by <@cc.vdown>-1</> for every <@cc.vup>10%</> order acquisition efficiency provided by all other Operators stationed at that Trading Post (to a minimum of 1); Furthermore, increases order acquisition efficiency by <@cc.vup>+4%</> for every <@cc.vup>1</> order(s) (<$cc.t.strong2><@cc.rem>Special Interaction Rules</></> apply with certain skills)",
    roomType: "TRADING",
    efficiency: 0,
    targets: [],
    skillIcon: "bskill_tra_limit_count",
    unlockElite: 1,
    unlockLevel: 1,
    slot: 1,
});

// Shamare - her E2 skill on an operator still at E1.
const WHISPERS_LOCKED = skill({
    buffId: "trade_ord_vodfox[000]",
    buffName: "Whispers",
    description: "When this Operator is assigned to a Trading Post, the order acquisition efficiency contributed by all other Operators assigned to that Trading Post <@cc.vdown>becomes 0</>, but each Operator increases the order acquisition efficiency of this Operator by <@cc.vup>+45%</>, while total Morale consumed is increased by <@cc.vdown>+0.25</> per hour.",
    roomType: "TRADING",
    efficiency: 0,
    targets: [],
    skillIcon: "bskill_tra_vodfox",
    unlockElite: 2,
    unlockLevel: 1,
    slot: 1,
    unlocked: false,
    live: false,
});

// Ptilopsis - two tiers of one slot; the promotion replaced α with β.
const RHINE_ALPHA = skill({
    buffId: "manu_prod_spd[001]",
    buffName: "Rhine Tech α",
    description: "When this Operator is assigned to a Factory, productivity <@cc.vup>+15%</>",
    roomType: "MANUFACTURE",
    efficiency: 15,
    targets: ["F_GOLD", "F_EXP", "F_DIAMOND"],
    skillIcon: "bskill_man_spd1",
    unlockElite: 0,
    unlockLevel: 1,
    slot: 0,
    live: false,
});

const RHINE_BETA = skill({
    buffId: "manu_prod_spd[011]",
    buffName: "Rhine Tech β",
    description: "When this Operator is assigned to a Factory, productivity <@cc.vup>+25%</>",
    roomType: "MANUFACTURE",
    efficiency: 25,
    targets: ["F_GOLD", "F_EXP", "F_DIAMOND"],
    skillIcon: "bskill_man_spd2",
    unlockElite: 2,
    unlockLevel: 1,
    slot: 0,
});

// Texas - a paired skill naming another operator with the keyword colour.
const FEUD = skill({
    buffId: "trade_ord_spd&cost_P[000]",
    buffName: "Feud",
    description: "When this Operator is assigned to the same Trading Post as <@cc.kw>Lappland</>, Morale consumed each hour <@cc.vdown>+0.3</>, and order acquisition efficiency <@cc.vup>+65%</>",
    roomType: "TRADING",
    efficiency: 0,
    targets: [],
    skillIcon: "bskill_tra_texas1",
    unlockElite: 0,
    unlockLevel: 1,
    slot: 0,
});

const TACIT_UNDERSTANDING = skill({
    buffId: "trade_ord_limit&cost_P[010]",
    buffName: "Tacit Understanding",
    description: "When this Operator is assigned to the same Trading Post as <@cc.kw>Exusiai</>, Morale consumed each hour <@cc.vup>-0.3</>",
    roomType: "TRADING",
    efficiency: 0,
    targets: [],
    skillIcon: "bskill_tra_texas2",
    unlockElite: 2,
    unlockLevel: 1,
    slot: 1,
    unlocked: false,
    live: false,
});

// Amiya - a Control Center skill (no room-specific icon colour, same layout).
const AGREEMENT = skill({
    buffId: "control_tra_spd[000]",
    buffName: "Agreement",
    description: "When this Operator is assigned to the Control Center, all Trading Posts' order efficiency <@cc.vup>+7%</> (only the most effective one will take effect when assigned Operators have the same skill effect)",
    roomType: "CONTROL",
    efficiency: 0,
    targets: [],
    skillIcon: "bskill_ctrl_t_spd",
    unlockElite: 0,
    unlockLevel: 1,
    slot: 0,
});

/** The card width these lines live in: the 19rem room popover. */
const Column = ({ children }: { children: ReactNode }) => <div className="flex w-76 flex-col gap-2.5 rounded-lg border border-border bg-popover p-3">{children}</div>;

/** A kit as `RoomPopover` lists it under a seated operator. */
const Kit = ({ name, skills }: { name: string; skills: Skill[] }) => (
    <div className="flex flex-col gap-1">
        <span className="text-[12px] text-foreground">{name}</span>
        <div className="ml-3 flex flex-col gap-1.5 border-border border-l pl-2.5">
            {skills.map((s) => (
                <BaseSkill key={s.buffId} skill={s} />
            ))}
        </div>
    </div>
);

/** One unlocked skill line. */
export const Unlocked = () => (
    <Column>
        <BaseSkill skill={STREET_ECONOMICS} />
    </Column>
);

/** A skill the operator has not promoted far enough for: greyed, with its E/Lv requirement. */
export const LockedByPromotion = () => (
    <Column>
        <BaseSkill skill={WHISPERS_LOCKED} />
    </Column>
);

/** Rich-text markup: gains, losses and keywords in the game's own colours. */
export const MarkupColours = () => (
    <Column>
        <BaseSkill skill={BASIC_NEEDS} />
        <BaseSkill skill={FEUD} />
        <BaseSkill skill={AGREEMENT} />
    </Column>
);

/** Whole kits stacked under their operators, as the room popover lays them out. */
export const OperatorKits = () => (
    <Column>
        <Kit name="Jaye" skills={[STREET_ECONOMICS, BASIC_NEEDS]} />
        <Kit name="Ptilopsis" skills={[RHINE_ALPHA, RHINE_BETA]} />
        <Kit name="Texas" skills={[FEUD, TACIT_UNDERSTANDING]} />
    </Column>
);
