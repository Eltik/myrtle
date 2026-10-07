import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeftIcon, ChevronDownIcon, ChevronUpIcon, GripVerticalIcon, HeartIcon, LayoutGridIcon, ListOrderedIcon, PlusIcon, TargetIcon, Trash2Icon, XIcon } from "lucide-react";
import { type KeyboardEvent, useId, useMemo, useState } from "react";
import { EntityPickerBody } from "#/components/grids/EntityPickerDialog";
import { EntityAvatar } from "#/components/tier-lists/entities";
import { useEntityLabels } from "#/components/tier-lists/kinds";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { useOperatorName } from "#/hooks/use-operator-name";
import { useInvalidateProfile } from "#/hooks/use-resync-roster";
import { updateUserSettingsFn } from "#/lib/api/auth";
import { gridQueryOptions, myGridsQueryOptions } from "#/lib/api/grids";
import { publicPlansQueryOptions } from "#/lib/api/planner";
import { ALL_ENTITY_KINDS, type ITierEntity, type TierEntityKind } from "#/lib/api/tier-entities";
import { myTierListsQueryOptions, tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ShowcaseView } from "#/types/generated/ShowcaseView";
import { moveEntry } from "../../../layout";
import { blocksForSave, draftFromView, draftKey, droppedOnSave, newFavourites, SHOWCASE_MAX_BLOCKS, SHOWCASE_MAX_IDS, SHOWCASE_TITLE_MAX, type ShowcaseBlockType, type ShowcaseDraftBlock, showcaseForSave, slugFromInput, toggleFavourite } from "../../../showcase";
import type { messages } from "./ShowcaseEditor.messages";
import { BLOCK_ACCENT, RemovedBadge } from "./ShowcaseTab";

type EditorT = TypedT<typeof messages>;
type FavouritesDraft = Extract<ShowcaseDraftBlock, { type: "favourites" }>;

const DRAG_MIME = "application/x-showcase-block";
const ENTITY_DRAG_MIME = "application/x-showcase-entity";

const BLOCK_ICON = { favourites: HeartIcon, grid: LayoutGridIcon, tier_list: ListOrderedIcon, plan: TargetIcon } as const;

interface IShowcaseEditorProps {
    uid: string;
    view: ShowcaseView;
    /** The owner hid their Plans tab, so visitors are not sent plan blocks. */
    plansHidden: boolean;
    onClose: () => void;
}

/**
 * The owner's showcase editor, in place of the showcase. Blocks reorder by drag or by
 * the arrow buttons, as tabs do in the tab editor; a favourites block's picks reorder
 * by drag or by the arrow keys on a focused pick. Nothing is stored until Save, which
 * sends the showcase alone.
 */
