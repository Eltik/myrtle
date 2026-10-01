import { type UseQueryResult, useMutation, useQueries, useQuery, useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useCallback, useMemo, useReducer, useState } from "react";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type ITierEntity, parseEntityKey, type TierEntityKind, toTierEntity } from "#/lib/api/tier-entities";
import { type ITierListDetail, publishTierListVersionFn, setTierListFlairFn, setTierListVisibilityFn, tierEntityCatalogueQueryOptions, tierListDetailQueryOptions, tierListFlairsQueryOptions, tierListVersionsQueryOptions } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { EntitySummary } from "#/types/generated/EntitySummary";
import { DragControllerProvider } from "./drag-controller";
import { EditHero } from "./EditHero";
import styles from "./Editor.module.css";
import { EditTierRow } from "./EditTierRow";
import { EntityPool, type IKindCatalogue } from "./EntityPool";
import { PickTierDialog } from "./PickTierDialog";
import { PoolKindsDialog } from "./PoolKindsDialog";
import { PublishingPanel } from "./PublishingPanel";
import { PublishVersionDialog } from "./PublishVersionDialog";
import { type ISaveProgress, saveEdits } from "./save";
import type { messages as saveMessages } from "./save.messages";
import { detailToEditState, diffStates, editReducer, type IEditState, type IEditTier, type IPendingChange, nextFallbackTierColor, offeredKinds, placedEntity, placedEntityKeys } from "./state";
import type { messages as stateMessages } from "./state.messages";
import type { messages } from "./TierListEditor.messages";
import { TierSettingsDialog } from "./TierSettingsDialog";

/** This screen renders its own chrome plus the labels `state.ts` and `save.ts` derive. */
type EditorT = TypedT<typeof messages & typeof stateMessages & typeof saveMessages>;

interface ITierListEditorProps {
    slug: string;
}

export function TierListEditor({ slug }: ITierListEditorProps) {
    const { user } = useAuth();
    const queryClient = useQueryClient();

    const gamedataServer = useGamedataServer();
    const { data: detail } = useSuspenseQuery(tierListDetailQueryOptions(slug, gamedataServer));

    if (!detail) return <EditorMissing />;

    const isOwner = Boolean(user && detail.author?.id === user.id);
    const isTierListAdmin = user?.role === "tier_list_admin" || user?.role === "super_admin";
    const isTierListEditor = user?.role === "tier_list_editor";
    const canEditOfficial = detail.listType === "official" && isTierListEditor;
    const canEdit = isOwner || isTierListAdmin || canEditOfficial;
    if (!canEdit) return <EditorForbidden slug={slug} />;

    return <EditorContent slug={slug} detail={detail} queryClient={queryClient} />;
}

interface IEditorContentProps {
    slug: string;
    detail: ITierListDetail;
    queryClient: ReturnType<typeof useQueryClient>;
}

/** A catalogue entry is offered, not placed: no order, note or edit time of its own yet. */
const UNPLACED = { subOrder: 0, description: null, updatedAt: new Date(0).toISOString() } as const;

