import type { IEnemyDamageType, IEnemyLevel } from "#/lib/api/enemies";
import type { EnemyListMessageKey } from "./constants";

export interface ILevelToken {
    accent: string;
    accentSoft: string;
    badgeBg: string;
    badgeFg: string;
    badgeBorder: string;
    /** The level as text on the card or its own tint (`--enemy-*-ink`); `accent` is for borders, bars and fills. */
    ink: string;
}

export const LEVEL_TOKENS: Record<IEnemyLevel, ILevelToken> = {
    NORMAL: {
        accent: "var(--muted-foreground)",
        accentSoft: "color-mix(in oklch, var(--muted-foreground) 24%, transparent)",
        badgeBg: "color-mix(in oklch, var(--muted-foreground) 14%, transparent)",
        badgeFg: "var(--muted-foreground)",
        badgeBorder: "color-mix(in oklch, var(--muted-foreground) 36%, transparent)",
        ink: "var(--muted-foreground)",
    },
    ELITE: {
        accent: "var(--enemy-elite)",
        accentSoft: "color-mix(in oklch, var(--enemy-elite) 25%, transparent)",
        badgeBg: "color-mix(in oklch, var(--enemy-elite) 15%, transparent)",
        badgeFg: "var(--enemy-elite-ink)",
        badgeBorder: "color-mix(in oklch, var(--enemy-elite) 45%, transparent)",
        ink: "var(--enemy-elite-ink)",
    },
    BOSS: {
        accent: "var(--primary)",
        accentSoft: "color-mix(in oklch, var(--primary) 30%, transparent)",
        badgeBg: "color-mix(in oklch, var(--primary) 14%, transparent)",
        badgeFg: "var(--enemy-boss-ink)",
        badgeBorder: "color-mix(in oklch, var(--primary) 45%, transparent)",
        ink: "var(--enemy-boss-ink)",
    },
};

export interface IDamageToken {
    color: string;
    /** The damage type as text (`--dmg-*-ink`); `color` is for dots and fills. */
    ink: string;
    /** Message key for the damage type's name, shared with `DAMAGE_TYPE_LABEL_KEY`. */
    labelKey: EnemyListMessageKey;
}

export const DAMAGE_TOKENS: Record<IEnemyDamageType, IDamageToken> = {
    PHYSIC: { color: "var(--dmg-physic)", ink: "var(--dmg-physic-ink)", labelKey: "damageType.PHYSIC" },
    MAGIC: { color: "var(--dmg-magic)", ink: "var(--dmg-magic-ink)", labelKey: "damageType.MAGIC" },
    HEAL: { color: "var(--dmg-heal)", ink: "var(--dmg-heal-ink)", labelKey: "damageType.HEAL" },
    NO_DAMAGE: { color: "var(--dmg-none)", ink: "var(--dmg-none-ink)", labelKey: "damageType.NO_DAMAGE" },
};
