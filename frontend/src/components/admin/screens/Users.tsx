import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ExternalLinkIcon, RefreshCwIcon, SearchIcon, XIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { Skeleton } from "#/components/ui/skeleton";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { HCode, PageHead } from "../AdminShell";
import { SERVER_TINT } from "../Primitives";

const SERVERS = ["all", "EN", "JP", "KR", "CN"] as const;
type Server = (typeof SERVERS)[number];

function SegTabs<T extends string>({ value, items, onChange }: { value: T; items: { value: T; label: string }[]; onChange: (v: T) => void }): React.ReactElement {
    return (
        <div className="inline-flex gap-px rounded-[9px] border border-border bg-card p-0.75">
            {items.map((it) => (
                <button
                    key={it.value}
                    type="button"
                    onClick={() => onChange(it.value)}
                    className={cn("inline-flex h-6.5 cursor-pointer items-center rounded-md px-3 font-medium text-[12.5px] transition-colors", value === it.value ? "bg-background text-foreground shadow-xs/5" : "text-muted-foreground hover:text-foreground")}
                >
                    {it.label}
                </button>
            ))}
        </div>
    );
}

function serverTint(server: string): string {
    return SERVER_TINT[server.toUpperCase()] ?? SERVER_TINT.EN;
}

function UserAvatar({ user, size = 22 }: { user: { nickname: string | null; secretary: string | null; secretary_skin_id: string | null; server: string }; size?: number }): React.ReactElement {
    const [failed, setFailed] = useState(false);
    const url = getSecretaryAvatarURL({ secretary: user.secretary, secretary_skin_id: user.secretary_skin_id });
    return (
        <span className="relative inline-block shrink-0 overflow-hidden rounded-full align-[-7px]" style={{ width: size, height: size, background: serverTint(user.server) }}>
            {failed ? <span className="absolute inset-0 flex items-center justify-center font-bold text-[10px] text-white/85">{user.nickname?.[0]?.toUpperCase() ?? "?"}</span> : <img src={url} alt="" loading="lazy" decoding="async" className="absolute inset-0 size-full object-cover" onError={() => setFailed(true)} />}
        </span>
    );
}

