import { useSuspenseQuery } from "@tanstack/react-query";
import { Link, useBlocker } from "@tanstack/react-router";
import { CheckIcon, ExternalLinkIcon, LockIcon, RotateCcwIcon } from "lucide-react";
import { useCallback, useId, useMemo, useReducer, useRef, useState } from "react";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "#/components/ui/breadcrumb";
import { Button } from "#/components/ui/button";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Kicker } from "#/components/ui/kicker";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { Switch } from "#/components/ui/switch";
import { Textarea } from "#/components/ui/textarea";
import { useAuth } from "#/hooks/use-auth";
import { gridQueryOptions, type IGrid } from "#/lib/api/grids";
import type { ITierEntity, TierEntityKind } from "#/lib/api/tier-entities";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { truncateCodePoints } from "#/lib/markdown/sanitize-input";
import { AllowedKindsDialog, KindChips } from "./AllowedKinds";
import type { messages as kindsMessages } from "./AllowedKinds.messages";
import { ConfirmDialog } from "./ConfirmDialog";
import { EntityPickerDialog } from "./EntityPickerDialog";
import { GridBoard, type IGridBoardEditor } from "./GridBoard";
import type { messages } from "./GridEditor.messages";
import { GridNotice } from "./GridNotice";
import { useGridSave } from "./save";
import { GRID_DESCRIPTION_MAX, GRID_MAX_SIZE, GRID_MIN_SIZE, GRID_TITLE_MAX } from "./shared";
import { contentLostByResize, gridReducer, gridToState, type IGridEditState, pickerTarget, picksByKind, picksLostByKinds } from "./state";

type EditorT = TypedT<typeof messages>;
type KindsT = TypedT<typeof kindsMessages>;

export function GridEditor({ slug }: { slug: string }) {
    const server = useGamedataServer();
    const { user } = useAuth();
    const { data: grid } = useSuspenseQuery(gridQueryOptions(slug, user?.id ?? null, server));
    if (!grid) return <EditorNotice slug={null} kind="missing" />;
    if (!grid.can_edit) return <EditorNotice slug={slug} kind="forbidden" />;
    return <EditorContent grid={grid} viewerId={user?.id ?? null} />;
}

