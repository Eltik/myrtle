import { useQueries, useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ExternalLinkIcon, SearchIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { operatorNoteAuditLogQueryOptions } from "#/lib/api/admin";
import { operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { userQueryOptions } from "#/lib/api/user";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import { HCode, PageHead } from "../AdminShell";

function ActorCell({ uid }: { uid: string }): React.ReactElement {
    const profile = useQuery({ ...userQueryOptions(uid), retry: 0 });
    const u = profile.data;
    return (
        <Link to="/user/$id" params={{ id: uid }} target="_blank" className="inline-flex items-center gap-2 hover:underline">
            <span className="relative inline-block size-5.5 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
                {u ? <img src={getSecretaryAvatarURL({ secretary: u.secretary, secretary_skin_id: u.secretary_skin_id })} alt="" loading="lazy" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} /> : null}
            </span>
            <span>
                <span className="font-medium">{u?.nickname ?? "-"}</span>
                <span className="ml-1 font-mono text-[11.5px] text-muted-foreground">UID {uid}</span>
            </span>
        </Link>
    );
}

function OperatorTargetCell({ operatorId }: { operatorId: string }): React.ReactElement {
    const opsQuery = useQuery(operatorsIndexQueryOptions());
    const op = opsQuery.data?.find((o) => o.id === operatorId);
    return (
        <Link to="/operators/$id" params={{ id: operatorId }} target="_blank" className="inline-flex items-center gap-2 hover:underline">
            <span className="inline-flex size-6 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[10px]">
                <OperatorAvatar charId={operatorId} name={op?.name ?? "?"} />
            </span>
            <span>
                <span className="font-medium">{op?.name ?? operatorId}</span>
                <span className="ml-1 font-mono text-[11.5px] text-muted-foreground">{operatorId}</span>
            </span>
            <ExternalLinkIcon className="size-3 opacity-60" />
        </Link>
    );
}

interface IFlatAuditRow {
    when: string;
    actor: string;
    action: string;
    target: string;
    detail: string;
    severity: "info" | "success" | "warning" | "muted";
}

function severityBadge(s: IFlatAuditRow["severity"]): React.ReactElement {
    if (s === "info") return <Badge variant="info">info</Badge>;
    if (s === "success") return <Badge variant="success">success</Badge>;
    if (s === "warning") return <Badge variant="warning">warning</Badge>;
    return <Badge variant="outline">system</Badge>;
}

function formatRelative(iso: string): string {
    const t = Date.parse(iso);
    if (!Number.isFinite(t)) return iso;
    const diff = (Date.now() - t) / 1000;
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} h ago`;
    if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} d ago`;
    return new Date(t).toLocaleDateString();
}

