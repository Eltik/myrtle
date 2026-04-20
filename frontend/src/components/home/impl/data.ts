export interface Operator {
    id: string;
    name: string;
    rarity: number;
    role: string;
    arch: string;
}

export interface TierEntry {
    name: string;
    operators: Operator[];
}

export interface TierList {
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
    tiers: TierEntry[];
}
