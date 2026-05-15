import { useMutation, useQuery, useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useCallback, useMemo, useReducer, useState } from "react";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { ApiError } from "#/lib/api/_shared";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { type ITierListDetail, type ITierOperator, publishTierListVersionFn, setTierListFlairFn, setTierListVisibilityFn, tierListDetailQueryOptions, tierListFlairsQueryOptions, tierListVersionsQueryOptions } from "#/lib/api/tier-lists";
import { indexEntryToTierOperator } from "../shared";
import { DragControllerProvider } from "./drag-controller";
import { EditHero } from "./EditHero";
import styles from "./Editor.module.css";
import { EditTierRow } from "./EditTierRow";
import { OperatorPool } from "./OperatorPool";
import { PickTierDialog } from "./PickTierDialog";
import { PublishingPanel } from "./PublishingPanel";
import { PublishVersionDialog } from "./PublishVersionDialog";
import { type ISaveProgress, saveEdits } from "./save";
import { detailToEditState, diffStates, editReducer, type IEditState, type IEditTier, type IPendingChange, nextFallbackTierColor, placedOperatorIds } from "./state";
import { TierSettingsDialog } from "./TierSettingsDialog";

interface ITierListEditorProps {
    slug: string;
}

interface IPickerState {
    operator: ITierOperator;
    currentTierId: string | null;
}

export function TierListEditor({ slug }: ITierListEditorProps) {
    const { user } = useAuth();
    const queryClient = useQueryClient();

    const { data: detail } = useSuspenseQuery(tierListDetailQueryOptions(slug));
    const { data: operators } = useSuspenseQuery(operatorsIndexQueryOptions());

    if (!detail) return <EditorMissing />;

    const isOwner = Boolean(user && detail.author?.id === user.id);
    const isTierListAdmin = user?.role === "tier_list_admin" || user?.role === "super_admin";
    const canEdit = isOwner || isTierListAdmin;
    if (!canEdit) return <EditorForbidden slug={slug} />;

    return <EditorContent slug={slug} detail={detail} operators={operators ?? []} queryClient={queryClient} />;
}

interface IEditorContentProps {
    slug: string;
    detail: ITierListDetail;
    operators: import("#/types/operators").IOperatorIndexEntry[];
    queryClient: ReturnType<typeof useQueryClient>;
}

