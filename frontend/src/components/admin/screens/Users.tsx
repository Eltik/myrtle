import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CheckIcon, ExternalLinkIcon, RefreshCwIcon, SearchIcon, UserCogIcon, XIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { Skeleton } from "#/components/ui/skeleton";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type ISetUserRoleInput, isSuperAdmin, setUserRoleFn, type UserRole } from "#/lib/api/admin";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { type TypedRichT, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getSecretaryAvatarURL } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { HCode, PageHead } from "../AdminShell";
import { RoleBadge, SERVER_TINT } from "../Primitives";
import type { messages } from "./Users.messages";

type UsersT = TypedT<typeof messages>;
type UsersRichT = TypedRichT<typeof messages>;

const ASSIGNABLE_ROLES: readonly UserRole[] = ["user", "tier_list_editor", "tier_list_admin", "translator", "super_admin"] as const;

/** True when the event started inside the role cell, which owns its own menu. */
function fromRoleCell(e: React.SyntheticEvent): boolean {
    return e.target instanceof Element && e.target.closest("[data-role-cell]") !== null;
}

/**
 * The backend refuses a self-role-change with a 400 rather than letting a
 * super-admin demote themselves out of the panel, so the message is worth
 * rewriting into something actionable.
 */
function roleErrorMessage(err: unknown, t: UsersT): string {
    const raw = err instanceof Error ? err.message : String(err);
    if (raw.toLowerCase().includes("your own role")) return t("users.roleError.self");
    return raw;
}

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
    const t: UsersT = useT("admin");
    const rt: UsersRichT = useRichT("admin");
    const locale = useLocale();
    const { user: signedIn } = useAuth();
    const [server, setServer] = useState<Server>("all");
    const [query, setQuery] = useState("");
    const [opened, setOpened] = useState<IUserProfile | null>(null);

    const canAssignRoles = isSuperAdmin(signedIn?.role);

    // Use /search as the canonical list (when q="" it returns all public profiles
    // ordered by score). Filter client-side by server.
    const usersQuery = useQuery(searchUsersQueryOptions({ q: query || undefined, limit: 200 }));

    const entries = usersQuery.data?.entries ?? [];
    const filtered = useMemo(() => (server === "all" ? entries : entries.filter((u) => u.server.toUpperCase() === server)), [entries, server]);

    return (
        <>
            <PageHead
                kicker={t("users.kicker")}
                title={t("users.title")}
                sub={rt("users.sub", {
                    search: <HCode>/search</HCode>,
                    table: <span className="font-mono">users</span>,
                    endpoint: <HCode>PUT /admin/users/&#123;id&#125;/role</HCode>,
                })}
                action={
                    <Button variant="outline" size="sm" loading={usersQuery.isFetching} onClick={() => usersQuery.refetch()}>
                        <RefreshCwIcon />
                        {t("users.refresh")}
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
                            <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("users.searchPlaceholder")} size="sm" />
                        </InputGroup>
                    </div>
                    <SegTabs<Server>
                        value={server}
                        onChange={setServer}
                        items={[
                            { value: "all", label: t("users.allServers") },
                            { value: "EN", label: "EN" },
                            { value: "JP", label: "JP" },
                            { value: "KR", label: "KR" },
                            { value: "CN", label: "CN" },
                        ]}
                    />
                    <div className="flex-1" />
                    <span className="text-[12px] text-muted-foreground">{t("users.countShown", { filtered: filtered.length, total: entries.length })}</span>
                </div>
                {usersQuery.isPending ? (
                    <div className="space-y-2 p-4">
                        <Skeleton className="h-12" />
                        <Skeleton className="h-12" />
                        <Skeleton className="h-12" />
                    </div>
                ) : filtered.length === 0 ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">{server !== "all" ? t("users.emptyServer", { server }) : t("users.emptyAll")}</div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full min-w-160 border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {[t("users.th.doctor"), t("users.th.server"), t("users.th.level"), t("users.th.score"), t("users.th.grade"), t("users.th.role"), ""].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.map((u) => (
                                    <tr key={u.uid} onClick={(e) => !fromRoleCell(e) && setOpened(u)} onKeyDown={(e) => !fromRoleCell(e) && (e.key === "Enter" || e.key === " ") && setOpened(u)} className="cursor-pointer border-border border-b last:border-0 hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]">
                                        <td className="px-3.5 py-2.5">
                                            <span className="mr-2 inline-block align-[-6px]">
                                                <UserAvatar user={u} />
                                            </span>
                                            <span className="font-medium">{u.nickname ?? "-"}</span>
                                            <span className="font-medium font-mono text-[11.5px] text-muted-foreground">
                                                {" "}
                                                · {t("users.uid")}&nbsp;{u.uid}
                                            </span>
                                        </td>
                                        <td className="px-3.5 py-2.5 font-mono">{u.server.toUpperCase()}</td>
                                        <td className="px-3.5 py-2.5 tabular-nums">{u.level ?? "-"}</td>
                                        <td className="px-3.5 py-2.5 tabular-nums">{u.total_score?.toLocaleString(locale, { maximumFractionDigits: 2 }) ?? "-"}</td>
                                        <td className="px-3.5 py-2.5">{u.grade ? <Badge variant="outline">{u.grade}</Badge> : <span className="text-muted-foreground">-</span>}</td>
                                        <td className="px-3.5 py-2.5" data-role-cell="">
                                            <RoleControl user={u} canAssign={canAssignRoles} isSelf={signedIn?.id === u.id} />
                                        </td>
                                        <td className="px-3.5 py-2.5">
                                            <Button variant="ghost" size="xs" render={<Link to="/user/$id" params={{ id: u.uid }} target="_blank" onClick={(e) => e.stopPropagation()} />}>
                                                <ExternalLinkIcon />
                                                {t("users.open")}
                                            </Button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {opened ? <UserDetailDrawer user={opened} canAssign={canAssignRoles} isSelf={signedIn?.id === opened.id} onClose={() => setOpened(null)} /> : null}
        </>
    );
}

