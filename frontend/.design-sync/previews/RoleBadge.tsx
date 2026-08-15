import { RoleBadge } from "frontend";

const roleRows = [
    { uid: "10289471", nickname: "Kyostinv", server: "EN", role: "super_admin", joined: "3y ago" },
    { uid: "42118900", nickname: "Dr. Reisen", server: "JP", role: "tier_list_admin", joined: "14mo ago" },
    { uid: "77004512", nickname: "Ceylonade", server: "EN", role: "tier_list_editor", joined: "8mo ago" },
    { uid: "51930277", nickname: "Hoshiguma", server: "CN", role: "user", joined: "3w ago" },
];

export function AllRoles() {
    return (
        <div className="flex flex-col items-start gap-3">
            <RoleBadge role="super_admin" />
            <RoleBadge role="tier_list_admin" />
            <RoleBadge role="tier_list_editor" />
            <RoleBadge role="user" />
        </div>
    );
}

export function InUsersTable() {
    return (
        <div className="relative max-w-2xl overflow-hidden rounded-2xl border border-border bg-card">
            <table className="w-full border-collapse text-[13px]">
                <thead>
                    <tr>
                        {["Doctor", "Server", "Role", "Joined"].map((h) => (
                            <th key={h} className="border-border border-b px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                {h}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {roleRows.map((r) => (
                        <tr key={r.uid} className="border-border border-b last:border-0">
                            <td className="px-3.5 py-2.5">
                                <span className="font-medium">{r.nickname}</span>
                                <span className="ml-1.5 font-mono text-[11.5px] text-muted-foreground">UID {r.uid}</span>
                            </td>
                            <td className="px-3.5 py-2.5 font-mono text-muted-foreground">{r.server}</td>
                            <td className="px-3.5 py-2.5">
                                <RoleBadge role={r.role} />
                            </td>
                            <td className="whitespace-nowrap px-3.5 py-2.5 text-muted-foreground tabular-nums">{r.joined}</td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

export function InlineWithActor() {
    return (
        <div className="flex w-fit flex-col gap-2.5">
            <div className="flex items-center gap-2 text-[13px]">
                <span className="font-medium">Kyostinv</span>
                <RoleBadge role="super_admin" />
                <span className="text-[12px] text-muted-foreground">published version 14 of /endgame-dps</span>
            </div>
            <div className="flex items-center gap-2 text-[13px]">
                <span className="font-medium">Ceylonade</span>
                <RoleBadge role="tier_list_editor" />
                <span className="text-[12px] text-muted-foreground">edited the notes on Skadi the Corrupting Heart</span>
            </div>
            <div className="flex items-center gap-2 text-[13px]">
                <span className="font-medium">Hoshiguma</span>
                <RoleBadge role="user" />
                <span className="text-[12px] text-muted-foreground">synced a roster from the EN server</span>
            </div>
        </div>
    );
}