export function Users(): React.ReactElement {
    const [server, setServer] = useState<Server>("all");
    const [query, setQuery] = useState("");
    const [opened, setOpened] = useState<IUserProfile | null>(null);

    // Use /search as the canonical list (when q="" it returns all public profiles
    // ordered by score). Filter client-side by server.
    const usersQuery = useQuery(searchUsersQueryOptions({ q: query || undefined, limit: 200 }));

    const entries = usersQuery.data?.entries ?? [];
    const filtered = useMemo(() => (server === "all" ? entries : entries.filter((u) => u.server.toUpperCase() === server)), [entries, server]);

    return (
        <>
            <PageHead
                kicker="Manage"
                title="Users"
                sub={
                    <>
                        Doctors with synced profiles. Pulled from <HCode>/search</HCode>. Role assignment lives on the backend <span className="font-mono">users</span> table; the admin panel doesn't yet expose it.
                    </>
                }
                action={
                    <Button variant="outline" size="sm" loading={usersQuery.isFetching} onClick={() => usersQuery.refetch()}>
                        <RefreshCwIcon />
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
                            <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search by nickname…" size="sm" />
                        </InputGroup>
                    </div>
                    <SegTabs<Server>
                        value={server}
                        onChange={setServer}
                        items={[
                            { value: "all", label: "All servers" },
                            { value: "EN", label: "EN" },
                            { value: "JP", label: "JP" },
                            { value: "KR", label: "KR" },
                            { value: "CN", label: "CN" },
                        ]}
                    />
                    <div className="flex-1" />
                    <span className="text-[12px] text-muted-foreground">
                        {filtered.length} of {entries.length} shown
                    </span>
                </div>
                {usersQuery.isPending ? (
                    <div className="space-y-2 p-4">
                        <Skeleton className="h-12" />
                        <Skeleton className="h-12" />
                        <Skeleton className="h-12" />
                    </div>
                ) : filtered.length === 0 ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">{server !== "all" ? <>No public profiles on the {server} server yet.</> : "No users match."}</div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full min-w-160 border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {["Doctor", "Server", "Level", "Score", "Grade", "Role", ""].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.map((u) => (
                                    <tr key={u.uid} onClick={() => setOpened(u)} onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && setOpened(u)} className="cursor-pointer border-border border-b last:border-0 hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]">
                                        <td className="px-3.5 py-2.5">
                                            <span className="mr-2 inline-block align-[-6px]">
                                                <UserAvatar user={u} />
                                            </span>
                                            <span className="font-medium">{u.nickname ?? "-"}</span>
                                            <span className="font-medium font-mono text-[11.5px] text-muted-foreground"> · UID&nbsp;{u.uid}</span>
                                        </td>
                                        <td className="px-3.5 py-2.5 font-mono">{u.server.toUpperCase()}</td>
                                        <td className="px-3.5 py-2.5 tabular-nums">{u.level ?? "-"}</td>
                                        <td className="px-3.5 py-2.5 tabular-nums">{u.total_score?.toLocaleString(undefined, { maximumFractionDigits: 2 }) ?? "-"}</td>
                                        <td className="px-3.5 py-2.5">{u.grade ? <Badge variant="outline">{u.grade}</Badge> : <span className="text-muted-foreground">-</span>}</td>
                                        <td className="px-3.5 py-2.5">
                                            <span className="font-mono text-[12px] text-muted-foreground">{u.role}</span>
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <Button variant="ghost" size="xs" render={<Link to="/user/$id" params={{ id: u.uid }} target="_blank" onClick={(e) => e.stopPropagation()} />}>
                                                <ExternalLinkIcon />
                                                Open
                                            </Button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {opened ? <UserDetailDrawer user={opened} onClose={() => setOpened(null)} /> : null}
        </>
    );
}

function UserDetailDrawer({ user, onClose }: { user: IUserProfile; onClose: () => void }): React.ReactElement {
    return (
        <>
            <button type="button" className="fixed inset-0 z-50 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label="Close drawer" />
            <aside className="fixed top-0 right-0 bottom-0 z-51 flex w-110 max-w-[92vw] flex-col border-border border-l bg-background shadow-[-20px_0_60px_oklch(0_0_0/0.18)]">
                <div className="flex items-center justify-between gap-2 border-border border-b px-4.5 py-3.5">
                    <div className="flex items-center gap-2.5">
                        <UserAvatar user={user} size={34} />
                        <div className="flex flex-col gap-0.5">
                            <span className="font-semibold text-[15px] leading-tight">{user.nickname ?? "-"}</span>
                            <span className="font-mono text-[11.5px] text-muted-foreground">
                                UID&nbsp;{user.uid} · {user.server.toUpperCase()}
                                {user.level ? ` · Level ${user.level}` : ""}
                            </span>
                        </div>
                    </div>
                    <Button variant="ghost" size="icon-sm" onClick={onClose} aria-label="Close">
                        <XIcon />
                    </Button>
                </div>

                <div className="flex-1 overflow-auto p-4.5">
                    <div className="mb-4 flex flex-wrap items-center gap-2">
                        {user.grade ? <Badge variant="outline">grade {user.grade}</Badge> : null}
                        <Badge variant={user.role === "super_admin" ? "default" : "outline"}>
                            <span className="font-mono">{user.role}</span>
                        </Badge>
                        {user.public_profile ? <Badge variant="success">public</Badge> : <Badge variant="warning">private</Badge>}
                    </div>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Profile</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <ScoreLine label="Total score" value={user.total_score} />
                            <ScoreLine label="Operators" value={user.operator_count} />
                            <ScoreLine label="Items" value={user.item_count} />
                            <ScoreLine label="Skins" value={user.skin_count} />
                            <ScoreLine label="LMD" value={user.lmd} />
                            <ScoreLine label="Orundum" value={user.orundum} />
                            <ScoreLine label="Sanity" value={user.sanity != null && user.max_sanity != null ? `${user.sanity} / ${user.max_sanity}` : "-"} last />
                        </CardContent>
                    </Card>

                    <div className="h-3.5" />

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">Actions</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <Button variant="outline" size="sm" className="w-full" render={<Link to="/user/$id" params={{ id: user.uid }} target="_blank" />}>
                                <ExternalLinkIcon />
                                Open public profile
                            </Button>
                            <div className="mt-3 text-[11.5px] text-muted-foreground">
                                Role assignment, force-resync, and session-revoke require direct DB / <span className="font-mono">/auth/update-settings</span> access - not yet exposed as REST endpoints.
                            </div>
                        </CardContent>
                    </Card>
                </div>
            </aside>
        </>
    );
}

function ScoreLine({ label, value, last }: { label: string; value: number | string | null | undefined; last?: boolean }): React.ReactElement {
    return (
        <div className={cn("flex items-center justify-between py-1.5", !last && "border-border border-b")}>
            <span className="text-[12.5px] text-muted-foreground">{label}</span>
            <span className="font-mono text-[12.5px] tabular-nums">{typeof value === "number" ? value.toLocaleString(undefined, { maximumFractionDigits: 2 }) : (value ?? "-")}</span>
        </div>
    );
}
