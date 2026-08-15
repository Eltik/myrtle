import { BuffsPanel } from "frontend";

// Team-side buffs for the HPS calculator. A preset chip lights up when ATK%,
// ASPD and target count all match it; the fields themselves are uncontrolled
// NumberFields keyed on `hydrationToken`.
const config = (atk: number, aspd: number, targets: number, extra: { flatAtk?: number; fragile?: number; spBoost?: number } = {}) => ({
    buffs: { atk, aspd, flatAtk: extra.flatAtk ?? 0, fragile: extra.fragile ?? 0 },
    targets,
    spBoost: extra.spBoost ?? 0,
});

const noop = () => {};

export const Solo = () => (
    <div className="max-w-md">
        <BuffsPanel buffs={config(0, 0, 1)} hydrationToken={1} onChangeBuffs={noop} onChangeBuffValues={noop} />
    </div>
);

export const BuffedAoE = () => (
    <div className="max-w-md">
        <BuffsPanel buffs={config(0.4, 0, 3)} hydrationToken={1} onChangeBuffs={noop} onChangeBuffValues={noop} />
    </div>
);

export const AspdPush = () => (
    <div className="max-w-md">
        <BuffsPanel buffs={config(0, 60, 1)} hydrationToken={1} onChangeBuffs={noop} onChangeBuffValues={noop} />
    </div>
);

export const FullTeamSupport = () => (
    <div className="max-w-md">
        <BuffsPanel buffs={config(0.8, 55, 4, { flatAtk: 350, fragile: 0.3, spBoost: 0.55 })} hydrationToken={1} onChangeBuffs={noop} onChangeBuffValues={noop} />
    </div>
);