function EditorContent({ slug, detail, queryClient }: IEditorContentProps) {
    const t: EditorT = useT("tierLists");
    const describeError = useErrorMessage();
    // Same server the parent read the detail with, so the optimistic
    // `setQueryData` below targets the entry the page is actually rendering.
    const gamedataServer = useGamedataServer();
    const initial = useMemo(() => detailToEditState(detail), [detail]);
    const [originalState, setOriginalState] = useState<IEditState>(initial);
    const [state, dispatch] = useReducer(editReducer, initial);
    const [editingTier, setEditingTier] = useState<IEditTier | null>(null);
    const [picker, setPicker] = useState<ITierEntity | null>(null);
    const [saveError, setSaveError] = useState<string | null>(null);
    const [saveProgress, setSaveProgress] = useState<ISaveProgress | null>(null);
    const [publishOpen, setPublishOpen] = useState(false);
    const [publishError, setPublishError] = useState<string | null>(null);
    const [kindsOpen, setKindsOpen] = useState(false);

    const { data: flairCatalog } = useQuery(tierListFlairsQueryOptions());
    const { data: versions } = useQuery(tierListVersionsQueryOptions(slug, gamedataServer));
    // The kinds the pool offers follow the EDITED state, so a kind ticked in the
    // kinds dialog gets its tab before the list is saved. Not suspended: a newly
    // ticked kind loads inside its own tab instead of blanking the editor.
    const kinds = useMemo(() => offeredKinds(state), [state]);
    const combineCatalogues = useCallback(
        (results: UseQueryResult<EntitySummary[]>[]) => {
            const out: Partial<Record<TierEntityKind, IKindCatalogue>> = {};
            kinds.forEach((kind, i) => {
                const result = results[i];
                out[kind] = {
                    entities: result?.data?.map((summary) => toTierEntity(summary.kind, summary.id, summary, UNPLACED)),
                    status: result?.status ?? "pending",
                    refetch: () => void result?.refetch(),
                };
            });
            return out;
        },
        [kinds],
    );
    const catalogues = useQueries({
        queries: kinds.map((kind) => tierEntityCatalogueQueryOptions(kind, gamedataServer)),
        combine: combineCatalogues,
    });
    const flairOptions = useMemo(() => (flairCatalog ?? []).filter((f) => f.isActive), [flairCatalog]);
    const latestVersion = versions && versions.length > 0 ? Math.max(...versions.map((v) => v.version)) : null;
    const nextVersion = (latestVersion ?? 0) + 1;

    const entityByKey = useMemo(() => {
        const merged = { ...state.entityByKey };
        for (const catalogue of Object.values(catalogues)) {
            for (const entity of catalogue?.entities ?? []) {
                if (merged[entity.key]) continue;
                merged[entity.key] = entity;
            }
        }
        return merged;
    }, [state.entityByKey, catalogues]);

    const placedKeys = useMemo(() => placedEntityKeys(state), [state]);
    const placedByKind = useMemo(() => {
        const counts: Partial<Record<TierEntityKind, number>> = {};
        for (const key of placedKeys) {
            const { kind } = parseEntityKey(key);
            counts[kind] = (counts[kind] ?? 0) + 1;
        }
        return counts;
    }, [placedKeys]);
    const notedKeys = useMemo(() => {
        const set = new Set<string>();
        for (const [id, desc] of Object.entries(state.descriptionByKey)) {
            if (desc.trim()) set.add(id);
        }
        return set;
    }, [state.descriptionByKey]);
    const pendingChanges: IPendingChange[] = useMemo(() => diffStates(originalState, state, t), [originalState, state, t]);
    const findCurrentTierId = useCallback((key: string): string | null => state.tiers.find((t) => t.entityKeys.includes(key))?.id ?? null, [state.tiers]);

    const handlePlace = useCallback(
        (key: string, tierId: string, index: number) => {
            dispatch({ type: "PLACE_ENTITY", key, tierId, index, entity: entityByKey[key] });
        },
        [entityByKey],
    );

    const handleUnplace = useCallback((key: string) => {
        dispatch({ type: "PLACE_ENTITY", key, tierId: null });
    }, []);

    const handleAddTier = useCallback(() => {
        dispatch({ type: "ADD_TIER", name: defaultTierName(state.tiers.length), color: nextFallbackTierColor(state.tiers.length), description: "" });
    }, [state.tiers.length]);

    const handleActivateEntity = useCallback((entity: ITierEntity) => {
        setPicker(entity);
    }, []);

    const handleEntityDescriptionChange = useCallback(
        (description: string) => {
            if (!picker) return;
            dispatch({ type: "SET_ENTITY_DESCRIPTION", key: picker.key, description });
        },
        [picker],
    );

    const saveMutation = useMutation({
        mutationFn: async () => {
            setSaveError(null);
            await saveEdits({
                slug,
                original: originalState,
                current: state,
                t,
                onProgress: setSaveProgress,
            });
        },
        onSuccess: async () => {
            setSaveProgress(null);
            await queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            const fresh = queryClient.getQueryData<ITierListDetail | null>(tierListDetailQueryOptions(slug, gamedataServer).queryKey);
            if (fresh) {
                const next = detailToEditState(fresh);
                setOriginalState(next);
                dispatch({ type: "RESET", state: next });
            } else {
                setOriginalState(state);
            }
            toastManager.add({
                id: `tl-edit-save-${Date.now()}`,
                title: t("edit.toast.savedTitle"),
                description: t("edit.toast.savedBody"),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            setSaveProgress(null);
            const message = describeError(err);
            setSaveError(message);
            toastManager.add({
                id: `tl-edit-save-err-${Date.now()}`,
                title: t("edit.toast.saveFailedTitle"),
                description: message,
                type: "error",
            });
        },
    });

    const handleSave = useCallback(() => {
        if (!pendingChanges.length || saveMutation.isPending) return;
        saveMutation.mutate();
    }, [pendingChanges.length, saveMutation]);

    const flairMutation = useMutation({
        mutationFn: (flairId: number | null) => setTierListFlairFn({ data: { slug, flairId } }),
        onSuccess: async (_, flairId) => {
            const next = flairId === null ? null : ((flairCatalog ?? []).find((f) => f.id === flairId) ?? null);
            queryClient.setQueryData<ITierListDetail | null>(tierListDetailQueryOptions(slug, gamedataServer).queryKey, (prev) => (prev ? { ...prev, flair: next } : prev));
            await queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({
                id: `tl-flair-${Date.now()}`,
                title: next ? t("edit.toast.flairSetTitle") : t("edit.toast.flairClearedTitle"),
                description: next ? t("edit.toast.flairSetBody", { label: next.label }) : t("edit.toast.flairClearedBody"),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = describeError(err);
            toastManager.add({ id: `tl-flair-err-${Date.now()}`, title: t("edit.toast.flairFailedTitle"), description: message, type: "error" });
        },
    });

    const visibilityMutation = useMutation({
        mutationFn: (isListed: boolean) => setTierListVisibilityFn({ data: { slug, isListed } }),
        onSuccess: async (_, isListed) => {
            queryClient.setQueryData<ITierListDetail | null>(tierListDetailQueryOptions(slug, gamedataServer).queryKey, (prev) => (prev ? { ...prev, isListed } : prev));
            await queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({
                id: `tl-visibility-${Date.now()}`,
                title: isListed ? t("edit.toast.publicTitle") : t("edit.toast.hiddenTitle"),
                description: isListed ? t("edit.toast.publicBody") : t("edit.toast.hiddenBody"),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = describeError(err);
            toastManager.add({ id: `tl-visibility-err-${Date.now()}`, title: t("edit.toast.visibilityFailedTitle"), description: message, type: "error" });
        },
    });

    const publishMutation = useMutation({
        mutationFn: (changelog: string) => publishTierListVersionFn({ data: { slug, changelog: changelog.length > 0 ? changelog : null } }),
        onSuccess: async (version) => {
            setPublishError(null);
            setPublishOpen(false);
            await queryClient.invalidateQueries({ queryKey: ["tier-lists", "versions", slug] });
            toastManager.add({
                id: `tl-publish-${Date.now()}`,
                title: t("edit.toast.publishedTitle", { version: version.version }),
                description: version.changelog ? t("edit.toast.publishedWithChangelog") : t("edit.toast.publishedNoChangelog"),
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = describeError(err);
            setPublishError(message);
        },
    });

    const publishingDisabledReason = pendingChanges.length > 0 ? t("edit.publishBlocked") : null;
    const canPublish = !publishMutation.isPending && publishingDisabledReason === null;

    const handleOpenPublishDialog = useCallback(() => {
        setPublishError(null);
        setPublishOpen(true);
    }, []);

    const handleClosePublishDialog = useCallback(() => {
        if (publishMutation.isPending) return;
        setPublishOpen(false);
        setPublishError(null);
    }, [publishMutation.isPending]);

    const handlePublish = useCallback(
        (changelog: string) => {
            if (publishMutation.isPending) return;
            publishMutation.mutate(changelog);
        },
        [publishMutation],
    );

    const handleReset = useCallback(() => {
        if (!pendingChanges.length) return;
        dispatch({ type: "RESET", state: originalState });
        setSaveError(null);
    }, [pendingChanges.length, originalState]);

    const handleSaveTierSettings = useCallback(
        (next: { name: string; color: string; description: string }) => {
            if (!editingTier) return;
            dispatch({ type: "UPDATE_TIER", tierId: editingTier.id, ...next });
            setEditingTier(null);
        },
        [editingTier],
    );

    const handleDeleteTier = useCallback(() => {
        if (!editingTier) return;
        dispatch({ type: "DELETE_TIER", tierId: editingTier.id });
        setEditingTier(null);
    }, [editingTier]);

    const handleClearTier = useCallback(() => {
        if (!editingTier) return;
        dispatch({ type: "CLEAR_TIER", tierId: editingTier.id });
        setEditingTier(null);
    }, [editingTier]);

    const handlePickTier = useCallback(
        (tierId: string | null) => {
            if (!picker) return;
            dispatch({ type: "PLACE_ENTITY", key: picker.key, tierId, entity: picker });
        },
        [picker],
    );

    return (
        <DragControllerProvider entityByKey={entityByKey} onPlace={handlePlace} onUnplace={handleUnplace}>
            <main className="min-h-dvh pb-24">
                <EditHero
                    slug={slug}
                    title={state.title}
                    description={state.description}
                    onTitleChange={(title) => dispatch({ type: "SET_META", title, description: state.description })}
                    onDescriptionChange={(description) => dispatch({ type: "SET_META", title: state.title, description })}
                    onSave={handleSave}
                    onReset={handleReset}
                    onAddTier={handleAddTier}
                    pendingChanges={pendingChanges}
                    saving={saveMutation.isPending}
                    saveError={saveError}
                    saveProgress={saveProgress}
                />

                <div className="page-gutter mt-4 grid gap-4 [--page-max:1280px] sm:mt-6 sm:gap-6 lg:grid-cols-[minmax(0,1fr)_320px] lg:items-start">
                    <div className="min-w-0">
                        <section className={styles.board} aria-label={t("edit.board.label", { title: state.title })}>
                            {state.tiers.map((tier, idx) => (
                                <EditTierRow
                                    key={tier.id}
                                    tier={tier}
                                    entities={tier.entityKeys.map((key) => placedEntity(entityByKey, key))}
                                    notedKeys={notedKeys}
                                    canMoveUp={idx > 0}
                                    canMoveDown={idx < state.tiers.length - 1}
                                    onMoveUp={() => dispatch({ type: "MOVE_TIER", tierId: tier.id, direction: "up" })}
                                    onMoveDown={() => dispatch({ type: "MOVE_TIER", tierId: tier.id, direction: "down" })}
                                    onOpenSettings={() => setEditingTier(tier)}
                                    onPlace={handlePlace}
                                    onUnplace={handleUnplace}
                                    onActivateEntity={handleActivateEntity}
                                />
                            ))}
                        </section>
                        {state.tiers.length === 0 && (
                            <div className="mt-3 rounded-xl border border-border border-dashed bg-muted/20 px-5 py-10 text-center">
                                <p className="m-0 font-medium font-sans text-foreground text-sm">{t("edit.board.emptyTitle")}</p>
                                <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">{t("edit.board.emptyBody")}</p>
                                <Button onClick={handleAddTier} size="sm" className="mt-3">
                                    {t("edit.board.addTier")}
                                </Button>
                            </div>
                        )}
                    </div>

                    <aside className="flex flex-col gap-3 lg:sticky lg:top-20 lg:h-[calc(100dvh-6rem)] lg:min-h-0">
                        <PublishingPanel
                            flair={detail.flair ? { id: detail.flair.id, label: detail.flair.label, color: detail.flair.color } : null}
                            flairOptions={flairOptions}
                            isListed={detail.isListed ?? true}
                            canPublish={canPublish}
                            publishingDisabledReason={publishingDisabledReason}
                            settingFlair={flairMutation.isPending}
                            settingVisibility={visibilityMutation.isPending}
                            onSetFlair={(flairId) => flairMutation.mutate(flairId)}
                            onSetVisibility={(next) => visibilityMutation.mutate(next)}
                            onOpenPublishDialog={handleOpenPublishDialog}
                        />
                        <EntityPool kinds={kinds} catalogues={catalogues} placedKeys={placedKeys} onUnplace={handleUnplace} onPickerActivate={handleActivateEntity} onEditKinds={() => setKindsOpen(true)} rootClassName="h-[70dvh] lg:h-auto lg:min-h-0 lg:flex-1" />
                    </aside>
                </div>

                <TierSettingsDialog tier={editingTier} canDelete={state.tiers.length > 1} onClose={() => setEditingTier(null)} onSave={handleSaveTierSettings} onDelete={handleDeleteTier} onClear={handleClearTier} />

                <PickTierDialog entity={picker} currentTierId={picker ? findCurrentTierId(picker.key) : null} description={picker ? (state.descriptionByKey[picker.key] ?? "") : ""} tiers={state.tiers} onClose={() => setPicker(null)} onPick={handlePickTier} onDescriptionChange={handleEntityDescriptionChange} />

                <PoolKindsDialog
                    open={kindsOpen}
                    kinds={kinds}
                    placedByKind={placedByKind}
                    onClose={() => setKindsOpen(false)}
                    onApply={(next) => {
                        dispatch({ type: "SET_ENTITY_KINDS", kinds: next });
                        setKindsOpen(false);
                    }}
                />

                <PublishVersionDialog open={publishOpen} publishing={publishMutation.isPending} latestVersion={latestVersion} nextVersion={nextVersion} publishError={publishError} onClose={handleClosePublishDialog} onPublish={handlePublish} />
            </main>
        </DragControllerProvider>
    );
}

function defaultTierName(existing: number): string {
    const presets = ["S", "A", "B", "C", "D", "E", "F"];
    return presets[existing] ?? `T${existing + 1}`;
}

function EditorMissing() {
    const t: TypedT<typeof messages> = useT("tierLists");

    return (
        <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
            <h1 className="m-0 font-bold font-sans text-2xl text-foreground tracking-tight">{t("edit.missing.title")}</h1>
            <p className="mt-3 font-sans text-muted-foreground text-sm">{t("edit.missing.body")}</p>
            <Button render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />} variant="outline" className="mt-6">
                {t("edit.missing.action")}
            </Button>
        </main>
    );
}

function EditorForbidden({ slug }: { slug: string }) {
    const t: TypedT<typeof messages> = useT("tierLists");

    return (
        <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
            <h1 className="m-0 font-bold font-sans text-2xl text-foreground tracking-tight">{t("edit.forbidden.title")}</h1>
            <p className="mt-3 font-sans text-muted-foreground text-sm">{t("edit.forbidden.body")}</p>
            <Button render={<Link to="/tier-lists/$id" params={{ id: slug }} />} variant="outline" className="mt-6">
                {t("edit.forbidden.action")}
            </Button>
        </main>
    );
}
