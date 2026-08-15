import { OverviewSection } from "frontend";

/** `Options` blocks straight out of `GET /level/{stageId}`. */
const OPTIONS_4_10 = {
    characterLimit: 10,
    maxLifePoint: 3,
    initialCost: 10,
    maxCost: 99,
    costIncreaseTime: 1,
    moveMultiplier: 0.5,
    steeringEnabled: true,
    isTrainingLevel: false,
    isPredefinedCardsSelectable: false,
    maxPlayTime: -1,
};
const OPTIONS_CE6 = {
    characterLimit: 8,
    maxLifePoint: 3,
    initialCost: 50,
    maxCost: 99,
    costIncreaseTime: 3,
    moveMultiplier: 0.5,
    steeringEnabled: true,
    isTrainingLevel: false,
    isPredefinedCardsSelectable: false,
    maxPlayTime: -1,
};

/** 4-10 — Extinguished Flames: ten slots, DP every second, enemies at half speed. */
export const BossStage = () => (
    <div className="max-w-4xl">
        <OverviewSection level={{ options: OPTIONS_4_10 }} />
    </div>
);

/** CE-6 — Experimental Material Escort: 50 starting DP, but a tick every three seconds. */
export const ResourceRun = () => (
    <div className="max-w-4xl">
        <OverviewSection level={{ options: OPTIONS_CE6 }} />
    </div>
);

/** At the default ×1 move multiplier the Move Speed card is dropped entirely. */
export const NormalMoveSpeed = () => (
    <div className="max-w-4xl">
        <OverviewSection level={{ options: { ...OPTIONS_4_10, moveMultiplier: 1, characterLimit: 6, initialCost: 25 } }} />
    </div>
);