function UserDetailDrawer({ user, canAssign, isSelf, onClose }: { user: IUserProfile; canAssign: boolean; isSelf: boolean; onClose: () => void }): React.ReactElement {
    const t: UsersT = useT("admin");
    const rt: UsersRichT = useRichT("admin");
    return (
        <>
            <button type="button" className="fixed inset-0 z-50 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("users.closeDrawer")} />
            <aside className="fixed top-0 right-0 bottom-0 z-51 flex w-110 max-w-[92vw] flex-col border-border border-l bg-background shadow-[-20px_0_60px_oklch(0_0_0/0.18)]">
                <div className="flex items-center justify-between gap-2 border-border border-b px-4.5 py-3.5">
                    <div className="flex items-center gap-2.5">
                        <UserAvatar user={user} size={34} />
                        <div className="flex flex-col gap-0.5">
                            <span className="font-semibold text-[15px] leading-tight">{user.nickname ?? "-"}</span>
                            <span className="font-mono text-[11.5px] text-muted-foreground">
                                {t("users.uid")}&nbsp;{user.uid} · {user.server.toUpperCase()}
                                {user.level ? t("users.level", { level: user.level }) : ""}
                            </span>
                        </div>
                    </div>
                    <Button variant="ghost" size="icon-sm" onClick={onClose} aria-label={t("users.close")}>
                        <XIcon />
                    </Button>
                </div>

                <div className="flex-1 overflow-auto p-4.5">
                    <div className="mb-4 flex flex-wrap items-center gap-2">
                        {user.grade ? <Badge variant="outline">{t("users.gradeBadge", { grade: user.grade })}</Badge> : null}
                        <RoleBadge role={user.role} />
                        {user.public_profile ? <Badge variant="success">{t("users.public")}</Badge> : <Badge variant="warning">{t("users.private")}</Badge>}
                    </div>

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("users.profile.title")}</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <ScoreLine label={t("users.score.total")} value={user.total_score} />
                            <ScoreLine label={t("users.score.operators")} value={user.operator_count} />
                            <ScoreLine label={t("users.score.items")} value={user.item_count} />
                            <ScoreLine label={t("users.score.skins")} value={user.skin_count} />
                            {/* "LMD", "Orundum" and "Sanity" are the game's own names for these resources and stay as game vocabulary. */}
                            <ScoreLine label="LMD" value={user.lmd} />
                            <ScoreLine label="Orundum" value={user.orundum} />
                            <ScoreLine label="Sanity" value={user.sanity != null && user.max_sanity != null ? `${user.sanity} / ${user.max_sanity}` : "-"} last />
                        </CardContent>
                    </Card>

                    <div className="h-3.5" />

                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm">{t("users.actions.title")}</CardTitle>
                        </CardHeader>
                        <CardContent className="pt-0">
                            <Button variant="outline" size="sm" className="w-full" render={<Link to="/user/$id" params={{ id: user.uid }} target="_blank" />}>
                                <ExternalLinkIcon />
                                {t("users.openPublicProfile")}
                            </Button>
                            {canAssign ? (
                                <div className="mt-3 flex flex-col gap-1.5">
                                    <span className="font-medium text-[12px]">{t("users.globalRole")}</span>
                                    <RoleControl user={user} canAssign={canAssign} isSelf={isSelf} />
                                    <span className="text-[11px] text-muted-foreground">{t("users.globalRole.hint")}</span>
                                </div>
                            ) : null}
                            <div className="mt-3 text-[11.5px] text-muted-foreground">{rt("users.resync", { endpoint: <span className="font-mono">/auth/update-settings</span> })}</div>
                        </CardContent>
                    </Card>
                </div>
            </aside>
        </>
    );
}

