import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";

export interface ILevelToken {
    accent: string;
    accentSoft: string;
    badgeBg: string;
    badgeFg: string;
    badgeBorder: string;
}

export const LEVEL_TOKENS: Record<IEnemyLevel, ILevelToken> = {
    NORMAL: {
        accent: "var(--muted-foreground)",
        accentSoft: "color-mix(in oklch, var(--muted-foreground) 24%, transparent)",
        badgeBg: "color-mix(in oklch, var(--muted-foreground) 14%, transparent)",
        badgeFg: "var(--muted-foreground)",
        badgeBorder: "color-mix(in oklch, var(--muted-foreground) 36%, transparent)",
    },
    ELITE: {
        accent: "var(--enemy-elite)",
        accentSoft: "color-mix(in oklch, var(--enemy-elite) 25%, transparent)",
        badgeBg: "color-mix(in oklch, var(--enemy-elite) 15%, transparent)",
        badgeFg: "var(--enemy-elite)",
        badgeBorder: "color-mix(in oklch, var(--enemy-elite) 45%, transparent)",
    },
    BOSS: {
        accent: "var(--primary)",
        accentSoft: "color-mix(in oklch, var(--primary) 30%, transparent)",
        badgeBg: "color-mix(in oklch, var(--primary) 14%, transparent)",
        badgeFg: "var(--primary)",
        badgeBorder: "color-mix(in oklch, var(--primary) 45%, transparent)",
    },
};

export interface IDamageToken {
    color: string;
    label: string;
}

export const DAMAGE_TOKENS: Record<IEnemyDamageType, IDamageToken> = {
    PHYSIC: { color: "var(--dmg-physic)", label: "Physical" },
    MAGIC: { color: "var(--dmg-magic)", label: "Arts" },
    HEAL: { color: "var(--dmg-heal)", label: "Heal" },
    NO_DAMAGE: { color: "var(--dmg-none)", label: "No Damage" },
};