export function Audit(): React.ReactElement {
    const [filter, setFilter] = useState<"all" | "operator-notes" | "info" | "warning">("all");
    const [search, setSearch] = useState("");

    const notesQuery = useQuery(operatorNotesListQueryOptions());

    // Fetch the audit log for each operator that has a note. Limited to top 12 to avoid hammering the backend.
    const operatorIds = (notesQuery.data ?? []).slice(0, 12).map((n) => n.operator_id);
    const auditQueries = useQueries({
        queries: operatorIds.map((id) => ({ ...operatorNoteAuditLogQueryOptions(id), enabled: true })),
    });

    const rows: IFlatAuditRow[] = useMemo(() => {
        const out: IFlatAuditRow[] = [];
        auditQueries.forEach((q, idx) => {
            const id = operatorIds[idx];
            if (!q.data || !id) return;
            for (const entry of q.data) {
                out.push({
                    when: entry.changed_at,
                    actor: entry.changed_by,
                    action: "operator-notes.update",
                    target: id,
                    detail: `${entry.field_name}: ${entry.old_value ? `-${entry.old_value.slice(0, 40)}` : ""} ${entry.new_value ? `+${entry.new_value.slice(0, 40)}` : ""}`.trim(),
                    severity: "info",
                });
            }
        });
        return out.sort((a, b) => Date.parse(b.when) - Date.parse(a.when));
    }, [auditQueries, operatorIds]);

    const filtered = useMemo(() => {
        return rows.filter((r) => {
            if (filter === "info" && r.severity !== "info") return false;
            if (filter === "warning" && r.severity !== "warning") return false;
            if (filter === "operator-notes" && r.action !== "operator-notes.update") return false;
            if (search) {
                const q = search.toLowerCase();
                if (!r.actor.toLowerCase().includes(q) && !r.action.toLowerCase().includes(q) && !r.target.toLowerCase().includes(q) && !r.detail.toLowerCase().includes(q)) return false;
            }
            return true;
        });
    }, [rows, filter, search]);

    const loading = notesQuery.isPending || auditQueries.some((q) => q.isPending);

    return (
        <>
            <PageHead
                kicker="Operate"
                title="Audit log"
                sub={
                    <>
                        Append-only edit trail from <HCode>operator_notes_audit</HCode>. Each row was written when an admin saved an operator note. Permission grants and tier-list publishes don't yet emit audit rows in v3.
                    </>
                }
            />

            <div className="relative overflow-hidden rounded-2xl border border-border bg-card shadow-xs/5 before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] dark:before:shadow-[0_-1px_--theme(--color-white/6%)]">
                <div className="flex flex-wrap items-center gap-2.5 border-border border-b p-3.5">
                    <div className="w-full min-w-0 max-w-90 sm:min-w-70 sm:flex-1">
                        <InputGroup>
                            <InputGroupAddon>
                                <SearchIcon />
                            </InputGroupAddon>
                            <Input placeholder="Filter by actor UID, action, target, detail…" size="sm" value={search} onChange={(e) => setSearch(e.target.value)} />
                        </InputGroup>
                    </div>
                    <div className="inline-flex max-w-full gap-px overflow-x-auto rounded-[9px] border border-border bg-card p-0.75">
                        {(["all", "operator-notes", "info", "warning"] as const).map((f) => (
                            <button key={f} type="button" onClick={() => setFilter(f)} className={cn("inline-flex h-6.5 cursor-pointer items-center rounded-md px-3 font-medium text-[12.5px] transition-colors", filter === f ? "bg-background text-foreground shadow-xs/5" : "text-muted-foreground hover:text-foreground")}>
                                {f === "all" ? "All" : f === "operator-notes" ? "Operator notes" : f.charAt(0).toUpperCase() + f.slice(1)}
                            </button>
                        ))}
                    </div>
                    <div className="flex-1" />
                    <span className="text-[12px] text-muted-foreground">{filtered.length} events</span>
                </div>
                {loading ? (
                    <div className="space-y-2 p-4">
                        <Skeleton className="h-10" />
                        <Skeleton className="h-10" />
                        <Skeleton className="h-10" />
                    </div>
                ) : filtered.length === 0 ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">No audit rows yet.</div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full min-w-190 border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {["When", "Actor", "Action", "Target", "Detail", "Severity"].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.slice(0, 200).map((r, i) => (
                                    <tr
                                        // biome-ignore lint/suspicious/noArrayIndexKey: audit rows are positional within the merged stream
                                        key={i}
                                        className="border-border border-b last:border-0 hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]"
                                    >
                                        <td className="px-3.5 py-2.5 text-muted-foreground">{formatRelative(r.when)}</td>
                                        <td className="px-3.5 py-2.5">
                                            <ActorCell uid={r.actor} />
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <span className="font-mono text-[12.5px]">{r.action}</span>
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <OperatorTargetCell operatorId={r.target} />
                                        </td>
                                        <td className="px-3.5 py-2.5 text-[12.5px] text-muted-foreground">{r.detail}</td>
                                        <td className="px-3.5 py-2.5">{severityBadge(r.severity)}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </>
    );
}