export function ShowcaseEditor({ uid, view, plansHidden, onClose }: IShowcaseEditorProps) {
    const t: EditorT = useT("user");
    const labels = useEntityLabels();
    const describeError = useErrorMessage();
    const invalidateProfile = useInvalidateProfile();
    const [draft, setDraft] = useState<ShowcaseDraftBlock[]>(() => draftFromView(view));
    const [adding, setAdding] = useState(false);
    const [picking, setPicking] = useState<string | null>(null);
    const [dragFrom, setDragFrom] = useState<number | null>(null);
    const [dragOver, setDragOver] = useState<number | null>(null);
    const [announcement, setAnnouncement] = useState("");

    const save = useMutation({
        mutationFn: (next: ShowcaseDraftBlock[]) =>
            // Only the showcase: the privacy flags, the tabs and the background stay as stored.
            updateUserSettingsFn({ data: { profile_layout: showcaseForSave(blocksForSave(next)) } }),
        onSuccess: async () => {
            await invalidateProfile();
            toastManager.add({ id: `showcase-${Date.now()}`, title: t("profile.showcase.editor.saved.title"), description: t("profile.showcase.editor.saved.body"), type: "success" });
            onClose();
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `showcase-err-${Date.now()}`, title: t("profile.showcase.editor.saveFailed.title"), description: describeError(err), type: "error" });
        },
    });

    const blockName = (block: ShowcaseDraftBlock) => (block.type === "favourites" ? block.title.trim() || t("profile.showcase.editor.favouritesOf", { kind: labels.plural(block.kind) }) : typeText(t)[block.type].label);

    const move = (index: number, delta: number) => {
        const block = draft[index];
        const moved = moveEntry(draft, index, delta);
        if (!block || !moved) return;
        const { next, to } = moved;
        setDraft(next);
        setAnnouncement(t("profile.showcase.editor.moved", { item: blockName(block), position: to + 1, total: next.length }));
        // A move to either end disables the arrow that was pressed; keep the keyboard on the row's other arrow.
        const edge = to === 0 ? "down" : to === next.length - 1 ? "up" : null;
        if (edge) focusSoon(`[data-showcase-move="${block.key}:${edge}"]`);
    };
    const drop = (to: number) => {
        if (dragFrom !== null) move(dragFrom, to - dragFrom);
        setDragFrom(null);
        setDragOver(null);
    };
    const update = (key: string, change: (block: ShowcaseDraftBlock) => ShowcaseDraftBlock) => setDraft((prev) => prev.map((b) => (b.key === key ? change(b) : b)));
    const add = (block: ShowcaseDraftBlock) => {
        setDraft((prev) => (prev.length >= SHOWCASE_MAX_BLOCKS ? prev : [...prev, block]));
        setAdding(false);
        if (block.type === "favourites") setPicking(block.key);
    };

    const full = draft.length >= SHOWCASE_MAX_BLOCKS;
    const dropped = droppedOnSave(draft);
    const hasPlan = draft.some((b) => b.type === "plan" && !b.removed);
    const pickingBlock = draft.find((b): b is FavouritesDraft => b.key === picking && b.type === "favourites") ?? null;

    return (
        <section className="flex flex-col gap-4 rounded-2xl border border-border/50 bg-card/40 p-4 sm:p-5" aria-labelledby="showcase-editor-kicker">
            <div className="flex flex-col gap-1 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <Kicker id="showcase-editor-kicker">{t("profile.showcase.editor.kicker")}</Kicker>
                    <p className="max-w-prose text-muted-foreground text-sm">{t("profile.showcase.editor.help")}</p>
                </div>
                <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{t("profile.showcase.editor.count", { count: draft.length, max: SHOWCASE_MAX_BLOCKS })}</span>
            </div>

            {draft.length === 0 ? (
                <p className="rounded-xl border border-border border-dashed px-4 py-8 text-center text-muted-foreground text-sm">{t("profile.showcase.editor.empty")}</p>
            ) : (
                <ol className="m-0 flex list-none flex-col gap-2 p-0" aria-label={t("profile.showcase.editor.list")}>
                    {draft.map((block, i) => {
                        const name = blockName(block);
                        const Icon = BLOCK_ICON[block.type];
                        return (
                            <li
                                key={block.key}
                                onDragOver={(e) => {
                                    if (dragFrom === null || !e.dataTransfer.types.includes(DRAG_MIME)) return;
                                    e.preventDefault();
                                    e.dataTransfer.dropEffect = "move";
                                    if (dragOver !== i) setDragOver(i);
                                }}
                                onDrop={(e) => {
                                    if (!e.dataTransfer.types.includes(DRAG_MIME)) return;
                                    e.preventDefault();
                                    drop(i);
                                }}
                                className={cn("relative overflow-hidden rounded-xl border border-border/50 bg-background/60 transition-colors", dragFrom === i && "opacity-50", dragOver === i && dragFrom !== i && "border-primary/60 bg-primary/5")}
                            >
                                <div aria-hidden="true" className="absolute inset-y-0 left-0 w-0.5" style={{ background: BLOCK_ACCENT[block.type] }} />
                                <div className="flex items-center gap-2 py-1.5 pr-1.5 pl-2">
                                    {/* Only the grip starts a drag, so text in the title field stays selectable. */}
                                    <span
                                        draggable
                                        onDragStart={(e) => {
                                            e.dataTransfer.effectAllowed = "move";
                                            e.dataTransfer.setData(DRAG_MIME, block.key);
                                            const row = e.currentTarget.closest("li");
                                            if (row) e.dataTransfer.setDragImage(row, 16, 16);
                                            setDragFrom(i);
                                        }}
                                        onDragEnd={() => {
                                            setDragFrom(null);
                                            setDragOver(null);
                                        }}
                                        className="flex cursor-grab items-center text-muted-foreground active:cursor-grabbing"
                                        aria-hidden="true"
                                    >
                                        <GripVerticalIcon className="size-4 shrink-0" />
                                    </span>
                                    <Icon aria-hidden="true" className="size-4 shrink-0 text-muted-foreground" />
                                    <span className="min-w-0 flex-1 truncate font-medium text-sm">{block.type === "favourites" ? name : <ReferentName block={block} uid={uid} />}</span>
                                    {block.removed && <RemovedBadge />}
                                    <Button type="button" size="icon-xs" variant="outline" data-showcase-move={`${block.key}:up`} onClick={() => move(i, -1)} disabled={i === 0} aria-label={t("profile.showcase.editor.moveUp", { item: name })}>
                                        <ChevronUpIcon />
                                    </Button>
                                    <Button type="button" size="icon-xs" variant="outline" data-showcase-move={`${block.key}:down`} onClick={() => move(i, 1)} disabled={i === draft.length - 1} aria-label={t("profile.showcase.editor.moveDown", { item: name })}>
                                        <ChevronDownIcon />
                                    </Button>
                                    <Button type="button" size="icon-xs" variant="ghost" onClick={() => setDraft((prev) => prev.filter((b) => b.key !== block.key))} aria-label={t("profile.showcase.editor.remove", { item: name })}>
                                        <Trash2Icon />
                                    </Button>
                                </div>
                                {block.type === "favourites" && !block.removed && <FavouritesFields block={block} onChange={(next) => update(block.key, () => next)} onPick={() => setPicking(block.key)} onAnnounce={setAnnouncement} />}
                            </li>
                        );
                    })}
                </ol>
            )}
            <p className="sr-only" aria-live="polite">
                {announcement}
            </p>

            {hasPlan && plansHidden && <p className="text-muted-foreground text-sm">{t("profile.showcase.editor.plansHidden")}</p>}
            {dropped > 0 && <p className="text-muted-foreground text-sm">{t("profile.showcase.editor.dropped", { count: dropped })}</p>}

            <div className="flex flex-wrap items-center gap-2">
                <Button type="button" variant="outline" size="sm" onClick={() => setAdding(true)} disabled={full}>
                    <PlusIcon />
                    {t("profile.showcase.editor.add")}
                </Button>
                {full && <span className="text-muted-foreground text-xs">{t("profile.showcase.editor.full", { max: SHOWCASE_MAX_BLOCKS })}</span>}
                <div className="ml-auto flex items-center gap-2">
                    <Button type="button" variant="outline" size="sm" onClick={onClose} disabled={save.isPending}>
                        {t("profile.showcase.editor.cancel")}
                    </Button>
                    <Button type="button" size="sm" onClick={() => save.mutate(draft)} loading={save.isPending}>
                        {t("profile.showcase.editor.save")}
                    </Button>
                </div>
            </div>

            <AddBlockDialog open={adding} uid={uid} draft={draft} onClose={() => setAdding(false)} onAdd={add} />
            <FavouritesPickerDialog block={pickingBlock} onChange={(next) => update(next.key, () => next)} onClose={() => setPicking(null)} />
        </section>
    );
}

