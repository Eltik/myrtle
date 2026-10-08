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

/** `rowFromScore`: one ranked row per standing, `value` the ranked score (0..1). */
const toRow = (seed: ISeed, value: number) => ({
    uid: seed.uid,
    nickname: seed.nickname,
    nick_number: null,
    level: seed.level,
    avatar_id: seed.avatar_id,
    server: seed.server,
    grade: seed.grade,
    rank: seed.rank_global,
    delta: seed.rank_delta,
    value,
    isSelf: seed.isSelf ?? false,
});

const BY_TOTAL = SEEDS.map((s) => toRow(s, s.total_score));
const BY_OPERATORS = [...SEEDS].sort((a, b) => b.operator_score - a.operator_score).map((s, i) => ({ ...toRow(s, s.operator_score), rank: i + 1 }));

/** Originite Prime holdings, `rowFromItem`: the value is the quantity, the bar a share of the top holding. */
const OP_HOLDINGS = [6421, 5180, 3977, 2210, 1046];
const BY_ITEM = SEEDS.map((s, i) => toRow(s, OP_HOLDINGS[i]));

/** The item catalog as the page holds it once loaded; `meta` is the item-table row (only the joined fields are read here). */
const CATALOG = {
    items: [{ item_id: "4002", holders: 2412, top: 6421, total_quantity: 1873390, meta: null, name: "Originite Prime", rarityNum: 6, iconId: "DIAMOND" }],
    population: 2629,
};
const EMPTY_CATALOG = { items: [], population: null };

const noop = () => {};

const Frame = ({ children }: { children: ReactNode }) => <div className="overflow-hidden rounded-2xl border border-border bg-card shadow-[0_1px_2px_rgb(0_0_0/0.04)]">{children}</div>;

export const TopRanks = () => (
    <Frame>
        <LeaderboardTable rows={BY_TOTAL} ranking={{ kind: "score", sort: "total_score" }} onRanking={noop} catalog={EMPTY_CATALOG} materials={undefined as never} topValue={null as never} intervalKey="leaderboard.interval.day.since" />
    </Frame>
);

export const SortedByOperators = () => (
    <Frame>
        <LeaderboardTable rows={BY_OPERATORS} ranking={{ kind: "score", sort: "operator_score" }} onRanking={noop} catalog={EMPTY_CATALOG} materials={undefined as never} topValue={null as never} intervalKey="leaderboard.interval.week.since" />
    </Frame>
);

/** Ranked by an item: counts with bars against the top holding; grades still shown. */
export const RankedByItem = () => (
    <Frame>
        <LeaderboardTable rows={BY_ITEM} ranking={{ kind: "item", item: "4002" }} onRanking={noop} catalog={CATALOG} materials={undefined as never} topValue={OP_HOLDINGS[0]} />
    </Frame>
);

export const NoResults = () => (
    <Frame>
        <LeaderboardTable rows={[]} ranking={{ kind: "score", sort: "total_score" }} onRanking={noop} catalog={EMPTY_CATALOG} materials={undefined as never} topValue={null as never} />
    </Frame>
);
