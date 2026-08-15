import { EnemyPanel } from "frontend";

// Enemy context for the DPS calculator. The NumberFields are uncontrolled and
// remount on `hydrationToken`, so each story just hands over a settled config.
// Shred auto-expands when any shred value is non-zero.
const enemy = (defense: number, res: number, targets: number, spBoost: number, shred: Record<string, number> = {}) => ({ defense, res, targets, spBoost, shred });

const noop = () => {};

export const ElitePreset = () => (
    <div className="max-w-md">
        <EnemyPanel enemy={enemy(1000, 0, 1, 0)} hydrationToken={1} onChangeEnemy={noop} onChangeShred={noop} />
    </div>
);

export const BossWithShred = () => (
    <div className="max-w-md">
        <EnemyPanel enemy={enemy(2500, 30, 1, 0.5, { def: 20, defFlat: 300, res: 10, resFlat: 0 })} hydrationToken={1} onChangeEnemy={noop} onChangeShred={noop} />
    </div>
);

export const DroneWave = () => (
    <div className="max-w-md">
        <EnemyPanel enemy={enemy(200, 0, 3, 0)} hydrationToken={1} onChangeEnemy={noop} onChangeShred={noop} />
    </div>
);

export const CustomEnemy = () => (
    <div className="max-w-md">
        <EnemyPanel enemy={enemy(1400, 45, 2, 1.2)} hydrationToken={1} onChangeEnemy={noop} onChangeShred={noop} />
    </div>
);
