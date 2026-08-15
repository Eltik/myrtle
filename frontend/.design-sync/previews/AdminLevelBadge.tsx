import { AdminLevelBadge } from "frontend";

const grants = [
    { uid: "10289471", nickname: "Kyostinv", list: "/endgame-dps", level: "Admin" as const, granted: "3mo ago" },
    { uid: "42118900", nickname: "Dr. Reisen", list: "/module-priority-2025", level: "Publish" as const, granted: "6w ago" },
    { uid: "77004512", nickname: "Ceylonade", list: "/cc12-sandbox-picks", level: "Edit" as const, granted: "12d ago" },
    { uid: "51930277", nickname: "Hoshiguma", list: "/rogue-trader-tiers", level: "View" as const, granted: "4d ago" },
];

export function AllLevels() {
    return (
        <div className="flex flex-col items-start gap-3">
            <AdminLevelBadge level="Admin" />
            <AdminLevelBadge level="Publish" />
            <AdminLevelBadge level="Edit" />
            <AdminLevelBadge level="View" />
        </div>
    );
}

export function InPermissionsTable() {
    return (
        <div className="relative max-w-2xl overflow-hidden rounded-2xl border border-border bg-card">
            <table className="w-full border-collapse text-[13px]">
                <thead>
                    <tr>
                        {["Doctor", "Tier list", "Access", "Granted"].map((h) => (
                            <th key={h} className="border-border border-b px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                {h}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {grants.map((g) => (
                        <tr key={g.uid} className="border-border border-b last:border-0">
                            <td className="px-3.5 py-2.5">
                                <span className="font-medium">{g.nickname}</span>
                                <span className="ml-1.5 font-mono text-[11.5px] text-muted-foreground">UID {g.uid}</span>
                            </td>
                            <td className="px-3.5 py-2.5 font-mono text-muted-foreground">{g.list}</td>
                            <td className="px-3.5 py-2.5">
                                <AdminLevelBadge level={g.level} />
                            </td>
                            <td className="whitespace-nowrap px-3.5 py-2.5 text-muted-foreground tabular-nums">{g.granted}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

export function AccessLadder() {
    return (
        <div className="flex max-w-md flex-col gap-2.5">
            {[
                { level: "View" as const, what: "Read the list and its version history" },
                { level: "Edit" as const, what: "Move operators between tiers" },
                { level: "Publish" as const, what: "Snapshot a new public version" },
                { level: "Admin" as const, what: "Grant access and delete the list" },
            ].map((r) => (
                <div key={r.level} className="flex items-center gap-3 border-border border-b pb-2.5 last:border-0 last:pb-0">
                    <span className="w-20 shrink-0">
                        <AdminLevelBadge level={r.level} />
                    </span>
                    <span className="text-[12.5px] text-muted-foreground">{r.what}</span>
                </div>
            ))}
        </div>
    );
}
