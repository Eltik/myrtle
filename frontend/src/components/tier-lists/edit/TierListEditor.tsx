import { useMutation, useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useCallback, useMemo, useReducer, useState } from "react";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { type ITierListDetail, type ITierOperator, TierListApiError, tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import { indexEntryToTierOperator } from "../shared";
import { DragControllerProvider } from "./drag-controller";
import { EditHero } from "./EditHero";
import styles from "./Editor.module.css";
import { EditTierRow } from "./EditTierRow";
import { OperatorPool } from "./OperatorPool";
import { PickTierDialog } from "./PickTierDialog";
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

    const isOwner = user && detail.author?.id === user.id;
    if (!isOwner) return <EditorForbidden slug={slug} />;

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
            const message = err instanceof TierListApiError ? err.message : err instanceof Error ? err.message : "Couldn't save changes.";
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
                            <div className="mt-3 rounded-xl border border-dashed border-border bg-muted/20 px-5 py-10 text-center">
                                <p className="m-0 font-sans text-sm font-medium text-foreground">No tiers yet.</p>
                                <p className="mt-1 font-sans text-[12.5px] text-muted-foreground">Create your first tier to start ranking operators.</p>
                                <Button onClick={handleAddTier} size="sm" className="mt-3">
                                    Add tier
                                </Button>
                            </div>
                        )}
                    </div>

                    <aside className="lg:sticky lg:top-20 lg:h-[calc(100dvh-6rem)]">
                        <OperatorPool operators={operators} placedIds={placedIds} onUnplace={handleUnplace} onPickerActivate={handleActivateOperator} />
                    </aside>
                </div>

                <TierSettingsDialog tier={editingTier} canDelete={state.tiers.length > 1} onClose={() => setEditingTier(null)} onSave={handleSaveTierSettings} onDelete={handleDeleteTier} />

                <PickTierDialog operator={picker?.operator ?? null} currentTierId={picker?.currentTierId ?? null} tiers={state.tiers} onClose={() => setPicker(null)} onPick={handlePickTier} />
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
            <h1 className="m-0 font-sans text-2xl font-bold tracking-tight text-foreground">Tier list not found</h1>
            <p className="mt-3 font-sans text-sm text-muted-foreground">It may have been removed, or the link could be wrong.</p>
            <Button render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />} variant="outline" className="mt-6">
                Back to my lists
            </Button>
        </main>
    );
}

function EditorForbidden({ slug }: { slug: string }) {
    return (
        <main className="mx-auto w-[min(720px,calc(100%-2rem))] py-20 text-center">
            <h1 className="m-0 font-sans text-2xl font-bold tracking-tight text-foreground">You can't edit this list</h1>
            <p className="mt-3 font-sans text-sm text-muted-foreground">Only the author can edit this tier list.</p>
            <Button render={<Link to="/tier-lists/$id" params={{ id: slug }} />} variant="outline" className="mt-6">
                View public page
            </Button>
        </main>
    );
}
