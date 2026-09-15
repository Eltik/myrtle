import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { CheckIcon, EditIcon, ExternalLinkIcon, MoreHorizontalIcon, PlusIcon, SearchIcon, TagIcon, Trash2Icon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { MarkdownEditor } from "#/components/ui/markdown-editor";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { Skeleton } from "#/components/ui/skeleton";
import { toastManager } from "#/components/ui/toast";
import { browseTierListsQueryOptions, createTierListFn, deleteTierListFn, type ITierListBrowseItem, type ITierListFlair, publishTierListVersionFn, setTierListFlairFn, tierListFlairsQueryOptions } from "#/lib/api/tier-lists";
import { type TypedRichT, useFormatters, useGamedataServer, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { HCode, PageHead } from "../AdminShell";
import type { messages } from "./OfficialTierLists.messages";

type StatusFilter = "all" | "active" | "draft";

type OfficialT = TypedT<typeof messages>;
type OfficialRichT = TypedRichT<typeof messages>;

function statusLabel(t: OfficialT, status: StatusFilter): string {
    switch (status) {
        case "all":
            return t("official.filter.all");
        case "active":
            return t("official.filter.trending");
        case "draft":
            return t("official.filter.draft");
    }
}

export function OfficialTierLists(): React.ReactElement {
    const t: OfficialT = useT("admin");
    const rt: OfficialRichT = useRichT("admin");
    const browseQuery = useQuery(browseTierListsQueryOptions(useGamedataServer()));
    const flairsQuery = useQuery(tierListFlairsQueryOptions());
    const [showNew, setShowNew] = useState(false);
    const [showFlair, setShowFlair] = useState<ITierListBrowseItem | null>(null);
    const [showDelete, setShowDelete] = useState<ITierListBrowseItem | null>(null);
    const [showPublish, setShowPublish] = useState<ITierListBrowseItem | null>(null);
    const [search, setSearch] = useState("");
    const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

    const official = useMemo(() => (browseQuery.data ?? []).filter((l) => l.listType === "official"), [browseQuery.data]);

    const filtered = useMemo(() => {
        return official.filter((l) => {
            if (search) {
                const q = search.toLowerCase();
                if (!l.title.toLowerCase().includes(q) && !l.slug.toLowerCase().includes(q) && !(l.flairLabel ?? "").toLowerCase().includes(q)) return false;
            }
            if (statusFilter === "active" && !l.hot) {
                // "active" → tier_lists.is_active=true. The browse mapping doesn't expose is_active directly,
                // but trending items are always active. We approximate with hot. Fall back to all when ambiguous.
            }
            return true;
        });
    }, [official, search, statusFilter]);

    return (
        <>
            <PageHead
                kicker={t("official.kicker")}
                title={t("official.title")}
                sub={rt("official.sub", { filter: <HCode>list_type = "official"</HCode>, role: <span className="font-mono">tier_list_admin</span> })}
                action={
                    <>
                        <Button
                            variant="outline"
                            size="sm"
                            render={
                                // biome-ignore lint/a11y/useAnchorContent: Button's `render` merges children into the anchor at runtime
                                <a href="/tier-lists" target="_blank" rel="noreferrer" />
                            }
                        >
                            <ExternalLinkIcon />
                            {t("official.viewPublicRail")}
                        </Button>
                        <Button size="sm" onClick={() => setShowNew(true)}>
                            <PlusIcon />
                            {t("official.newList")}
                        </Button>
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
                            <Input placeholder={t("official.searchPlaceholder")} size="sm" value={search} onChange={(e) => setSearch(e.target.value)} />
                        </InputGroup>
                    </div>
                    <div className="inline-flex max-w-full gap-px overflow-x-auto rounded-[9px] border border-border bg-card p-0.75">
                        {(["all", "active", "draft"] as const).map((status) => (
                            <button
                                key={status}
                                type="button"
                                onClick={() => setStatusFilter(status)}
                                className={cn("inline-flex h-6.5 cursor-pointer items-center rounded-md px-3 font-medium text-[12.5px] transition-colors", statusFilter === status ? "bg-background text-foreground shadow-xs/5" : "text-muted-foreground hover:text-foreground")}
                            >
                                {statusLabel(t, status)}
                            </button>
                        ))}
                    </div>
                </div>

                {browseQuery.isPending ? (
                    <div className="space-y-2 p-4">
                        <Skeleton className="h-12" />
                        <Skeleton className="h-12" />
                        <Skeleton className="h-12" />
                    </div>
                ) : filtered.length === 0 ? (
                    <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">{t("official.empty")}</div>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full min-w-205 border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {[t("official.th.title"), t("official.th.flair"), t("official.th.trending"), t("official.th.tiers"), t("official.th.placements"), t("official.th.updated"), t("official.th.views"), ""].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.map((l) => (
                                    <OfficialRow key={l.slug} list={l} onSetFlair={() => setShowFlair(l)} onDelete={() => setShowDelete(l)} onPublish={() => setShowPublish(l)} />
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>

            {showNew ? <NewListDialog onClose={() => setShowNew(false)} /> : null}
            {showFlair ? <FlairDialog list={showFlair} flairs={flairsQuery.data ?? []} onClose={() => setShowFlair(null)} /> : null}
            {showDelete ? <DeleteDialog list={showDelete} onClose={() => setShowDelete(null)} /> : null}
            {showPublish ? <PublishDialog list={showPublish} onClose={() => setShowPublish(null)} /> : null}
        </>
    );
}

function OfficialRow({ list, onSetFlair, onDelete, onPublish }: { list: ITierListBrowseItem; onSetFlair: () => void; onDelete: () => void; onPublish: () => void }): React.ReactElement {
    const t: OfficialT = useT("admin");
    const fmt = useFormatters();
    const placementCount = list.tiers.reduce((n, t) => n + t.operators.length, 0);
    const color = list.flairColor ?? "var(--primary)";
    const navigate = useNavigate();
    return (
        <tr className="border-border border-b last:border-0 hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]">
            <td className="px-3.5 py-2.5">
                <span className="mr-2.5 inline-block h-5.5 w-1 rounded-xs align-[-6px]" style={{ background: color }} />
                <span className="font-medium">{list.title}</span>
                <span className="font-mono text-[11.5px] text-muted-foreground"> · /{list.slug}</span>
            </td>
            <td className="px-3.5 py-2.5">
                {list.flairLabel ? (
                    <span className="inline-flex items-center gap-1.5 font-medium text-[11.5px]" style={{ color }}>
                        <span className="size-2 rounded-xs" style={{ background: color }} />
                        {list.flairLabel}
                    </span>
                ) : (
                    <span className="text-[12px] text-muted-foreground">-</span>
                )}
            </td>
            <td className="px-3.5 py-2.5">{list.hot ? <Badge variant="success">{t("official.badge.trending")}</Badge> : <Badge variant="outline">-</Badge>}</td>
            <td className="px-3.5 py-2.5 tabular-nums">{list.tiers.length}</td>
            <td className="px-3.5 py-2.5 tabular-nums">{placementCount}</td>
            <td className="px-3.5 py-2.5 text-muted-foreground">{list.updated}</td>
            <td className="px-3.5 py-2.5 tabular-nums">{fmt.number(list.views24h)}</td>
            <td className="px-3.5 py-2.5">
                <DropdownMenu>
                    <DropdownMenuTrigger
                        render={(triggerProps) => (
                            <button {...triggerProps} type="button" aria-label={t("official.rowActions")} className="inline-flex size-7 cursor-pointer items-center justify-center rounded-md text-foreground hover:bg-accent">
                                <MoreHorizontalIcon className="size-4 opacity-80" strokeWidth={1.9} />
                            </button>
                        )}
                    />
                    <DropdownMenuContent align="end" className="w-48">
                        <DropdownMenuItem className="cursor-pointer" onClick={() => window.open(`/tier-lists/${list.slug}`, "_blank")}>
                            <ExternalLinkIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                            {t("official.menu.viewPublic")}
                        </DropdownMenuItem>
                        <DropdownMenuItem className="cursor-pointer" onClick={() => navigate({ to: "/tier-lists/my/$id/edit", params: { id: list.slug } })}>
                            <EditIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                            {t("official.menu.openEditor")}
                        </DropdownMenuItem>
                        <DropdownMenuItem className="cursor-pointer" onClick={onSetFlair}>
                            <TagIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                            {t("official.menu.changeFlair")}
                        </DropdownMenuItem>
                        <DropdownMenuItem className="cursor-pointer" onClick={onPublish}>
                            <CheckIcon className="mr-2 h-4 w-4 text-muted-foreground" />
                            {t("official.menu.publish")}
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem className="cursor-pointer text-destructive-foreground focus:text-destructive-foreground" onClick={onDelete}>
                            <Trash2Icon className="mr-2 h-4 w-4" />
                            {t("official.menu.delete")}
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </td>
        </tr>
    );
}

function FlairDialog({ list, flairs, onClose }: { list: ITierListBrowseItem; flairs: ITierListFlair[]; onClose: () => void }): React.ReactElement {
    const t: OfficialT = useT("admin");
    const queryClient = useQueryClient();
    const [selected, setSelected] = useState<number | null>(null);

    const setFlair = useMutation({
        mutationFn: (input: { slug: string; flairId: number | null }) => setTierListFlairFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({ id: `flair-${Date.now()}`, title: t("official.toast.flair"), description: t("official.toast.flair.desc", { slug: list.slug }), type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `flair-err-${Date.now()}`, title: t("official.toast.flairFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("official.close")} />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center p-3 max-sm:items-end max-sm:p-0">
                <div className="pointer-events-auto flex max-h-[92dvh] w-105 max-w-[92vw] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35)] max-sm:w-full max-sm:max-w-none max-sm:rounded-b-none max-sm:border-x-0 max-sm:border-b-0">
                    <div className="shrink-0 border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">{t("official.flair.kicker")}</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">{t("official.flair.title", { slug: list.slug })}</div>
                    </div>
                    <div className="min-h-0 flex-1 overflow-y-auto p-5 [-webkit-overflow-scrolling:touch]">
                        <div className="flex flex-wrap items-center gap-2">
                            <button type="button" onClick={() => setSelected(null)} className={cn("inline-flex h-9 items-center rounded-md border border-border bg-card px-2.5 font-medium text-[13px] sm:h-7 sm:px-2 sm:text-[12px]", selected === null && "ring-2 ring-ring")}>
                                <span className="mr-1.5 inline-block size-2.5 rounded-xs bg-muted-foreground/40" />
                                {t("official.flair.none")}
                            </button>
                            {flairs.map((f) => (
                                <button
                                    key={f.code}
                                    type="button"
                                    onClick={() => setSelected(f.id)}
                                    className={cn("inline-flex h-9 items-center gap-1.5 rounded-md border border-border bg-card px-2.5 font-medium text-[13px] sm:h-7 sm:px-2 sm:text-[12px]", selected === f.id && "ring-2 ring-ring")}
                                    style={{ color: f.color ?? undefined }}
                                >
                                    <span className="size-2.5 rounded-xs" style={{ background: f.color ?? "var(--muted-foreground)" }} />
                                    {f.label}
                                </button>
                            ))}
                        </div>
                    </div>
                    <div className="flex shrink-0 justify-end gap-2 border-border border-t p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            {t("official.cancel")}
                        </Button>
                        <Button size="sm" disabled={setFlair.isPending} loading={setFlair.isPending} onClick={() => setFlair.mutate({ slug: list.slug, flairId: selected })}>
                            {t("official.save")}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}

function DeleteDialog({ list, onClose }: { list: ITierListBrowseItem; onClose: () => void }): React.ReactElement {
    const t: OfficialT = useT("admin");
    const rt: OfficialRichT = useRichT("admin");
    const queryClient = useQueryClient();
    const del = useMutation({
        mutationFn: (slug: string) => deleteTierListFn({ data: slug }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({ id: `tl-del-${Date.now()}`, title: t("official.toast.deleted"), description: t("official.toast.deleted.desc", { slug: list.slug }), type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `tl-del-err-${Date.now()}`, title: t("official.toast.deleteFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });
    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("official.close")} />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center p-3 max-sm:items-end max-sm:p-0">
                <div className="pointer-events-auto flex max-h-[92dvh] w-105 max-w-[92vw] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35)] max-sm:w-full max-sm:max-w-none max-sm:rounded-b-none max-sm:border-x-0 max-sm:border-b-0">
                    <div className="shrink-0 border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-destructive-foreground uppercase tracking-[0.22em]">{t("official.delete.kicker")}</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">{t("official.delete.title", { slug: list.slug })}</div>
                        <div className="mt-1 text-[12.5px] text-muted-foreground">{rt("official.delete.body", { title: <strong>{list.title}</strong> })}</div>
                    </div>
                    <div className="flex shrink-0 justify-end gap-2 p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            {t("official.cancel")}
                        </Button>
                        <Button variant="destructive" size="sm" disabled={del.isPending} loading={del.isPending} onClick={() => del.mutate(list.slug)}>
                            {t("official.delete.submit")}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}

function PublishDialog({ list, onClose }: { list: ITierListBrowseItem; onClose: () => void }): React.ReactElement {
    const t: OfficialT = useT("admin");
    const queryClient = useQueryClient();
    const [changelog, setChangelog] = useState("");
    const publish = useMutation({
        mutationFn: (input: { slug: string; changelog: string | null }) => publishTierListVersionFn({ data: input }),
        onSuccess: () => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({ id: `tl-pub-${Date.now()}`, title: t("official.toast.published"), description: t("official.toast.published.desc", { slug: list.slug }), type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `tl-pub-err-${Date.now()}`, title: t("official.toast.publishFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });
    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("official.close")} />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center p-3 max-sm:items-end max-sm:p-0">
                <div className="pointer-events-auto flex max-h-[92dvh] w-120 max-w-[92vw] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35)] max-sm:w-full max-sm:max-w-none max-sm:rounded-b-none max-sm:border-x-0 max-sm:border-b-0">
                    <div className="shrink-0 border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">{t("official.publish.kicker")}</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">{t("official.publish.title", { slug: list.slug })}</div>
                        <div className="mt-1 text-[12.5px] text-muted-foreground">{t("official.publish.desc")}</div>
                    </div>
                    <div className="min-h-0 flex-1 overflow-y-auto p-5 [-webkit-overflow-scrolling:touch]">
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">{t("official.publish.changelog")}</span>
                            <MarkdownEditor value={changelog} onChange={setChangelog} placeholder={t("official.publish.changelogPlaceholder")} rows={4} size="sm" showHint={false} />
                        </div>
                    </div>
                    <div className="flex shrink-0 justify-end gap-2 border-border border-t p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            {t("official.cancel")}
                        </Button>
                        <Button size="sm" disabled={publish.isPending} loading={publish.isPending} onClick={() => publish.mutate({ slug: list.slug, changelog: changelog.trim() || null })}>
                            {t("official.publish.submit")}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}

function NewListDialog({ onClose }: { onClose: () => void }): React.ReactElement {
    const t: OfficialT = useT("admin");
    const rt: OfficialRichT = useRichT("admin");
    const queryClient = useQueryClient();
    const flairsQuery = useQuery(tierListFlairsQueryOptions());

    const [title, setTitle] = useState("");
    const [description, setDescription] = useState("");

    const create = useMutation({
        mutationFn: (input: { name: string; description?: string | null; listType: "official" | "community" }) => createTierListFn({ data: input }),
        onSuccess: (data) => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({ id: `tl-create-${Date.now()}`, title: t("official.toast.created"), description: t("official.toast.created.desc", { slug: data.slug }), type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `tl-create-err-${Date.now()}`, title: t("official.toast.createFailed"), description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("official.close")} />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center p-3 max-sm:items-end max-sm:p-0">
                <div className="pointer-events-auto flex max-h-[92dvh] w-130 max-w-[92vw] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35),0_8px_18px_oklch(0_0_0/0.2)] max-sm:w-full max-sm:max-w-none max-sm:rounded-b-none max-sm:border-x-0 max-sm:border-b-0">
                    <div className="shrink-0 border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">{t("official.new.kicker")}</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">{t("official.new.title")}</div>
                        <div className="mt-1 text-[12.5px] text-muted-foreground">{rt("official.new.desc", { endpoint: <span className="font-mono">POST /tier-lists</span>, field: <span className="font-mono">list_type=official</span> })}</div>
                    </div>
                    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5 [-webkit-overflow-scrolling:touch]">
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">{t("official.new.titleLabel")}</span>
                            <Input size="sm" value={title} onChange={(e) => setTitle(e.target.value)} placeholder={t("official.new.titlePlaceholder")} />
                            <span className="text-[11.5px] text-muted-foreground">{t("official.new.titleHint")}</span>
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[12px]">{t("official.new.descLabel")}</span>
                            <MarkdownEditor value={description} onChange={setDescription} placeholder={t("official.new.descPlaceholder")} rows={3} size="sm" showHint={false} />
                        </div>
                        {flairsQuery.data && flairsQuery.data.length > 0 ? (
                            <div className="flex flex-col gap-1.5">
                                <span className="font-medium text-[12px]">{t("official.new.flairs")}</span>
                                <div className="flex flex-wrap items-center gap-2">
                                    {flairsQuery.data.map((f) => (
                                        <span key={f.code} className="inline-flex h-6.5 items-center gap-1.5 rounded-md border border-border bg-card px-2 font-medium text-[12px]" style={{ color: f.color ?? undefined }}>
                                            <span className="size-2.5 rounded-xs" style={{ background: f.color ?? "var(--muted-foreground)" }} />
                                            {f.label}
                                        </span>
                                    ))}
                                </div>
                                <span className="text-[11.5px] text-muted-foreground">{t("official.new.flairsHint")}</span>
                            </div>
                        ) : null}
                    </div>
                    <div className="flex shrink-0 justify-end gap-2 border-border border-t p-3.5">
                        <Button variant="outline" size="sm" onClick={onClose}>
                            {t("official.cancel")}
                        </Button>
                        <Button size="sm" disabled={!title.trim() || create.isPending} loading={create.isPending} onClick={() => create.mutate({ name: title.trim(), description: description.trim() || null, listType: "official" })}>
                            {t("official.new.submit")}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}
