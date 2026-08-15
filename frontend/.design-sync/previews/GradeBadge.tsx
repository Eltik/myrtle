import { Avatar, AvatarFallback, AvatarImage, GradeBadge } from "frontend";

const GRADE_SCALE = [
    { grade: "SS+", note: "Top 0.1%" },
    { grade: "SS", note: "Top 1%" },
    { grade: "S", note: "Top 5%" },
    { grade: "A", note: "Top 20%" },
    { grade: "B", note: "Top 50%" },
    { grade: "C", note: "Ranked" },
];

const ROWS = [
    { rank: 1, uid: "10000817", nickname: "Kyostinv", grade: "SS+", avatar: "char_4064_mlynar", score: "98.4%" },
    { rank: 2, uid: "21748302", nickname: "Doktah", grade: "SS", avatar: "char_263_skadi", score: "96.1%" },
    { rank: 3, uid: "30914477", nickname: "Ceylonade", grade: "S", avatar: "char_180_amgoat", score: "91.7%" },
    { rank: 4, uid: "18220561", nickname: "Eltik", grade: "A", avatar: "char_102_texas", score: "84.2%" },
];

export const GradeScale = () => (
    <div className="flex flex-wrap items-center gap-5">
        {GRADE_SCALE.map(({ grade, note }) => (
            <div className="flex flex-col items-center gap-1.5" key={grade}>
                <GradeBadge grade={grade} />
                <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{note}</span>
            </div>
        ))}
    </div>
);

export const InLeaderboardRows = () => (
    <div className="w-full max-w-md divide-y divide-border/60 overflow-hidden rounded-2xl border border-border bg-card">
        {ROWS.map((row) => (
            <div className="flex items-center gap-3 px-4 py-3" key={row.uid}>
                <span className="w-8 font-mono font-semibold text-foreground text-sm tabular-nums leading-none">#{row.rank}</span>
                <Avatar className="size-9 rounded-[10px]">
                    <AvatarImage alt={row.nickname} src={`https://api.myrtle.moe/api/avatar/${row.avatar}`} />
                    <AvatarFallback className="rounded-[10px] text-xs">{row.nickname.slice(0, 2).toUpperCase()}</AvatarFallback>
                </Avatar>
                <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                    <span className="truncate font-sans font-semibold text-[13.5px] text-foreground leading-tight tracking-tight">{row.nickname}</span>
                    <span className="font-mono text-[11px] text-muted-foreground tabular-nums leading-none">#{row.uid}</span>
                </span>
                <GradeBadge grade={row.grade} />
                <span className="w-14 text-right font-mono font-semibold text-[13px] text-foreground tabular-nums">{row.score}</span>
            </div>
        ))}
    </div>
);

export const Unranked = () => (
    <div className="flex w-full max-w-sm items-center gap-3 rounded-xl border border-border bg-card px-4 py-3.5">
        <GradeBadge grade={null} />
        <div className="flex min-w-0 flex-col gap-1">
            <span className="font-sans font-semibold text-[13.5px] text-foreground leading-tight tracking-tight">Not graded yet</span>
            <span className="font-mono text-[11px] text-muted-foreground leading-none">Sync a public profile to receive a composite grade.</span>
        </div>
    </div>
);
