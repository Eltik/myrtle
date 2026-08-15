import { LeaderboardTable } from "frontend";
import type { ReactNode } from "react";

interface ISeed {
    uid: string;
    nickname: string;
    server: string;
    level: number;
    avatar_id: string;
    grade: string;
    rank_global: number;
    rank_delta: number | null;
    total_score: number;
    operator_score: number;
    isSelf?: boolean;
}

const SEEDS: ISeed[] = [
    { uid: "10000817", nickname: "Kyostinv", server: "EN", level: 120, avatar_id: "char_4064_mlynar", grade: "SS+", rank_global: 1, rank_delta: 0, total_score: 0.984, operator_score: 0.971, isSelf: false },
    { uid: "21748302", nickname: "Doktah", server: "JP", level: 120, avatar_id: "char_263_skadi", grade: "SS", rank_global: 2, rank_delta: 3, total_score: 0.961, operator_score: 0.938 },
    { uid: "30914477", nickname: "Ceylonade", server: "CN", level: 118, avatar_id: "char_180_amgoat", grade: "S", rank_global: 3, rank_delta: -1, total_score: 0.917, operator_score: 0.902 },
    { uid: "18220561", nickname: "Eltik", server: "EN", level: 114, avatar_id: "char_102_texas", grade: "A", rank_global: 4, rank_delta: 12, total_score: 0.842, operator_score: 0.869, isSelf: true },
    { uid: "44012908", nickname: "Rhodes Admin", server: "KR", level: 109, avatar_id: "char_4045_heidi", grade: "B", rank_global: 5, rank_delta: -4, total_score: 0.773, operator_score: 0.741 },
];

const toEntry = (seed: ISeed) => ({
    id: seed.uid,
    uid: seed.uid,
    nickname: seed.nickname,
    nick_number: null,
    level: seed.level,
    avatar_id: seed.avatar_id,
    secretary: null,
    secretary_skin_id: null,
    server: seed.server,
    grade: seed.grade,
    total_score: seed.total_score,
    operator_score: seed.operator_score,
    stage_score: seed.total_score - 0.07,
    roguelike_score: seed.total_score - 0.19,
    sandbox_score: seed.total_score - 0.24,
    medal_score: seed.total_score - 0.11,
    base_score: seed.total_score - 0.05,
    skin_score: seed.total_score - 0.31,
    rank_global: seed.rank_global,
    rank_server: seed.rank_global,
    rank_delta: seed.rank_delta,
    isSelf: seed.isSelf ?? false,
});

const ENTRIES = SEEDS.map(toEntry);

const Frame = ({ children }: { children: ReactNode }) => <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-[0_1px_2px_rgb(0_0_0/0.04)]">{children}</div>;

export const TopRanks = () => (
    <Frame>
        <LeaderboardTable entries={ENTRIES} intervalLabel="since yesterday" onSort={() => {}} sort="total_score" />
    </Frame>
);

export const SortedByOperators = () => (
    <Frame>
        <LeaderboardTable entries={ENTRIES} intervalLabel="in the past 7 days" onSort={() => {}} sort="operator_score" />
    </Frame>
);

export const NoResults = () => (
    <Frame>
        <LeaderboardTable entries={[]} onSort={() => {}} sort="total_score" />
    </Frame>
);
