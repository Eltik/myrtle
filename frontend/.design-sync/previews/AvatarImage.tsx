import { Avatar, AvatarFallback, AvatarImage, Card } from "frontend";

const avatar = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const SQUAD = [
    { id: "char_4064_mlynar", initials: "ML", name: "Mlynar", role: "Guard" },
    { id: "char_263_skadi", initials: "SK", name: "Skadi", role: "Guard" },
    { id: "char_180_amgoat", initials: "EY", name: "Eyjafjalla", role: "Caster" },
    { id: "char_249_mlyss", initials: "MU", name: "Muelsyse", role: "Vanguard" },
] as const;

export const OperatorRoster = () => (
    <div className="flex flex-wrap gap-5">
        {SQUAD.map((op) => (
            <div className="flex w-20 flex-col items-center gap-1.5" key={op.id}>
                <Avatar className="size-12 rounded-xl border border-border">
                    <AvatarImage alt={op.name} src={avatar(op.id)} />
                    <AvatarFallback className="rounded-xl text-xs">{op.initials}</AvatarFallback>
                </Avatar>
                <span className="truncate text-center font-medium text-foreground text-xs">{op.name}</span>
                <span className="text-[10px] text-muted-foreground">{op.role}</span>
            </div>
        ))}
    </div>
);

export const SkinArt = () => (
    <div className="flex items-start gap-4">
        {[
            { id: "char_263_skadi", initials: "SK", name: "Skadi", sub: "Default E2" },
            { id: "char_1028_texas2", initials: "TX", name: "Texas the Omertosa", sub: "Default E2" },
            { id: "char_002_amiya", initials: "AM", name: "Amiya", sub: "Default E2" },
        ].map((skin) => (
            <div className="flex w-28 flex-col gap-2" key={skin.id}>
                <Avatar className="size-16 rounded-lg border border-border">
                    <AvatarImage alt={skin.name} src={avatar(skin.id)} />
                    <AvatarFallback className="rounded-lg">{skin.initials}</AvatarFallback>
                </Avatar>
                <div className="min-w-0">
                    <p className="truncate font-medium text-foreground text-xs">{skin.name}</p>
                    <p className="text-[10px] text-muted-foreground">{skin.sub}</p>
                </div>
            </div>
        ))}
    </div>
);

export const LeaderboardRows = () => (
    <Card className="max-w-md divide-y divide-border">
        {[
            { id: "char_4064_mlynar", initials: "VE", name: "Dr. Vector", rank: 1, score: "18,402" },
            { id: "char_263_skadi", initials: "KA", name: "Dr. Kal", rank: 2, score: "17,915" },
            { id: "char_180_amgoat", initials: "SI", name: "Dr. Silence", rank: 3, score: "17,204" },
        ].map((row) => (
            <div className="flex items-center gap-3 px-4 py-2.5" key={row.rank}>
                <span className="w-5 font-mono text-muted-foreground text-xs tabular-nums">#{row.rank}</span>
                <Avatar className="size-8">
                    <AvatarImage alt={row.name} src={avatar(row.id)} />
                    <AvatarFallback className="text-[10px]">{row.initials}</AvatarFallback>
                </Avatar>
                <span className="flex-1 truncate font-medium text-foreground text-sm">{row.name}</span>
                <span className="font-mono text-foreground text-xs tabular-nums">{row.score}</span>
            </div>
        ))}
    </Card>
);
