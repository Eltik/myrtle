import { ServerTag } from "frontend";

const SERVERS = ["EN", "JP", "CN", "KR", "TW"];

const ROWS = [
    { uid: "10000817", nickname: "Kyostinv", server: "EN", level: 120 },
    { uid: "21748302", nickname: "Doktah", server: "JP", level: 120 },
    { uid: "30914477", nickname: "Ceylonade", server: "CN", level: 118 },
    { uid: "44012908", nickname: "Rhodes Admin", server: "KR", level: 109 },
];

export const AllServers = () => (
    <div className="flex flex-wrap items-center gap-2">
        {SERVERS.map((code) => (
            <ServerTag key={code} server={code} />
        ))}
    </div>
);

export const InDoctorRows = () => (
    <div className="w-full max-w-md divide-y divide-border/60 overflow-hidden rounded-2xl border border-border bg-card">
        {ROWS.map((row) => (
            <div className="flex items-center gap-3 px-4 py-3" key={row.uid}>
                <span className="min-w-0 flex-1 truncate font-sans font-semibold text-[13.5px] text-foreground leading-tight tracking-tight">{row.nickname}</span>
                <ServerTag server={row.server} />
                <span className="w-14 text-right font-mono text-muted-foreground text-xs tabular-nums">Lv {row.level}</span>
            </div>
        ))}
    </div>
);

export const UnknownServer = () => (
    <div className="flex items-center gap-3">
        <ServerTag server="EN" />
        <ServerTag server="bili" />
        <span className="font-mono text-[11px] text-muted-foreground leading-none">Unmapped server codes fall back to the muted token.</span>
    </div>
);