function EditorContent({ grid, viewerId }: { grid: IGrid; viewerId: string | null }) {
    const t: EditorT = useT("grids");
    const tk: KindsT = useT("grids");
    const slug = grid.slug;

    const initial = useMemo(() => gridToState(grid), [grid]);
    const [state, dispatch] = useReducer(gridReducer, initial);
    const { dirty, saving, saveError, save, discard } = useGridSave({ slug, viewerId, initial, state, dispatch });
    const [pickIndex, setPickIndex] = useState<number | null>(null);
    const [pendingResize, setPendingResize] = useState<{ rows: number; cols: number; lost: number } | null>(null);
    const [kindsOpen, setKindsOpen] = useState(false);
    const [pendingKinds, setPendingKinds] = useState<{ kinds: TierEntityKind[]; lost: number } | null>(null);

    const trimmedTitle = state.title.trim();
    const titleValid = trimmedTitle.length > 0 && Array.from(trimmedTitle).length <= GRID_TITLE_MAX;

    // Leaving with unsaved changes, or while a save is still in flight, asks in-app first. The ref keeps the check current without re-registering the blocker each render.
    const blockRef = useRef(false);
    blockRef.current = dirty || saving;
    const blocker = useBlocker({ shouldBlockFn: () => blockRef.current, enableBeforeUnload: () => blockRef.current, withResolver: true });

    const handleSave = useCallback(() => {
        if (!dirty || !titleValid || saving) return;
        save(state);
    }, [dirty, titleValid, saving, save, state]);

    // Shrinking the board past cells with content drops them, so it asks first.
    const requestResize = useCallback(
        (rows: number, cols: number) => {
            const lost = contentLostByResize(state, rows, cols);
            if (lost > 0) setPendingResize({ rows, cols, lost });
            else dispatch({ type: "resize", rows, cols });
        },
        [state],
    );

    const placedByKind = useMemo(() => picksByKind(state.cells), [state.cells]);

    // Removing a type some cells hold clears those picks, so it asks first.
    const applyKinds = useCallback(
        (kinds: TierEntityKind[]) => {
            setKindsOpen(false);
            const lost = picksLostByKinds(state, kinds);
            if (lost > 0) setPendingKinds({ kinds, lost });
            else dispatch({ type: "setKinds", kinds });
        },
        [state],
    );

    const boardEditor: IGridBoardEditor = useMemo(
        () => ({
            onPick: (index) => setPickIndex(index),
            onLabelChange: (index, label) => dispatch({ type: "setLabel", index, label }),
            onSwap: (from, to) => dispatch({ type: "swap", from, to }),
        }),
        [],
    );

    const handlePick = (entity: ITierEntity) => {
        if (pickIndex === null) return;
        dispatch({ type: "setEntity", index: pickIndex, entity });
        setPickIndex(null);
    };

    const handleClear = () => {
        if (pickIndex === null) return;
        dispatch({ type: "clearEntity", index: pickIndex });
        setPickIndex(null);
    };

    return (
        <main className="min-h-dvh pb-24">
            <EditorHeader
                slug={slug}
                state={state}
                dirty={dirty}
                titleValid={titleValid}
                saving={saving}
                saveError={saveError}
                onTitleChange={(title) => dispatch({ type: "setTitle", title })}
                onDescriptionChange={(description) => dispatch({ type: "setDescription", description })}
                onListedChange={(isListed) => dispatch({ type: "setListed", isListed })}
                onResize={requestResize}
                kindsLocked={grid.kinds_locked}
                onChangeKinds={() => setKindsOpen(true)}
                onSave={handleSave}
                onDiscard={discard}
            />

            <div className="page-gutter mt-6 [--page-max:1180px]">
                <GridBoard title={state.title} rows={state.rows} cols={state.cols} cells={state.cells} editor={boardEditor} />
                <p className="mt-3 text-center font-sans text-muted-foreground text-xs">{t("edit.hint")}</p>
            </div>

            <EntityPickerDialog target={pickerTarget(state, pickIndex)} kinds={state.entityKinds} onClose={() => setPickIndex(null)} onPick={handlePick} onClear={handleClear} />

            <ConfirmDialog
                open={pendingResize !== null}
                title={t("edit.shrink.title")}
                body={t("edit.shrink.body", { count: pendingResize?.lost ?? 0, rows: pendingResize?.rows ?? state.rows, cols: pendingResize?.cols ?? state.cols })}
                confirmLabel={t("edit.shrink.confirm")}
                cancelLabel={t("edit.shrink.cancel")}
                destructive
                onCancel={() => setPendingResize(null)}
                onConfirm={() => {
                    if (pendingResize) dispatch({ type: "resize", rows: pendingResize.rows, cols: pendingResize.cols });
                    setPendingResize(null);
                }}
            />

            <AllowedKindsDialog open={kindsOpen} kinds={state.entityKinds} placedByKind={placedByKind} onClose={() => setKindsOpen(false)} onApply={applyKinds} />

            <ConfirmDialog
                open={pendingKinds !== null}
                title={tk("kinds.clear.title")}
                body={tk("kinds.clear.body", { count: pendingKinds?.lost ?? 0 })}
                confirmLabel={tk("kinds.clear.confirm")}
                cancelLabel={tk("kinds.clear.cancel")}
                destructive
                onCancel={() => setPendingKinds(null)}
                onConfirm={() => {
                    if (pendingKinds) dispatch({ type: "setKinds", kinds: pendingKinds.kinds });
                    setPendingKinds(null);
                }}
            />

            <ConfirmDialog open={blocker.status === "blocked"} title={t("edit.leave.title")} body={t("edit.leave.body")} confirmLabel={t("edit.leave.confirm")} cancelLabel={t("edit.leave.cancel")} destructive onCancel={() => blocker.reset?.()} onConfirm={() => blocker.proceed?.()} />
        </main>
    );
}