/** Focuses the element `selector` matches once the next frame has rendered it. */
function focusSoon(selector: string) {
    requestAnimationFrame(() => document.querySelector<HTMLElement>(selector)?.focus());
}

/** Each block type's name and one-line description, as literal `t` calls so the extractor sees every key. */
function typeText(t: EditorT): Record<ShowcaseBlockType, { label: string; desc: string }> {
    return {
        favourites: { label: t("profile.showcase.editor.type.favourites"), desc: t("profile.showcase.editor.type.favouritesDesc") },
        grid: { label: t("profile.showcase.editor.type.grid"), desc: t("profile.showcase.editor.type.gridDesc") },
        tier_list: { label: t("profile.showcase.editor.type.tierList"), desc: t("profile.showcase.editor.type.tierListDesc") },
        plan: { label: t("profile.showcase.editor.type.plan"), desc: t("profile.showcase.editor.type.planDesc") },
    };
}

/** A grid's, tier list's or plan's own name, read from the caches the showcase already filled. */
function ReferentName({ block, uid }: { block: Exclude<ShowcaseDraftBlock, FavouritesDraft>; uid: string }) {
    const t: EditorT = useT("user");
    const server = useGamedataServer();
    const { user } = useAuth();
    const operatorName = useOperatorName();
    const grid = useQuery({ ...gridQueryOptions(block.type === "grid" ? block.slug : "", user?.id ?? null, server), enabled: block.type === "grid" && !block.removed });
    const list = useQuery({ ...tierListDetailQueryOptions(block.type === "tier_list" ? block.slug : "", server), enabled: block.type === "tier_list" && !block.removed });
    const plans = useQuery({ ...publicPlansQueryOptions(uid), enabled: block.type === "plan" });
    const kind = typeText(t)[block.type].label;
    let name: string;
    if (block.type === "grid") name = grid.data?.title ?? t("profile.showcase.editor.unknownGrid", { slug: block.slug });
    else if (block.type === "tier_list") name = list.data?.title ?? t("profile.showcase.editor.unknownTierList", { slug: block.slug });
    else {
        const op = plans.data?.find((p) => p.id === block.id)?.operator;
        name = op ? operatorName(op) : t("profile.showcase.editor.unknownPlan");
    }
    return (
        <>
            <span className="text-muted-foreground">{kind}</span>
            <span aria-hidden="true" className="mx-1.5 text-muted-foreground/60">
                ·
            </span>
            {name}
        </>
    );
}

