export interface IOperator {
    id: string;
    name: string;
    rarity: number;
    role: string;
    arch: string;
}

export interface ITierEntry {
    name: string;
    operators: IOperator[];
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