interface IEditorHeaderProps {
    slug: string;
    state: IGridEditState;
    dirty: boolean;
    titleValid: boolean;
    saving: boolean;
    saveError: string | null;
    onTitleChange: (title: string) => void;
    onDescriptionChange: (description: string) => void;
    onListedChange: (isListed: boolean) => void;
    onResize: (rows: number, cols: number) => void;
    /** The grid is a fork: its allowed types are its template's. */
    kindsLocked: boolean;
    onChangeKinds: () => void;
    onSave: () => void;
    onDiscard: () => void;
}

function EditorHeader({ slug, state, dirty, titleValid, saving, saveError, onTitleChange, onDescriptionChange, onListedChange, onResize, kindsLocked, onChangeKinds, onSave, onDiscard }: IEditorHeaderProps) {
    const t: EditorT = useT("grids");
    const tk: KindsT = useT("grids");
    const titleId = useId();
    const descId = useId();
    const listedId = useId();
    const listedHintId = useId();

    return (
        <header className="border-border/60 border-b bg-linear-to-b from-card/40 to-transparent">
            <div className="page-gutter pt-5 pb-6 [--page-max:1180px] sm:pt-8 sm:pb-7">
                <Breadcrumb className="mb-3">
                    <BreadcrumbList className="text-xs">
                        <BreadcrumbItem>
                            <BreadcrumbLink render={<Link to="/grids/my" />}>{t("edit.breadcrumbMine")}</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbLink render={<Link to="/grids/$slug" params={{ slug }} />}>{state.title.trim() || t("edit.breadcrumbUntitled")}</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage className="font-medium">{t("edit.breadcrumbEdit")}</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>

                <div className="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
                    <div className="flex min-w-0 flex-1 flex-col gap-3">
                        <Kicker>{t("edit.kicker")}</Kicker>
                        <Field>
                            <FieldLabel htmlFor={titleId} className="sr-only">
                                {t("edit.titleLabel")}
                            </FieldLabel>
                            <Input
                                id={titleId}
                                value={state.title}
                                onChange={(e) => onTitleChange(truncateCodePoints((e.target as HTMLInputElement).value, GRID_TITLE_MAX))}
                                placeholder={t("edit.titlePlaceholder")}
                                size="lg"
                                aria-invalid={!titleValid || undefined}
                                className="font-bold text-2xl sm:text-3xl [&_input]:font-sans [&_input]:tracking-tight"
                            />
                        </Field>
                        <Field>
                            <FieldLabel htmlFor={descId} className="sr-only">
                                {t("edit.descriptionLabel")}
                            </FieldLabel>
                            <Textarea id={descId} value={state.description} onChange={(e) => onDescriptionChange(truncateCodePoints(e.target.value, GRID_DESCRIPTION_MAX))} placeholder={t("edit.descriptionPlaceholder")} rows={2} />
                        </Field>

                        <div className="flex flex-wrap items-end gap-x-5 gap-y-3">
                            <SizeStepper label={t("edit.rows")} value={state.rows} onChange={(rows) => onResize(rows, state.cols)} />
                            <SizeStepper label={t("edit.cols")} value={state.cols} onChange={(cols) => onResize(state.rows, cols)} />
                            <div className="flex items-center gap-2.5 pb-1">
                                <Switch id={listedId} checked={state.isListed} onCheckedChange={onListedChange} aria-describedby={listedHintId} />
                                <div className="flex flex-col">
                                    <label htmlFor={listedId} className="cursor-pointer font-medium font-sans text-foreground text-sm">
                                        {t("edit.listed")}
                                    </label>
                                    <span id={listedHintId} className="font-sans text-muted-foreground text-xs">
                                        {state.isListed ? t("edit.listedOn") : t("edit.listedOff")}
                                    </span>
                                </div>
                            </div>
                        </div>

                        <div className="flex flex-col gap-1.5">
                            <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{tk("kinds.label")}</span>
                            <div className="flex flex-wrap items-center gap-2">
                                <KindChips kinds={state.entityKinds} />
                                {kindsLocked ? (
                                    <span className="inline-flex items-center gap-1 font-sans text-muted-foreground text-xs">
                                        <LockIcon className="h-3 w-3" aria-hidden="true" />
                                        {tk("kinds.locked")}
                                    </span>
                                ) : (
                                    <Button type="button" variant="outline" size="xs" onClick={onChangeKinds}>
                                        {tk("kinds.change")}
                                    </Button>
                                )}
                            </div>
                        </div>
                    </div>

                    <div className="flex w-full shrink-0 flex-col gap-3 lg:w-72">
                        <div className="rounded-xl border border-border bg-card p-3 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
                            <div className="flex items-center justify-between gap-2">
                                <span className="font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">{dirty ? t("edit.unsaved") : t("edit.allSaved")}</span>
                                {!dirty && <CheckIcon className="h-3.5 w-3.5 text-primary" aria-hidden="true" />}
                            </div>
                            <p className="mt-1 font-sans text-[12px] text-muted-foreground">{dirty ? t("edit.unsavedBody") : t("edit.upToDate")}</p>
                            {!titleValid && <p className="mt-1 font-sans text-[12px] text-destructive-foreground">{t("edit.titleRequired")}</p>}
                            {saveError && (
                                <div role="alert" className="mt-2 rounded-md border border-destructive/30 bg-destructive/8 px-2 py-1.5 font-sans text-[11.5px] text-destructive-foreground">
                                    {saveError}
                                </div>
                            )}
                        </div>
                        <div className="flex items-center gap-1.5">
                            <Button type="button" onClick={onSave} disabled={!dirty || saving || !titleValid} loading={saving} className="flex-1">
                                <CheckIcon />
                                {t("edit.save")}
                            </Button>
                            <Button type="button" variant="outline" onClick={onDiscard} disabled={!dirty || saving} size="icon" aria-label={t("edit.discard")} title={t("edit.discard")}>
                                <RotateCcwIcon />
                            </Button>
                            <Button type="button" variant="ghost" render={<Link to="/grids/$slug" params={{ slug }} target="_blank" rel="noreferrer" />} size="icon" aria-label={t("edit.openPublic")} title={t("edit.openPublic")}>
                                <ExternalLinkIcon />
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </header>
    );
}

function SizeStepper({ label, value, onChange }: { label: string; value: number; onChange: (value: number) => void }) {
    return (
        <NumberField value={value} min={GRID_MIN_SIZE} max={GRID_MAX_SIZE} step={1} onValueChange={(v) => v !== null && onChange(v)} size="sm" className="w-32 gap-1">
            <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{label}</span>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput aria-label={label} />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}

function EditorNotice({ slug, kind }: { slug: string | null; kind: "missing" | "forbidden" }) {
    const t: EditorT = useT("grids");
    return (
        <GridNotice
            title={kind === "missing" ? t("edit.missing.title") : t("edit.forbidden.title")}
            body={kind === "missing" ? t("edit.missing.body") : t("edit.forbidden.body")}
            action={
                slug ? (
                    <Button render={<Link to="/grids/$slug" params={{ slug }} />} variant="outline" className="mt-6">
                        {t("edit.forbidden.action")}
                    </Button>
                ) : (
                    <Button render={<Link to="/grids/my" />} variant="outline" className="mt-6">
                        {t("edit.missing.action")}
                    </Button>
                )
            }
        />
    );
}
