import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { PlusIcon, SparklesIcon } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { useDebounce } from "#/hooks/use-debounce";
import { APIError } from "#/lib/api/_shared";
import { createTierListFn, deleteTierListFn, type ITierListBrowseItem, myTierListsDetailedQueryOptions, updateTierListFn } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { matchesBrowseQuery, sortBrowseItems } from "../shared";
import { CreateListDialog } from "./CreateListDialog";
import { DeleteListDialog, type IDeleteListTarget } from "./DeleteListDialog";
import { EditListDialog, type IEditListInitial } from "./EditListDialog";
import { MyHero } from "./MyHero";
import { MyListCard } from "./MyListCard";
import { MyListRow } from "./MyListRow";
import type { messages } from "./MyTierLists.messages";
import { type MyListSort, type MyListTypeFilter, MyToolbar, type MyViewMode } from "./MyToolbar";

const COMMUNITY_QUOTA = 10;

function buildShareURL(slug: string): string {
    if (typeof window === "undefined") return `/tier-lists/${slug}`;
    return `${window.location.origin}/tier-lists/${slug}`;
}

interface IMyTierListsProps {
    initialSort: MyListSort;
    initialType: MyListTypeFilter;
    initialView: MyViewMode;
    initialQuery: string;
    onPersistSearch: (next: { sort: MyListSort; type: MyListTypeFilter; view: MyViewMode; q: string }) => void;
}

