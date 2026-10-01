import type { TierEntityKind } from "#/lib/api/tier-entities";

export interface IOperator {
    id: string;
    name: string;
    rarity: number;
    role: string;
    arch: string;
    /** Set for a non-operator placement (an enemy, an event...); absent for an operator. */
    kind?: Exclude<TierEntityKind, "operator">;
    /** The non-operator's resolved icon path, `null` when the extract has no art. */
    icon?: string | null;
}

export interface ITierEntry {
    name: string;
    operators: IOperator[];
    color?: string | null;
}

export interface ITierList {
    id: string;
    slug: string;
    title: string;
    tag: string;
    stage: string;
    author: { name: string; avatarId: string | null };
    updated: string;
    votes: number;
    views: number;
    comments: number;
    hot?: boolean;
    accent: string;
    tiers: ITierEntry[];
}
