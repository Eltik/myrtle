import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ExternalLinkIcon, RefreshCwIcon, SearchIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { globalAuditLogQueryOptions, type IAuditLogActor, type IAuditLogEntry } from "#/lib/api/admin";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import { HCode, PageHead } from "../AdminShell";

function ActorCell({ actor }: { actor: IAuditLogActor }): React.ReactElement {
    // actor.uid is null only when the FK target user has been hard-deleted.
    if (!actor.uid) {
        const shortUuid = `${actor.user_id.slice(0, 8)}…`;
        return (
            <span className="inline-flex items-center gap-2" title={`Internal user_id ${actor.user_id} - referenced user no longer exists.`}>
                <span className="inline-block size-5.5 shrink-0 rounded-full bg-muted" />
                <span>
                    <span className="font-medium text-muted-foreground">Deleted user</span>
                    <span className="ml-1 font-mono text-[11.5px] text-muted-foreground/70">{shortUuid}</span>
                </span>
            </span>
        );
    }

    return (
        <Link to="/user/$id" params={{ id: actor.uid }} target="_blank" className="inline-flex items-center gap-2 hover:underline">
            <span className="relative inline-block size-5.5 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
                <img src={getSecretaryAvatarURL({ secretary: actor.secretary, secretary_skin_id: actor.secretary_skin_id })} alt="" loading="lazy" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} />
            </span>
            <span>
                <span className="font-medium">{actor.nickname ?? "-"}</span>
                <span className="ml-1 font-mono text-[11.5px] text-muted-foreground">UID {actor.uid}</span>
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

type FieldFilter = "all" | "pros" | "cons" | "notes" | "trivia" | "summary";

function diffPreview(entry: IAuditLogEntry): React.ReactNode {
    const oldVal = entry.old_value?.trim() ?? "";
    const newVal = entry.new_value?.trim() ?? "";
    const truncate = (s: string, n = 80): string => (s.length > n ? `${s.slice(0, n)}…` : s);
    if (!oldVal && newVal) {
        return (
            <span className="text-[12.5px]">
                <Badge variant="success" className="mr-1.5 align-middle">
                    added
                </Badge>
                <span className="text-muted-foreground">{truncate(newVal)}</span>
            </span>
        );
    }
    if (oldVal && !newVal) {
        return (
            <span className="text-[12.5px]">
                <Badge variant="warning" className="mr-1.5 align-middle">
                    cleared
                </Badge>
                <span className="text-muted-foreground line-through">{truncate(oldVal)}</span>
            </span>
        );
    }
    return (
        <span className="text-[12.5px] text-muted-foreground">
            <span className="text-red-500 line-through">{truncate(oldVal, 40)}</span>
            <span className="mx-1 opacity-50">→</span>
            <span className="text-emerald-600 dark:text-emerald-400">{truncate(newVal, 40)}</span>
        </span>
    );
}

const PAGE_SIZE = 100;

export function Audit(): React.ReactElement {
    const [fieldFilter, setFieldFilter] = useState<FieldFilter>("all");
    const [search, setSearch] = useState("");

    const auditQuery = useQuery(globalAuditLogQueryOptions({ limit: PAGE_SIZE }));

    const filtered = useMemo(() => {
        const entries = auditQuery.data?.entries ?? [];
        return entries.filter((r) => {
            if (fieldFilter !== "all" && r.field_name !== fieldFilter) return false;
            if (search) {
                const q = search.toLowerCase();
                if (
                    !(r.actor.nickname ?? "").toLowerCase().includes(q) &&
                    !(r.actor.uid ?? "").toLowerCase().includes(q) &&
                    !r.actor.user_id.toLowerCase().includes(q) &&
                    !r.field_name.toLowerCase().includes(q) &&
                    !r.operator_id.toLowerCase().includes(q) &&
                    !(r.old_value ?? "").toLowerCase().includes(q) &&
                    !(r.new_value ?? "").toLowerCase().includes(q)
                )
                    return false;
            }
            return true;
        });
    }, [auditQuery.data, fieldFilter, search]);

    const loading = auditQuery.isPending;
    const total = auditQuery.data?.total ?? 0;
    const shown = auditQuery.data?.entries.length ?? 0;

    return (
        <>
            <PageHead
                kicker="Operate"
                title="Audit log"
                sub={
                    <>
                        Append-only edit trail from <HCode>operator_notes_audit_log</HCode>. Each row was written when an admin saved an operator note. Permission grants and tier-list publishes don't yet emit audit rows in v3.
                    </>
                }
                action={
                    <Button variant="outline" size="sm" onClick={() => auditQuery.refetch()} disabled={auditQuery.isFetching}>
                        <RefreshCwIcon className={cn(auditQuery.isFetching && "animate-spin")} />
                        Refresh
                    </Button>
                }
            />

            <div className="relative overflow-hidden rounded-2xl border border-border bg-card shadow-xs/5 before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] dark:before:shadow-[0_-1px_--theme(--color-white/6%)]">
                <div className="flex flex-wrap items-center gap-2.5 border-border border-b p-3.5">
                    <div className="w-full min-w-0 max-w-90 sm:min-w-70 sm:flex-1">
                        <InputGroup>
                            <InputGroupAddon>
                                <SearchIcon />
                            </InputGroupAddon>
                            <Input placeholder="Filter by actor, operator, field, content…" size="sm" value={search} onChange={(e) => setSearch(e.target.value)} />
                        </InputGroup>
                    </div>
                    <div className="inline-flex max-w-full gap-px overflow-x-auto rounded-[9px] border border-border bg-card p-0.75">
                        {(["all", "pros", "cons", "notes", "trivia", "summary"] as const).map((f) => (
                            <button
                                key={f}
                                type="button"
                                onClick={() => setFieldFilter(f)}
                                className={cn("inline-flex h-6.5 cursor-pointer items-center rounded-md px-3 font-medium text-[12.5px] transition-colors", fieldFilter === f ? "bg-background text-foreground shadow-xs/5" : "text-muted-foreground hover:text-foreground")}
                            >
                                {f === "all" ? "All fields" : f.charAt(0).toUpperCase() + f.slice(1)}
                            </button>
                        ))}
                    </div>
                    <div className="flex-1" />
                    <span className="text-[12px] text-muted-foreground">
                        {filtered.length} of {shown.toLocaleString()} shown
                        {total > shown ? <span className="ml-1 opacity-70">/ {total.toLocaleString()} total</span> : null}
                    </span>
                </div>
                {loading ? (
                    <div className="space-y-2 p-4">
                        <Skeleton className="h-10" />
                        <Skeleton className="h-10" />
                        <Skeleton className="h-10" />
                    </div>
                ) : auditQuery.isError ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-destructive">Failed to load audit log: {(auditQuery.error as Error)?.message ?? "unknown error"}</div>
                ) : filtered.length === 0 ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">{shown === 0 ? "No audit rows yet." : "No rows match your filter."}</div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full min-w-220 border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {["When", "Actor", "Operator", "Field", "Change"].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.map((r) => (
                                    <tr key={r.id} className="border-border border-b last:border-0 hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]">
                                        <td className="whitespace-nowrap px-3.5 py-2.5 text-muted-foreground" title={r.changed_at}>
                                            {formatRelative(r.changed_at)}
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <ActorCell actor={r.actor} />
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <OperatorTargetCell operatorId={r.operator_id} />
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <Badge variant="outline" className="font-mono">
                                                {r.field_name}
                                            </Badge>
                                        </td>
                                        <td className="max-w-105 px-3.5 py-2.5">{diffPreview(r)}</td>
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