export function MyTierLists({ initialSort, initialType, initialView, initialQuery, onPersistSearch }: IMyTierListsProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const { user } = useAuth();
    const authed = Boolean(user);

    const gamedataServer = useGamedataServer();
    const { data, isError, refetch, isFetching } = useQuery(myTierListsDetailedQueryOptions(authed, gamedataServer));
    const allLists = useMemo<ITierListBrowseItem[]>(() => data ?? [], [data]);

    const [sort, setSort] = useState<MyListSort>(initialSort);
    const [type, setType] = useState<MyListTypeFilter>(initialType);
    const [view, setView] = useState<MyViewMode>(initialView);
    const [inputQuery, setInputQuery] = useState(initialQuery);
    const debouncedQuery = useDebounce(inputQuery.trim(), 200);

    useEffect(() => {
        onPersistSearch({ sort, type, view, q: debouncedQuery });
    }, [sort, type, view, debouncedQuery, onPersistSearch]);

    const [isCreateOpen, setIsCreateOpen] = useState(false);
    const [editing, setEditing] = useState<IEditListInitial | null>(null);
    const [deleting, setDeleting] = useState<IDeleteListTarget | null>(null);
    const [mutationError, setMutationError] = useState<string | null>(null);

    const hasOfficial = useMemo(() => allLists.some((l) => l.listType === "official"), [allLists]);
    const officialCount = useMemo(() => allLists.filter((l) => l.listType === "official").length, [allLists]);
    const communityCount = useMemo(() => allLists.filter((l) => l.listType === "community").length, [allLists]);
    const totalViews = useMemo(() => allLists.reduce((sum, l) => sum + l.views, 0), [allLists]);
    const totalFavorites = useMemo(() => allLists.reduce((sum, l) => sum + l.favorites, 0), [allLists]);

    const filtered = useMemo(
        () =>
            allLists.filter((list) => {
                if (type !== "all" && list.listType !== type) return false;
                if (!matchesBrowseQuery(list, debouncedQuery)) return false;
                return true;
            }),
        [allLists, type, debouncedQuery],
    );

    const sorted = useMemo(() => sortBrowseItems(filtered, sort), [filtered, sort]);

    const invalidateQueries = useCallback(() => {
        queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
    }, [queryClient]);

    const createMutation = useMutation({
        mutationFn: (input: { name: string; description: string }) => createTierListFn({ data: { name: input.name, description: input.description || null, listType: "community" } }),
        onSuccess: (created) => {
            setIsCreateOpen(false);
            setMutationError(null);
            invalidateQueries();
            toastManager.add({
                id: `tl-create-${Date.now()}`,
                title: t("my.toast.createdTitle"),
                description: t("my.toast.createdBody", { name: created.name }),
                type: "success",
            });
            navigate({ to: "/tier-lists/$id", params: { id: created.slug } });
        },
        onError: (err: unknown) => {
            const message = err instanceof APIError ? err.message : err instanceof Error ? err.message : t("my.toast.createFailed");
            setMutationError(message);
        },
    });

    const updateMutation = useMutation({
        mutationFn: (input: { slug: string; name: string; description: string }) => updateTierListFn({ data: input }),
        onSuccess: () => {
            setEditing(null);
            setMutationError(null);
            invalidateQueries();
            toastManager.add({
                id: `tl-update-${Date.now()}`,
                title: t("my.toast.updatedTitle"),
                description: t("my.toast.updatedBody"),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = err instanceof APIError ? err.message : err instanceof Error ? err.message : t("my.toast.updateFailed");
            setMutationError(message);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (slug: string) => deleteTierListFn({ data: slug }),
        onSuccess: (_data, slug) => {
            const removedName = deleting?.name ?? t("my.toast.deletedFallbackName");
            setDeleting(null);
            setMutationError(null);
            queryClient.setQueryData<ITierListBrowseItem[] | undefined>(myTierListsDetailedQueryOptions(authed, gamedataServer).queryKey, (prev) => (prev ? prev.filter((l) => l.slug !== slug) : prev));
            invalidateQueries();
            toastManager.add({
                id: `tl-delete-${Date.now()}`,
                title: t("my.toast.deletedTitle"),
                description: t("my.toast.deletedBody", { name: removedName }),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = err instanceof APIError ? err.message : err instanceof Error ? err.message : t("my.toast.deleteFailed");
            setMutationError(message);
        },
    });

    const handleEdit = useCallback(
        (slug: string) => {
            const tl = allLists.find((l) => l.slug === slug);
            if (!tl) return;
            setMutationError(null);
            setEditing({ slug: tl.slug, name: tl.title, description: tl.description });
        },
        [allLists],
    );

    const handleDelete = useCallback(
        (slug: string) => {
            const tl = allLists.find((l) => l.slug === slug);
            if (!tl) return;
            setMutationError(null);
            setDeleting({ slug: tl.slug, name: tl.title });
        },
        [allLists],
    );

    const handleCopyLink = useCallback(
        async (slug: string) => {
            const url = buildShareURL(slug);
            try {
                await navigator.clipboard.writeText(url);
                toastManager.add({
                    id: `tl-copy-${Date.now()}`,
                    title: t("my.toast.copiedTitle"),
                    description: t("my.toast.copiedBody"),
                    type: "success",
                });
            } catch {
                toastManager.add({
                    id: `tl-copy-err-${Date.now()}`,
                    title: t("my.toast.copyFailedTitle"),
                    description: t("my.toast.copyFailedBody"),
                    type: "error",
                });
            }
        },
        [t],
    );

    const handleOpenCreate = useCallback(() => {
        setMutationError(null);
        setIsCreateOpen(true);
    }, []);

    return (
        <main className="min-h-dvh pb-24">
            <MyHero total={allLists.length} communityCount={communityCount} communityQuota={COMMUNITY_QUOTA} officialCount={officialCount} totalViews={totalViews} totalFavorites={totalFavorites} onCreate={handleOpenCreate} />

            <div id="my-tier-lists-grid" className="page-gutter mt-2 scroll-mt-32 [--page-max:1080px]">
                <MyToolbar sort={sort} type={type} view={view} query={inputQuery} resultCount={sorted.length} totalCount={allLists.length} hasOfficial={hasOfficial} onSortChange={setSort} onTypeChange={setType} onViewChange={setView} onQueryChange={setInputQuery} />

                <div className="mt-5">
                    {isError ? (
                        <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-10 text-center">
                            <p className="m-0 font-sans text-muted-foreground text-sm">{t("my.error")}</p>
                            <Button variant="outline" size="sm" className="mt-3" loading={isFetching} onClick={() => refetch()}>
                                {t("my.retry")}
                            </Button>
                        </div>
                    ) : allLists.length === 0 ? (
                        <Empty className="rounded-lg border border-border border-dashed bg-muted/10">
                            <EmptyHeader>
                                <EmptyMedia variant="icon">
                                    <SparklesIcon />
                                </EmptyMedia>
                                <EmptyTitle>{t("my.empty.title")}</EmptyTitle>
                                <EmptyDescription>{t("my.empty.body")}</EmptyDescription>
                            </EmptyHeader>
                            <EmptyContent>
                                <Button onClick={handleOpenCreate}>
                                    <PlusIcon />
                                    {t("my.empty.action")}
                                </Button>
                            </EmptyContent>
                        </Empty>
                    ) : sorted.length === 0 ? (
                        <div className="rounded-lg border border-border border-dashed bg-muted/20 px-5 py-12 text-center">
                            <p className="m-0 font-medium font-sans text-foreground text-sm">{t("my.filtered.emptyTitle")}</p>
                            <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">{t("my.filtered.emptyBody")}</p>
                            <Button
                                variant="outline"
                                size="sm"
                                className="mt-4"
                                onClick={() => {
                                    setInputQuery("");
                                    setType("all");
                                }}
                            >
                                {t("my.filtered.clear")}
                            </Button>
                        </div>
                    ) : view === "grid" ? (
                        <div className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
                            {sorted.map((tl) => (
                                <MyListCard key={tl.id} tl={tl} onEdit={handleEdit} onDelete={handleDelete} onCopyLink={handleCopyLink} />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col gap-2">
                            {sorted.map((tl) => (
                                <MyListRow key={tl.id} tl={tl} onEdit={handleEdit} onDelete={handleDelete} onCopyLink={handleCopyLink} />
                            ))}
                        </div>
                    )}
                </div>
            </div>

            <CreateListDialog
                open={isCreateOpen}
                onOpenChange={(next) => {
                    setIsCreateOpen(next);
                    if (!next) setMutationError(null);
                }}
                onSubmit={(input) => createMutation.mutate(input)}
                isSubmitting={createMutation.isPending}
                errorMessage={mutationError}
            />

            <EditListDialog
                initial={editing}
                onOpenChange={(next) => {
                    if (!next) {
                        setEditing(null);
                        setMutationError(null);
                    }
                }}
                onSubmit={(input) => updateMutation.mutate(input)}
                isSubmitting={updateMutation.isPending}
                errorMessage={mutationError}
            />

            <DeleteListDialog
                target={deleting}
                onOpenChange={(next) => {
                    if (!next) {
                        setDeleting(null);
                        setMutationError(null);
                    }
                }}
                onConfirm={(slug) => deleteMutation.mutate(slug)}
                isSubmitting={deleteMutation.isPending}
                errorMessage={mutationError}
            />
        </main>
    );
}