function EditorContent({ slug, detail, operators, queryClient }: IEditorContentProps) {
    const initial = useMemo(() => detailToEditState(detail), [detail]);
    const [originalState, setOriginalState] = useState<IEditState>(initial);
    const [state, dispatch] = useReducer(editReducer, initial);
    const [editingTier, setEditingTier] = useState<IEditTier | null>(null);
    const [picker, setPicker] = useState<IPickerState | null>(null);
    const [saveError, setSaveError] = useState<string | null>(null);
    const [saveProgress, setSaveProgress] = useState<ISaveProgress | null>(null);
    const [publishOpen, setPublishOpen] = useState(false);
    const [publishError, setPublishError] = useState<string | null>(null);

    const { data: flairCatalog } = useQuery(tierListFlairsQueryOptions());
    const { data: versions } = useQuery(tierListVersionsQueryOptions(slug));
    const flairOptions = useMemo(() => (flairCatalog ?? []).filter((f) => f.isActive), [flairCatalog]);
    const latestVersion = versions && versions.length > 0 ? Math.max(...versions.map((v) => v.version)) : null;
    const nextVersion = (latestVersion ?? 0) + 1;

    const operatorById = useMemo(() => {
        const merged = { ...state.operatorById };
        for (const op of operators) {
            if (merged[op.id]) continue;
            merged[op.id] = indexEntryToTierOperator(op);
        }
        return merged;
    }, [state.operatorById, operators]);

    const placedIds = useMemo(() => placedOperatorIds(state), [state]);
    const pendingChanges: IPendingChange[] = useMemo(() => diffStates(originalState, state), [originalState, state]);
    const findCurrentTierId = useCallback((operatorId: string): string | null => state.tiers.find((t) => t.operatorIds.includes(operatorId))?.id ?? null, [state.tiers]);

    const handlePlace = useCallback((operatorId: string, tierId: string, index: number) => {
        dispatch({ type: "PLACE_OPERATOR", operatorId, tierId, index });
    }, []);

    const handleUnplace = useCallback((operatorId: string) => {
        dispatch({ type: "PLACE_OPERATOR", operatorId, tierId: null });
    }, []);

    const handleAddTier = useCallback(() => {
        dispatch({ type: "ADD_TIER", name: defaultTierName(state.tiers.length), color: nextFallbackTierColor(state.tiers.length), description: "" });
    }, [state.tiers.length]);

    const handleActivateOperator = useCallback(
        (operator: ITierOperator) => {
            setPicker({ operator, currentTierId: findCurrentTierId(operator.id) });
        },
        [findCurrentTierId],
    );

    const saveMutation = useMutation({
        mutationFn: async () => {
            setSaveError(null);
            await saveEdits({
                slug,
                original: originalState,
                current: state,
                onProgress: setSaveProgress,
            });
        },
        onSuccess: async () => {
            setSaveProgress(null);
            await queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            const fresh = queryClient.getQueryData<ITierListDetail | null>(tierListDetailQueryOptions(slug).queryKey);
            if (fresh) {
                const next = detailToEditState(fresh);
                setOriginalState(next);
                dispatch({ type: "RESET", state: next });
            } else {
                setOriginalState(state);
            }
            toastManager.add({
                id: `tl-edit-save-${Date.now()}`,
                title: "Saved",
                description: "Your changes are live.",
                type: "success",
            });
        },
        onError: (err: unknown) => {
            setSaveProgress(null);
            const message = err instanceof ApiError ? err.message : err instanceof Error ? err.message : "Couldn't save changes.";
            setSaveError(message);
            toastManager.add({
                id: `tl-edit-save-err-${Date.now()}`,
                title: "Save failed",
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
            queryClient.setQueryData<ITierListDetail | null>(tierListDetailQueryOptions(slug).queryKey, (prev) => (prev ? { ...prev, flair: next } : prev));
            await queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({
                id: `tl-flair-${Date.now()}`,
                title: next ? "Flair updated" : "Flair cleared",
                description: next ? `Tagged as "${next.label}".` : "No flair on this list.",
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = err instanceof ApiError ? err.message : err instanceof Error ? err.message : "Couldn't update flair.";
            toastManager.add({ id: `tl-flair-err-${Date.now()}`, title: "Flair failed", description: message, type: "error" });
        },
    });

    const visibilityMutation = useMutation({
        mutationFn: (isListed: boolean) => setTierListVisibilityFn({ data: { slug, isListed } }),
        onSuccess: async (_, isListed) => {
            queryClient.setQueryData<ITierListDetail | null>(tierListDetailQueryOptions(slug).queryKey, (prev) => (prev ? { ...prev, isListed } : prev));
            await queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastManager.add({
                id: `tl-visibility-${Date.now()}`,
                title: isListed ? "Now public" : "Hidden from browse",
                description: isListed ? "This list appears on /tier-lists." : "Only people with the direct link can find it.",
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = err instanceof ApiError ? err.message : err instanceof Error ? err.message : "Couldn't update visibility.";
            toastManager.add({ id: `tl-visibility-err-${Date.now()}`, title: "Visibility failed", description: message, type: "error" });
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
                title: `Published v${version.version}`,
                description: version.changelog ? "Your changelog is now live." : "Snapshot saved.",
                type: "success",
            });
        },
        onError: (err: unknown) => {
            const message = err instanceof ApiError ? err.message : err instanceof Error ? err.message : "Couldn't publish version.";
            setPublishError(message);
        },
    });

    const publishingDisabledReason = pendingChanges.length > 0 ? "Save your changes before publishing a version." : null;
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

    const handlePickTier = useCallback(
        (tierId: string | null) => {
            if (!picker) return;
            dispatch({ type: "PLACE_OPERATOR", operatorId: picker.operator.id, tierId });
            setPicker(null);
        },
        [picker],
    );

    return (
        <DragControllerProvider operatorById={operatorById} onPlace={handlePlace} onUnplace={handleUnplace}>
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

                <div className="mx-auto mt-4 grid w-[min(1280px,calc(100%-1.5rem))] gap-4 sm:mt-6 sm:w-[min(1280px,calc(100%-2rem))] sm:gap-6 lg:grid-cols-[minmax(0,1fr)_320px] lg:items-start">
                    <div className="min-w-0">
                        <section className={styles.board} aria-label={`Edit board for ${state.title}`}>
                            {state.tiers.map((tier, idx) => (
                                <EditTierRow
                                    key={tier.id}
                                    tier={tier}
                                    operators={tier.operatorIds.map((id) => operatorById[id])}
                                    canMoveUp={idx > 0}
                                    canMoveDown={idx < state.tiers.length - 1}
                                    onMoveUp={() => dispatch({ type: "MOVE_TIER", tierId: tier.id, direction: "up" })}
                                    onMoveDown={() => dispatch({ type: "MOVE_TIER", tierId: tier.id, direction: "down" })}
                                    onOpenSettings={() => setEditingTier(tier)}
                                    onPlace={handlePlace}
                                    onUnplace={handleUnplace}
                                    onActivateOperator={handleActivateOperator}
                                />
                            ))}
                        </section>
                        {state.tiers.length === 0 && (
                            <div className="mt-3 rounded-xl border border-border border-dashed bg-muted/20 px-5 py-10 text-center">
                                <p className="m-0 font-medium font-sans text-foreground text-sm">No tiers yet.</p>
                                <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">Create your first tier to start ranking operators.</p>
                                <Button onClick={handleAddTier} size="sm" className="mt-3">
                                    Add tier
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
                        <OperatorPool operators={operators} placedIds={placedIds} onUnplace={handleUnplace} onPickerActivate={handleActivateOperator} rootClassName="h-[70dvh] lg:h-auto lg:min-h-0 lg:flex-1" />
                    </aside>
                </div>

                <TierSettingsDialog tier={editingTier} canDelete={state.tiers.length > 1} onClose={() => setEditingTier(null)} onSave={handleSaveTierSettings} onDelete={handleDeleteTier} />

                <PickTierDialog operator={picker?.operator ?? null} currentTierId={picker?.currentTierId ?? null} tiers={state.tiers} onClose={() => setPicker(null)} onPick={handlePickTier} />

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
    return (
        <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
            <h1 className="m-0 font-bold font-sans text-2xl text-foreground tracking-tight">Tier list not found</h1>
            <p className="mt-3 font-sans text-muted-foreground text-sm">It may have been removed, or the link could be wrong.</p>
            <Button render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />} variant="outline" className="mt-6">
                Back to my lists
            </Button>
        </main>
    );
}

function EditorForbidden({ slug }: { slug: string }) {
    return (
        <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
            <h1 className="m-0 font-bold font-sans text-2xl text-foreground tracking-tight">You can't edit this list</h1>
            <p className="mt-3 font-sans text-muted-foreground text-sm">Only the author can edit this tier list.</p>
            <Button render={<Link to="/tier-lists/$id" params={{ id: slug }} />} variant="outline" className="mt-6">
                View public page
            </Button>
        </main>
    );
}