interface IFavouritesFieldsProps {
    block: FavouritesDraft;
    onChange: (next: FavouritesDraft) => void;
    onPick: () => void;
    onAnnounce: (text: string) => void;
}

/** A favourites block's title and its picks: reorder by drag or arrow keys, Delete removes. */
function FavouritesFields({ block, onChange, onPick, onAnnounce }: IFavouritesFieldsProps) {
    const t: EditorT = useT("user");
    const labels = useEntityLabels();
    const titleId = useId();
    const hintId = useId();
    const [dragFrom, setDragFrom] = useState<number | null>(null);
    const plural = labels.plural(block.kind);

    const nameOf = (i: number) => {
        const e = block.entities[i];
        return e?.entity ? labels.tileLabel(e.entity) : (e?.id ?? "");
    };
    const moveEntity = (index: number, delta: number, focus: boolean) => {
        const moved = moveEntry(block.entities, index, delta);
        if (!moved) return;
        onChange({ ...block, entities: moved.next });
        onAnnounce(t("profile.showcase.editor.moved", { item: nameOf(index), position: moved.to + 1, total: moved.next.length }));
        // Keep the keyboard on the pick it is carrying.
        if (focus) focusSoon(`[data-showcase-pick="${block.key}:${moved.to}"]`);
    };
    const removeEntity = (index: number) => {
        onChange({ ...block, entities: block.entities.filter((_, i) => i !== index) });
        focusSoon(`[data-showcase-pick="${block.key}:${Math.max(0, index - 1)}"]`);
    };
    const onKey = (e: KeyboardEvent, i: number) => {
        if (e.key === "ArrowLeft" || e.key === "ArrowUp") moveEntity(i, -1, true);
        else if (e.key === "ArrowRight" || e.key === "ArrowDown") moveEntity(i, 1, true);
        else if (e.key === "Delete" || e.key === "Backspace") removeEntity(i);
        else return;
        e.preventDefault();
    };

    return (
        <div className="flex flex-col gap-2.5 border-border/40 border-t px-3 pt-2.5 pb-3">
            <div className="flex items-center gap-2">
                <label htmlFor={titleId} className="sr-only">
                    {t("profile.showcase.editor.titleLabel")}
                </label>
                <Input id={titleId} size="sm" value={block.title} maxLength={SHOWCASE_TITLE_MAX} placeholder={t("profile.showcase.editor.titlePlaceholder", { kind: plural })} onChange={(e) => onChange({ ...block, title: (e.target as HTMLInputElement).value })} className="max-w-xs" />
                <span className="ml-auto shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{t("profile.showcase.editor.entities", { count: block.entities.length, max: SHOWCASE_MAX_IDS })}</span>
            </div>
            {block.entities.length === 0 ? (
                <p className="m-0 text-muted-foreground text-xs">{t("profile.showcase.editor.emptyFavourites")}</p>
            ) : (
                <ul className="m-0 flex list-none flex-wrap gap-1.5 p-0" aria-label={t("profile.showcase.editor.entitiesList")} aria-describedby={hintId}>
                    {block.entities.map((item, i) => {
                        const name = nameOf(i);
                        return (
                            <li
                                key={item.id}
                                className="group relative"
                                onDragOver={(e) => {
                                    if (dragFrom === null || !e.dataTransfer.types.includes(ENTITY_DRAG_MIME)) return;
                                    e.preventDefault();
                                    e.stopPropagation();
                                }}
                                onDrop={(e) => {
                                    if (dragFrom === null || !e.dataTransfer.types.includes(ENTITY_DRAG_MIME)) return;
                                    e.preventDefault();
                                    e.stopPropagation();
                                    moveEntity(dragFrom, i - dragFrom, false);
                                    setDragFrom(null);
                                }}
                            >
                                <button
                                    type="button"
                                    draggable
                                    data-showcase-pick={`${block.key}:${i}`}
                                    onDragStart={(e) => {
                                        e.stopPropagation();
                                        e.dataTransfer.effectAllowed = "move";
                                        e.dataTransfer.setData(ENTITY_DRAG_MIME, item.id);
                                        setDragFrom(i);
                                    }}
                                    onDragEnd={() => setDragFrom(null)}
                                    onKeyDown={(e) => onKey(e, i)}
                                    aria-label={name}
                                    title={name}
                                    className={cn(
                                        "relative flex size-12 cursor-grab items-center justify-center overflow-hidden rounded-md border bg-[oklch(0.2_0.005_285)] text-sm text-white outline-none focus-visible:ring-2 focus-visible:ring-ring active:cursor-grabbing",
                                        item.entity ? "border-border/60" : "border-destructive/60 border-dashed",
                                        dragFrom === i && "opacity-40",
                                    )}
                                >
                                    {item.entity ? <EntityAvatar entity={item.entity} tone="dark" server={item.server ?? undefined} /> : <span className="line-clamp-3 break-all px-0.5 text-center font-mono text-[8px] leading-tight">{item.id}</span>}
                                </button>
                                <button
                                    type="button"
                                    tabIndex={-1}
                                    onClick={() => removeEntity(i)}
                                    aria-label={t("profile.showcase.editor.remove", { item: name })}
                                    className="absolute -top-1.5 -right-1.5 hidden size-5 items-center justify-center rounded-full border border-border bg-popover text-muted-foreground shadow-sm hover:text-foreground group-focus-within:flex group-hover:flex"
                                >
                                    <XIcon className="size-3" />
                                </button>
                            </li>
                        );
                    })}
                </ul>
            )}
            <div className="flex flex-wrap items-center gap-2">
                <Button type="button" variant="outline" size="xs" onClick={onPick}>
                    <PlusIcon />
                    {t("profile.showcase.editor.pick")}
                </Button>
                {block.entities.length > 1 && (
                    <span id={hintId} className="text-muted-foreground text-xs">
                        {t("profile.showcase.editor.entityHint")}
                    </span>
                )}
            </div>
        </div>
    );
}