/**
 * Role display, with assignment folded in for a super-admin. Your own row is
 * locked here rather than left to the backend's 400: a disabled control reads
 * better than a rejected write.
 */
function RoleControl({ user, canAssign, isSelf }: { user: IUserProfile; canAssign: boolean; isSelf: boolean }): React.ReactElement {
    const t: UsersT = useT("admin");
    const queryClient = useQueryClient();

    const setRole = useMutation({
        mutationFn: (input: ISetUserRoleInput) => setUserRoleFn({ data: input }),
        onSuccess: (_data, input) => {
            void queryClient.invalidateQueries({ queryKey: ["user", "search"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "stats"] });
            toastManager.add({ id: `role-set-${Date.now()}`, title: t("users.toast.roleUpdated"), description: t("users.toast.roleUpdated.desc", { name: user.nickname ?? user.uid, role: input.role }), type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `role-set-err-${Date.now()}`, title: t("users.toast.roleFailed"), description: roleErrorMessage(err, t), type: "error" }),
    });

    if (!canAssign) return <RoleBadge role={user.role} />;

    if (isSelf) {
        return (
            <span className="inline-flex items-center gap-1.5">
                <RoleBadge role={user.role} />
                <span className="text-[11px] text-muted-foreground">{t("users.you")}</span>
            </span>
        );
    }

    return (
        <DropdownMenu>
            <DropdownMenuTrigger
                render={(triggerProps) => (
                    <button {...triggerProps} type="button" disabled={setRole.isPending} className="inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-card px-1.5 hover:bg-accent disabled:cursor-default disabled:opacity-64">
                        <RoleBadge role={user.role} />
                        <UserCogIcon className="size-3.5 opacity-70" strokeWidth={1.9} />
                    </button>
                )}
            />
            <DropdownMenuContent align="start" className="w-52">
                {ASSIGNABLE_ROLES.map((r) => (
                    <DropdownMenuItem key={r} className="cursor-pointer" disabled={r === user.role} onClick={() => setRole.mutate({ userId: user.id, role: r })}>
                        {user.role === r ? <CheckIcon className="mr-2 h-4 w-4 text-primary" /> : <span className="mr-2 inline-block h-4 w-4" />}
                        <span className="font-mono text-[12px]">{r}</span>
                    </DropdownMenuItem>
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

function ScoreLine({ label, value, last }: { label: string; value: number | string | null | undefined; last?: boolean }): React.ReactElement {
    const locale = useLocale();
    return (
        <div className={cn("flex items-center justify-between py-1.5", !last && "border-border border-b")}>
            <span className="text-[12.5px] text-muted-foreground">{label}</span>
            <span className="font-mono text-[12.5px] tabular-nums">{typeof value === "number" ? value.toLocaleString(locale, { maximumFractionDigits: 2 }) : (value ?? "-")}</span>
        </div>
    );
}
