import { queryOptions, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { EditIcon, ExternalLinkIcon, MoreHorizontalIcon, PlusIcon, SearchIcon, TagIcon, Trash2Icon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { Skeleton } from "#/components/ui/skeleton";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { grantTierListPermissionFn, revokeTierListPermissionFn, type TierListPermissionLevel, tierListPermissionsQueryOptions } from "#/lib/api/admin";
import { browseTierListsQueryOptions, deleteTierListFn, type ITierListBrowseItem, setTierListFlairFn, tierListFlairsQueryOptions } from "#/lib/api/tier-lists";
import { type ISearchPage, searchUsersFn, searchUsersQueryOptions } from "#/lib/api/user";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { PageHead } from "../AdminShell";

const LEVELS = ["view", "edit", "publish", "admin"] as const;

function levelLabel(level: TierListPermissionLevel): string {
    return level.charAt(0).toUpperCase() + level.slice(1);
}

function allUsersByIdQueryOptions() {
    return queryOptions({
        queryKey: ["admin", "users", "by-id-map"],
        queryFn: async (): Promise<Map<string, IUserProfile>> => {
            const map = new Map<string, IUserProfile>();
            let offset = 0;
            const pageSize = 200;
            for (let i = 0; i < 25; i++) {
                const page: ISearchPage = await searchUsersFn({ data: { limit: pageSize, offset } });
                for (const entry of page.entries) map.set(entry.id, entry);
                if (page.entries.length < pageSize) break;
                offset += pageSize;
            }
            return map;
        },
        staleTime: 5 * 60 * 1000,
        gcTime: 30 * 60 * 1000,
    });
}

function LevelDots({ level, onChange }: { level: TierListPermissionLevel; onChange: (l: TierListPermissionLevel) => void }): React.ReactElement {
    return (
        <div className="inline-flex gap-px rounded-lg border border-border bg-card p-0.5">
            {LEVELS.map((l) => (
                <button
                    type="button"
                    key={l}
                    onClick={() => onChange(l)}
                    className={cn(
                        "cursor-pointer rounded-md px-2 py-1 font-medium text-[11.5px] transition-colors",
                        l === level && l === "admin" && "bg-primary text-primary-foreground",
                        l === level && l !== "admin" && "bg-[color-mix(in_srgb,var(--foreground)_6%,transparent)] text-foreground",
                        l !== level && "text-muted-foreground hover:text-foreground",
                    )}
                >
                    {levelLabel(l)}
                </button>
            ))}
        </div>
    );
}

function PermUserCell({ userId, lookup }: { userId: string; lookup: Map<string, IUserProfile> | undefined }): React.ReactElement {
    const u = lookup?.get(userId);
    const fallback = "linear-gradient(135deg, oklch(0.7 0.16 84), oklch(0.85 0.10 100))";
    return (
        <span className="flex items-center gap-2">
            <span className="relative inline-block size-5.5 shrink-0 overflow-hidden rounded-full align-[-6px]" style={{ background: fallback }}>
                {u ? <img src={getSecretaryAvatarURL({ secretary: u.secretary, secretary_skin_id: u.secretary_skin_id })} alt="" loading="lazy" decoding="async" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} /> : null}
            </span>
            <span>
                <span className="font-medium">{u?.nickname ?? "-"}</span>
                {u?.uid ? <span className="ml-1.5 font-medium font-mono text-[11.5px] text-muted-foreground">UID {u.uid}</span> : <span className="ml-1.5 font-medium font-mono text-[10.5px] text-muted-foreground">id {userId.slice(0, 8)}…</span>}
            </span>
        </span>
    );
}

export function Permissions(): React.ReactElement {
    const { isAuthenticated } = useAuth();
    const browseQuery = useQuery(browseTierListsQueryOptions());
    const flairsQuery = useQuery(tierListFlairsQueryOptions());
    const lookupQuery = useQuery(allUsersByIdQueryOptions());
    const [selectedSlug, setSelectedSlug] = useState<string | null>(null);
    const [search, setSearch] = useState("");
    const [showGrant, setShowGrant] = useState(false);
    const [showFlair, setShowFlair] = useState<ITierListBrowseItem | null>(null);
    const [showDelete, setShowDelete] = useState<ITierListBrowseItem | null>(null);

    const lists = browseQuery.data ?? [];
    const filtered = useMemo(
        () =>
            lists.filter((l) => {
                if (!search) return true;
                const q = search.toLowerCase();
                return l.slug.toLowerCase().includes(q) || l.title.toLowerCase().includes(q) || (l.author.name ?? "").toLowerCase().includes(q);
            }),
        [lists, search],
    );

    const selected = selectedSlug ? lists.find((l) => l.slug === selectedSlug) : filtered[0];

    return (
        <>
            <PageHead
                kicker="Manage"
                title="Tier Lists"
                sub={
                    <>
                        Every tier list on the platform. Pick one to manage its <strong>View → Edit → Publish → Admin</strong> permission ladder, or use the row menu for direct edits.
                    </>
                }
            />

            <div className="grid grid-cols-1 gap-4.5 lg:grid-cols-[420px_1fr]">
                <Card>
                    <CardHeader>
                        <CardTitle className="text-sm">Tier lists</CardTitle>
                        <CardDescription className="text-xs">
                            {lists.length} total · {lists.filter((l) => l.listType === "official").length} official
                        </CardDescription>
                    </CardHeader>
                    <div className="px-2 pb-2">
                        <InputGroup>
                            <InputGroupAddon>
                                <SearchIcon />
                            </InputGroupAddon>
                            <Input placeholder="Search slug or owner…" size="sm" value={search} onChange={(e) => setSearch(e.target.value)} />
                        </InputGroup>
                    </div>
                    <div className="max-h-170 overflow-auto overflow-x-hidden border-border border-t">
                        {browseQuery.isPending ? (
                            <div className="space-y-2 p-3.5">
                                <Skeleton className="h-14" />
                                <Skeleton className="h-14" />
                                <Skeleton className="h-14" />
                            </div>
                        ) : filtered.length === 0 ? (
                            <div className="px-3.5 py-8 text-center text-[12.5px] text-muted-foreground">No tier lists match.</div>
                        ) : (
                            filtered.map((tl) => <TierListRow key={tl.slug} tl={tl} active={selected?.slug === tl.slug} onSelect={() => setSelectedSlug(tl.slug)} onSetFlair={() => setShowFlair(tl)} onDelete={() => setShowDelete(tl)} />)
                        )}
                    </div>
                </Card>

                {selected ? (
                    <PermissionDetail key={selected.slug} list={selected} onGrant={() => setShowGrant(true)} authed={isAuthenticated} lookup={lookupQuery.data} />
                ) : (
                    <Card>
                        <CardContent className="p-12 text-center text-[13px] text-muted-foreground">Pick a tier list on the left.</CardContent>
                    </Card>
                )}
            </div>

            {showGrant && selected ? <GrantDialog slug={selected.slug} onClose={() => setShowGrant(false)} /> : null}
            {showFlair ? <FlairDialog list={showFlair} flairs={flairsQuery.data ?? []} onClose={() => setShowFlair(null)} /> : null}
            {showDelete ? (
                <DeleteDialog
                    list={showDelete}
                    onClose={() => setShowDelete(null)}
                    onDeleted={() => {
                        if (selectedSlug === showDelete.slug) setSelectedSlug(null);
                        setShowDelete(null);
                    }}
                />
            ) : null}
        </>
    );
}

function TierListRow({ tl, active, onSelect, onSetFlair, onDelete }: { tl: ITierListBrowseItem; active: boolean; onSelect: () => void; onSetFlair: () => void; onDelete: () => void }): React.ReactElement {
    const navigate = useNavigate();
    return (
        <div className={cn("group flex w-full min-w-0 items-center gap-2 border-0 border-border border-b p-3.5 last:border-0", active && "bg-[color-mix(in_srgb,var(--primary)_6%,var(--card))]")}>
            <button type="button" onClick={onSelect} className="flex min-w-0 flex-1 cursor-pointer items-center gap-2.5 text-left text-foreground">
                <span className="h-8 w-1.5 shrink-0 rounded-[3px]" style={{ background: active ? "var(--primary)" : "var(--border)" }} />
                <div className="min-w-0 flex-1 overflow-hidden">
                    <div className={cn("truncate font-mono font-semibold text-[13px]", active ? "text-primary" : "text-foreground")} title={`/${tl.slug}`}>
                        /{tl.slug}
                    </div>
                    <div className="mt-1 truncate text-[11.5px] text-muted-foreground">
                        {tl.author.name ?? "-"} · {tl.tiers.length} tier{tl.tiers.length === 1 ? "" : "s"}
                    </div>
                </div>
            </button>
            {tl.listType === "official" ? (
                <Badge variant="success" className="shrink-0">
                    official
                </Badge>
            ) : null}
            <DropdownMenu>
                <DropdownMenuTrigger
                    render={(triggerProps) => (
                        <button {...triggerProps} type="button" aria-label="Row actions" className="inline-flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-md text-foreground hover:bg-accent">
                            <MoreHorizontalIcon className="size-4 opacity-80" strokeWidth={1.9} />
                        </button>
                    )}
                />
                <DropdownMenuContent align="end" className="w-44">
                    <DropdownMenuItem className="cursor-pointer" onClick={() => window.open(`/tier-lists/${tl.slug}`, "_blank")}>
                        <ExternalLinkIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                        View public page
                    </DropdownMenuItem>
                    <DropdownMenuItem className="cursor-pointer" onClick={() => navigate({ to: "/tier-lists/my/$id/edit", params: { id: tl.slug } })}>
                        <EditIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                        Open editor
                    </DropdownMenuItem>
                    <DropdownMenuItem className="cursor-pointer" onClick={onSetFlair}>
                        <TagIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                        Change flair
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem className="cursor-pointer text-destructive-foreground focus:text-destructive-foreground" onClick={onDelete}>
                        <Trash2Icon className="mr-2 h-4 w-4" />
                        Delete tier list
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    );
}

function PermissionDetail({ list, onGrant, authed, lookup }: { list: ITierListBrowseItem; onGrant: () => void; authed: boolean; lookup: Map<string, IUserProfile> | undefined }): React.ReactElement {
    const queryClient = useQueryClient();
    const permsQuery = useQuery(tierListPermissionsQueryOptions(list.slug, authed));

    const updatePermission = useMutation({
        mutationFn: (input: { slug: string; userId: string; permission: TierListPermissionLevel }) => grantTierListPermissionFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "tier-lists", "permissions", list.slug] });
            toastManager.add({ id: `perm-grant-${Date.now()}`, title: "Permission updated", description: `Updated grant on /${list.slug}.`, type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `perm-grant-err-${Date.now()}`, title: "Failed to update permission", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const revoke = useMutation({
        mutationFn: (input: { slug: string; userId: string }) => revokeTierListPermissionFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "tier-lists", "permissions", list.slug] });
            toastManager.add({ id: `perm-revoke-${Date.now()}`, title: "Permission revoked", description: `Removed grant on /${list.slug}.`, type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `perm-revoke-err-${Date.now()}`, title: "Failed to revoke", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const perms = permsQuery.data ?? [];

    return (
        <Card>
            <CardHeader>
                <CardTitle className="text-sm">
                    <span className="font-mono">/{list.slug}</span>
                </CardTitle>
                <CardDescription className="text-xs">
                    {list.title} · owner {list.author.name ?? "-"} · {perms.length} grant{perms.length === 1 ? "" : "s"}
                </CardDescription>
                <CardAction>
                    <Button size="sm" onClick={onGrant}>
                        <PlusIcon />
                        Grant access
                    </Button>
                </CardAction>
            </CardHeader>
            {permsQuery.isError ? (
                <CardContent className="border-border border-t text-[13px] text-muted-foreground">
                    Couldn't load permissions for <span className="font-mono">/{list.slug}</span>.
                </CardContent>
            ) : permsQuery.isPending ? (
                <CardContent className="border-border border-t">
                    <Skeleton className="h-32 w-full" />
                </CardContent>
            ) : perms.length === 0 ? (
                <CardContent className="border-border border-t py-12 text-center text-[13px] text-muted-foreground">No grants yet - only the owner has access.</CardContent>
            ) : (
                <table className="w-full border-collapse border-border border-t text-[13px]">
                    <thead>
                        <tr>
                            {["Doctor", "Level", "Granted", "Granted by", ""].map((h) => (
                                <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                    {h}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {perms.map((p) => (
                            <tr key={`${p.userId}-${p.permission}`} className="border-border border-b last:border-0">
                                <td className="px-3.5 py-2.5">
                                    <PermUserCell userId={p.userId} lookup={lookup} />
                                </td>
                                <td className="px-3.5 py-2.5">
                                    <LevelDots level={p.permission} onChange={(level) => updatePermission.mutate({ slug: list.slug, userId: p.userId, permission: level })} />
                                </td>
                                <td className="px-3.5 py-2.5 text-muted-foreground">{new Date(p.grantedAt).toLocaleDateString()}</td>
                                <td className="px-3.5 py-2.5 text-muted-foreground">{p.grantedBy ? (lookup?.get(p.grantedBy)?.nickname ?? p.grantedBy.slice(0, 8)) : "-"}</td>
                                <td className="px-3.5 py-2.5">
                                    <Button variant="destructive-outline" size="xs" onClick={() => revoke.mutate({ slug: list.slug, userId: p.userId })} disabled={revoke.isPending}>
                                        <Trash2Icon />
                                        Revoke
                                    </Button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
            <div className="flex items-center justify-between border-border border-t px-3.5 py-3">
                <span className="text-[12px] text-muted-foreground">Owner has implicit Admin and cannot be revoked.</span>
            </div>
        </Card>
    );
}

function GrantDialog({ slug, onClose }: { slug: string; onClose: () => void }): React.ReactElement {
    const queryClient = useQueryClient();
    const [q, setQ] = useState("");
    const [selectedUserId, setSelectedUserId] = useState<string | null>(null);
    const [permission, setPermission] = useState<TierListPermissionLevel>("edit");
    const searchQuery = useQuery({ ...searchUsersQueryOptions({ q, limit: 8 }), enabled: q.length >= 2 });

    const grant = useMutation({
        mutationFn: (input: { slug: string; userId: string; permission: TierListPermissionLevel }) => grantTierListPermissionFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "tier-lists", "permissions", slug] });
            toastManager.add({ id: `perm-create-${Date.now()}`, title: "Grant created", description: `${levelLabel(permission)} access on /${slug}.`, type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `perm-create-err-${Date.now()}`, title: "Failed to grant", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    const results = searchQuery.data?.entries ?? [];

    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label="Close" />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center">
                <div className="pointer-events-auto w-120 max-w-[92vw] overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35),0_8px_18px_oklch(0_0_0/0.2)]">
                    <div className="border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">Manage · permissions</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">Grant access on /{slug}</div>
                    </div>
                    <div className="flex flex-col gap-4 p-5">
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">Find a Doctor</span>
                            <InputGroup>
                                <InputGroupAddon>
                                    <SearchIcon />
                                </InputGroupAddon>
                                <Input
                                    placeholder="Search by nickname…"
                                    size="sm"
                                    value={q}
                                    onChange={(e) => {
                                        setQ(e.target.value);
                                        setSelectedUserId(null);
                                    }}
                                />
                            </InputGroup>
                            {q.length >= 2 ? (
                                searchQuery.isPending ? (
                                    <Skeleton className="h-12 w-full" />
                                ) : results.length === 0 ? (
                                    <div className="text-[12.5px] text-muted-foreground">No matches.</div>
                                ) : (
                                    <ul className="max-h-48 overflow-auto rounded-lg border border-border">
                                        {results.map((r) => (
                                            <li key={r.id}>
                                                <button
                                                    type="button"
                                                    onClick={() => setSelectedUserId(r.id)}
                                                    className={cn("flex w-full items-center justify-between gap-2 border-0 border-border border-b px-3 py-2 text-left last:border-0 hover:bg-accent", selectedUserId === r.id && "bg-[color-mix(in_srgb,var(--primary)_6%,var(--card))]")}
                                                >
                                                    <span className="flex items-center gap-2">
                                                        <span className="relative inline-block size-6 overflow-hidden rounded-full bg-[linear-gradient(135deg,oklch(0.58_0.22_25),oklch(0.85_0.12_25))]">
                                                            <img src={getSecretaryAvatarURL({ secretary: r.secretary, secretary_skin_id: r.secretary_skin_id })} alt="" loading="lazy" className="absolute inset-0 size-full object-cover" onError={(e) => (e.currentTarget.style.display = "none")} />
                                                        </span>
                                                        <span>
                                                            <span className="font-medium">{r.nickname ?? "-"}</span>
                                                            <span className="ml-1.5 font-mono text-[11.5px] text-muted-foreground">UID {r.uid}</span>
                                                        </span>
                                                    </span>
                                                    <span className="font-mono text-[11px] text-muted-foreground">{r.server}</span>
                                                </button>
                                            </li>
                                        ))}
                                    </ul>
                                )
                            ) : null}
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">Level</span>
                            <LevelDots level={permission} onChange={setPermission} />
                        </div>
                    </div>
                    <div className="flex justify-end gap-2 border-border border-t p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button size="sm" disabled={!selectedUserId || grant.isPending} loading={grant.isPending} onClick={() => selectedUserId && grant.mutate({ slug, userId: selectedUserId, permission })}>
                            Grant {levelLabel(permission)}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}

function FlairDialog({ list, flairs, onClose }: { list: ITierListBrowseItem; flairs: { id: number; code: string; label: string; color: string | null }[]; onClose: () => void }): React.ReactElement {
    const queryClient = useQueryClient();
    const [selected, setSelected] = useState<number | null>(null);

    const setFlair = useMutation({
        mutationFn: (input: { slug: string; flairId: number | null }) => setTierListFlairFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({ id: `flair-${Date.now()}`, title: "Flair updated", description: `Flair updated on /${list.slug}.`, type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `flair-err-${Date.now()}`, title: "Failed to set flair", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label="Close" />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center">
                <div className="pointer-events-auto w-105 max-w-[92vw] overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35)]">
                    <div className="border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">Manage · flair</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">Change flair on /{list.slug}</div>
                    </div>
                    <div className="p-5">
                        <div className="flex flex-wrap items-center gap-2">
                            <button type="button" onClick={() => setSelected(null)} className={cn("inline-flex h-7 items-center rounded-md border border-border bg-card px-2 font-medium text-[12px]", selected === null && "ring-2 ring-ring")}>
                                <span className="mr-1.5 inline-block size-2.5 rounded-xs bg-muted-foreground/40" />
                                None
                            </button>
                            {flairs.map((f) => (
                                <button key={f.code} type="button" onClick={() => setSelected(f.id)} className={cn("inline-flex h-7 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-[12px]", selected === f.id && "ring-2 ring-ring")} style={{ color: f.color ?? undefined }}>
                                    <span className="size-2.5 rounded-xs" style={{ background: f.color ?? "var(--muted-foreground)" }} />
                                    {f.label}
                                </button>
                            ))}
                        </div>
                    </div>
                    <div className="flex justify-end gap-2 border-border border-t p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button size="sm" disabled={setFlair.isPending} loading={setFlair.isPending} onClick={() => setFlair.mutate({ slug: list.slug, flairId: selected })}>
                            Save
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}

function DeleteDialog({ list, onClose, onDeleted }: { list: ITierListBrowseItem; onClose: () => void; onDeleted: () => void }): React.ReactElement {
    const queryClient = useQueryClient();
    const del = useMutation({
        mutationFn: (slug: string) => deleteTierListFn({ data: slug }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({ id: `tl-del-${Date.now()}`, title: "Tier list deleted", description: `/${list.slug} is gone.`, type: "success" });
            onDeleted();
        },
        onError: (err: unknown) => toastManager.add({ id: `tl-del-err-${Date.now()}`, title: "Failed to delete", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });
    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label="Close" />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center">
                <div className="pointer-events-auto w-105 max-w-[92vw] overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35)]">
                    <div className="border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-destructive-foreground uppercase tracking-[0.22em]">Manage · delete</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">Delete /{list.slug}?</div>
                        <div className="mt-1 text-[12.5px] text-muted-foreground">
                            This is permanent. All tiers, placements, and version history for <strong>{list.title}</strong> are destroyed.
                        </div>
                    </div>
                    <div className="flex justify-end gap-2 p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            Cancel
                        </Button>
                        <Button variant="destructive" size="sm" disabled={del.isPending} loading={del.isPending} onClick={() => del.mutate(list.slug)}>
                            Delete tier list
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}