interface IFavouritesPickerDialogProps {
    /** The block being picked for; `null` when closed. */
    block: FavouritesDraft | null;
    onChange: (next: FavouritesDraft) => void;
    onClose: () => void;
}

/** The grid editor's entity picker, held open: each pick adds a favourite, a second pick of it takes it out. */
function FavouritesPickerDialog({ block, onChange, onClose }: IFavouritesPickerDialogProps) {
    const t: EditorT = useT("user");
    const labels = useEntityLabels();
    const selected = useMemo(() => new Set(block?.entities.map((e) => `${block.kind}:${e.id}`) ?? []), [block]);
    const kinds = useMemo(() => (block ? [block.kind] : []), [block]);
    const pick = (entity: ITierEntity, server: string | null) => {
        if (!block) return;
        onChange({ ...block, entities: toggleFavourite(block.entities, entity, server) });
    };
    return (
        <Dialog open={block !== null} onOpenChange={(next) => !next && onClose()}>
            <DialogPopup className="h-[min(85dvh,760px)] sm:max-w-3xl">
                <DialogHeader>
                    <DialogTitle>{block ? t("profile.showcase.picker.title", { kind: labels.plural(block.kind) }) : ""}</DialogTitle>
                    <DialogDescription>{t("profile.showcase.picker.description", { max: SHOWCASE_MAX_IDS })}</DialogDescription>
                </DialogHeader>
                {block && <EntityPickerBody kinds={kinds} current={null} selected={selected} onPick={pick} />}
                <DialogFooter className="items-center justify-between sm:justify-between">
                    <span className="font-mono text-muted-foreground text-xs tabular-nums">{t("profile.showcase.picker.count", { count: block?.entities.length ?? 0, max: SHOWCASE_MAX_IDS })}</span>
                    <DialogClose render={<Button type="button" />}>{t("profile.showcase.picker.done")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

type AddStep = "type" | ShowcaseBlockType;

interface IAddBlockDialogProps {
    open: boolean;
    uid: string;
    draft: readonly ShowcaseDraftBlock[];
    onClose: () => void;
    onAdd: (block: ShowcaseDraftBlock) => void;
}

function AddBlockDialog({ open, uid, draft, onClose, onAdd }: IAddBlockDialogProps) {
    const t: EditorT = useT("user");
    const [step, setStep] = useState<AddStep>("type");
    const close = () => {
        onClose();
        setStep("type");
    };
    const add = (block: ShowcaseDraftBlock) => {
        onAdd(block);
        setStep("type");
    };
    const heading = {
        type: t("profile.showcase.add.title"),
        favourites: t("profile.showcase.add.kindTitle"),
        grid: t("profile.showcase.add.gridTitle"),
        tier_list: t("profile.showcase.add.tierListTitle"),
        plan: t("profile.showcase.add.planTitle"),
    }[step];

    return (
        <Dialog open={open} onOpenChange={(next) => !next && close()}>
            <DialogPopup className="sm:max-w-xl">
                <DialogHeader>
                    <DialogTitle>{heading}</DialogTitle>
                    {step === "type" && <DialogDescription>{t("profile.showcase.add.description")}</DialogDescription>}
                </DialogHeader>
                <DialogPanel>
                    {step === "type" && <TypeStep onChoose={setStep} />}
                    {step === "favourites" && <KindStep onChoose={(kind) => add(newFavourites(kind))} />}
                    {(step === "grid" || step === "tier_list") && <ListStep type={step} draft={draft} onAdd={add} />}
                    {step === "plan" && <PlanStep uid={uid} draft={draft} onAdd={add} />}
                </DialogPanel>
                <DialogFooter className="justify-between sm:justify-between">
                    {step !== "type" ? (
                        <Button type="button" variant="ghost" onClick={() => setStep("type")}>
                            <ArrowLeftIcon />
                            {t("profile.showcase.add.back")}
                        </Button>
                    ) : (
                        <span />
                    )}
                    <DialogClose render={<Button type="button" variant="outline" />}>{t("profile.showcase.add.close")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

const TYPE_ORDER: readonly ShowcaseBlockType[] = ["favourites", "grid", "tier_list", "plan"];

function TypeStep({ onChoose }: { onChoose: (type: ShowcaseBlockType) => void }) {
    const t: EditorT = useT("user");
    const text = typeText(t);
    return (
        <ul className="m-0 grid list-none grid-cols-1 gap-2 p-0 sm:grid-cols-2">
            {TYPE_ORDER.map((type) => {
                const option = { type, ...text[type] };
                const Icon = BLOCK_ICON[option.type];
                return (
                    <li key={option.type}>
                        <button
                            type="button"
                            onClick={() => onChoose(option.type)}
                            className="flex h-full w-full cursor-pointer items-start gap-3 rounded-xl border border-border bg-background/60 p-3 text-start transition-colors hover:border-foreground/20 hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                        >
                            <span aria-hidden="true" className="flex size-8 shrink-0 items-center justify-center rounded-lg border border-border bg-muted/50" style={{ color: BLOCK_ACCENT[option.type] }}>
                                <Icon className="size-4" />
                            </span>
                            <span className="flex min-w-0 flex-col gap-0.5">
                                <span className="font-medium text-sm">{option.label}</span>
                                <span className="text-muted-foreground text-xs leading-snug">{option.desc}</span>
                            </span>
                        </button>
                    </li>
                );
            })}
        </ul>
    );
}

function KindStep({ onChoose }: { onChoose: (kind: TierEntityKind) => void }) {
    const labels = useEntityLabels();
    return (
        <ul className="m-0 flex list-none flex-wrap gap-1.5 p-0">
            {ALL_ENTITY_KINDS.map((kind) => (
                <li key={kind}>
                    <button
                        type="button"
                        onClick={() => onChoose(kind)}
                        className="inline-flex h-8 cursor-pointer items-center rounded-full border border-border bg-popover px-3 font-medium font-sans text-muted-foreground text-xs leading-none transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                        {labels.plural(kind)}
                    </button>
                </li>
            ))}
        </ul>
    );
}

/** One of the owner's grids or tier lists, or any by a pasted link, checked to exist before it is added. */
function ListStep({ type, draft, onAdd }: { type: "grid" | "tier_list"; draft: readonly ShowcaseDraftBlock[]; onAdd: (block: ShowcaseDraftBlock) => void }) {
    const t: EditorT = useT("user");
    const server = useGamedataServer();
    const { user } = useAuth();
    const queryClient = useQueryClient();
    const pasteId = useId();
    const [paste, setPaste] = useState("");
    const [error, setError] = useState<string | null>(null);
    const [checking, setChecking] = useState(false);
    const grids = useQuery({ ...myGridsQueryOptions(user?.id ?? null, server), enabled: type === "grid" && Boolean(user) });
    const lists = useQuery({ ...myTierListsQueryOptions(Boolean(user)), enabled: type === "tier_list" && Boolean(user) });
    const held = new Set(draft.filter((b) => b.type === type && !b.removed).map((b) => (b.type === "grid" || b.type === "tier_list" ? b.slug : "")));
    const options = type === "grid" ? (grids.data ?? []).map((g) => ({ slug: g.slug, title: g.title, meta: t("profile.showcase.add.size", { rows: g.rows, cols: g.cols }) })) : (lists.data ?? []).map((l) => ({ slug: l.slug, title: l.name, meta: null as string | null }));
    const pending = type === "grid" ? grids.isPending : lists.isPending;
    const make = (slug: string): ShowcaseDraftBlock => (type === "grid" ? { key: draftKey(), removed: false, type: "grid", slug } : { key: draftKey(), removed: false, type: "tier_list", slug });

    const addPasted = async () => {
        const slug = slugFromInput(paste, type);
        if (!slug) {
            setError(type === "grid" ? t("profile.showcase.add.notALink") : t("profile.showcase.add.notATierListLink"));
            return;
        }
        setChecking(true);
        setError(null);
        try {
            const found = type === "grid" ? await queryClient.fetchQuery(gridQueryOptions(slug, user?.id ?? null, server)) : await queryClient.fetchQuery(tierListDetailQueryOptions(slug, server));
            if (!found) setError(t("profile.showcase.add.notFound"));
            else onAdd(make(found.slug));
        } catch {
            setError(t("profile.showcase.add.notFound"));
        } finally {
            setChecking(false);
        }
    };

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
                <span className="kicker">{t("profile.showcase.add.yours")}</span>
                {pending ? (
                    <div className="h-16 animate-pulse rounded-xl bg-muted/40" />
                ) : options.length === 0 ? (
                    <p className="m-0 text-muted-foreground text-sm">{type === "grid" ? t("profile.showcase.add.noGrids") : t("profile.showcase.add.noTierLists")}</p>
                ) : (
                    <ul className="m-0 flex max-h-64 list-none flex-col gap-1 overflow-y-auto p-0">
                        {options.map((o) => {
                            const added = held.has(o.slug);
                            return (
                                <li key={o.slug}>
                                    <button
                                        type="button"
                                        disabled={added}
                                        onClick={() => onAdd(make(o.slug))}
                                        className="flex w-full cursor-pointer items-center gap-3 rounded-lg border border-transparent px-3 py-2 text-start transition-colors hover:border-border hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:opacity-60 disabled:hover:border-transparent disabled:hover:bg-transparent"
                                    >
                                        <span className="min-w-0 flex-1 truncate font-medium text-sm">{o.title}</span>
                                        {o.meta && <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{o.meta}</span>}
                                        {added && <span className="shrink-0 text-muted-foreground text-xs">{t("profile.showcase.add.added")}</span>}
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                )}
            </div>
            <form
                className="flex flex-col gap-1.5"
                onSubmit={(e) => {
                    e.preventDefault();
                    void addPasted();
                }}
            >
                <label htmlFor={pasteId} className="font-medium text-sm">
                    {t("profile.showcase.add.pasteLabel")}
                </label>
                <div className="flex gap-2">
                    <Input
                        id={pasteId}
                        value={paste}
                        placeholder={t("profile.showcase.add.pastePlaceholder")}
                        aria-invalid={error !== null || undefined}
                        onChange={(e) => {
                            setPaste((e.target as HTMLInputElement).value);
                            setError(null);
                        }}
                    />
                    <Button type="submit" variant="outline" disabled={!paste.trim()} loading={checking}>
                        {t("profile.showcase.add.pasteAdd")}
                    </Button>
                </div>
                {checking && <p className="m-0 text-muted-foreground text-xs">{t("profile.showcase.add.checking")}</p>}
                {error && <p className="m-0 text-destructive text-xs">{error}</p>}
            </form>
        </div>
    );
}

function PlanStep({ uid, draft, onAdd }: { uid: string; draft: readonly ShowcaseDraftBlock[]; onAdd: (block: ShowcaseDraftBlock) => void }) {
    const t: EditorT = useT("user");
    const operatorName = useOperatorName();
    const { data: plans, isPending } = useQuery(publicPlansQueryOptions(uid));
    const held = new Set(draft.filter((b) => b.type === "plan" && !b.removed).map((b) => (b.type === "plan" ? b.id : "")));
    if (isPending) return <div className="h-16 animate-pulse rounded-xl bg-muted/40" />;
    const shown = (plans ?? []).filter((p) => p.operator);
    if (shown.length === 0) return <p className="m-0 text-muted-foreground text-sm">{t("profile.showcase.add.noPlans")}</p>;
    return (
        <ul className="m-0 flex max-h-80 list-none flex-col gap-1 overflow-y-auto p-0">
            {shown.map((p) => {
                const added = held.has(p.id);
                const op = p.operator;
                return (
                    <li key={p.id}>
                        <button
                            type="button"
                            disabled={added}
                            onClick={() => onAdd({ key: draftKey(), removed: false, type: "plan", id: p.id, operatorId: p.operator_id })}
                            className="flex w-full cursor-pointer items-center gap-3 rounded-lg border border-transparent px-2 py-1.5 text-start transition-colors hover:border-border hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:opacity-60 disabled:hover:border-transparent disabled:hover:bg-transparent"
                        >
                            <span aria-hidden="true" className="relative flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border bg-muted/70">
                                <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" server={op.server} />
                            </span>
                            <span className="min-w-0 flex-1 truncate font-medium text-sm">{operatorName(op)}</span>
                            {added && <span className="shrink-0 text-muted-foreground text-xs">{t("profile.showcase.add.added")}</span>}
                        </button>
                    </li>
                );
            })}
        </ul>
    );
}
