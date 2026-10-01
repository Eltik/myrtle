/**
 * THE EXPORT SHEET: one story, the chapter, or a hand-picked set of its
 * stories, as an EPUB, Markdown or plain text, with the size estimated before
 * anything is fetched and per-story progress while it is built.
 *
 * A Dialog above 640 px and a bottom Sheet under it, the rule the chapter
 * sheet and the operator panel use. The format, pictures, branches and
 * typeface choices persist per browser under `myrtle.story.export.*`; the
 * scope does not, because it is a fact about where the sheet was opened from.
 * The Doctor's name starts from the reader's own setting every time.
 *
 * Nothing is fetched to estimate: words come from the index and pictures from
 * the chapter's illustrations listing (the one the Illustrations tab reads,
 * usually already cached), with the index's own count as the fallback.
 */
import { Radio as RadioPrimitive } from "@base-ui/react/radio";
import { RadioGroup as RadioGroupPrimitive } from "@base-ui/react/radio-group";
import { useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import type React from "react";
import { useEffect, useMemo, useRef, useState } from "react";
import { Button } from "#/components/ui/button";
import { Checkbox } from "#/components/ui/checkbox";
import { Dialog, DialogDescription, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Progress } from "#/components/ui/progress";
import { Sheet, SheetDescription, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { useMediaQuery } from "#/hooks/use-media-query";
import { storyIllustrationsQueryOptions, storyIndexQueryOptions } from "#/lib/api/story";
import { useFormatters, useGamedataServer, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type BookGroup, type BookIndex, scopeFileName, storyLabel } from "#/lib/story/book/book";
import { type BookFormat, estimateBook } from "#/lib/story/book/estimate";
import type { PaperSize } from "#/lib/story/book/pdf";
import { PdfStoryError } from "#/lib/story/book/pdfError";
import { type BookLabels, DEFAULT_LABELS } from "#/lib/story/book/render";
import { arcScope, rangeScope, storylineScope } from "#/lib/story/book/scopes";
import type { BookScope, BranchMode, ImageMode, Typeface } from "#/lib/story/book/types";
import { loadProgress } from "#/lib/story/progress";
import { humanTime, useReadingSpeed } from "#/lib/story/reading";
import { NICKNAME_MAX, resolveNickname, useStorySettings } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import type { LibGroup } from "../library/impl/derive";
import { READING_ORDER_MODES, type ReadingOrderMode, readingOrder } from "../library/impl/readingOrder";
import type { messages } from "./export.messages";
import { BLOB_WARN_BYTES, type ExportProgress, type PartialFile, pickTarget, runExport, saveBlob, savePicker } from "./run";
import { ExportStallError } from "./watchdog";

type ExportT = TypedT<typeof messages>;

export type ExportScopeKind = "story" | "chapter" | "selection" | "arc" | "storyline" | "range";

const FORMATS: readonly BookFormat[] = ["epub", "pdf", "markdown", "text"];
const PAPERS: readonly PaperSize[] = ["A5", "A4", "LETTER"];
/** The orders a range can be cut from: the two the library derives (the timeline has no data). */
const RANGE_MODES = READING_ORDER_MODES.filter((m): m is Exclude<ReadingOrderMode, "timeline"> => m !== "timeline");
/** Above this many groups the sheet stops fetching illustration listings for the estimate and falls back to the index's counts. */
const LISTING_LIMIT = 20;
const IMAGE_MODES: readonly ImageMode[] = ["cg", "cg+bg", "none"];
const BRANCH_MODES: readonly BranchMode[] = ["all", "path"];
const TYPEFACES: readonly Typeface[] = ["inter", "opendyslexic", "device"];

export const EXPORT_KEYS = {
    format: "myrtle.story.export.format",
    images: "myrtle.story.export.images",
    branches: "myrtle.story.export.branches",
    typeface: "myrtle.story.export.typeface",
    paper: "myrtle.story.export.paper",
} as const;

function oneOf<T extends string>(allowed: readonly T[]): { parse: (raw: string) => T | undefined; serialize: (value: T) => string } {
    return { parse: (raw) => ((allowed as readonly string[]).includes(raw) ? (raw as T) : undefined), serialize: (v) => v };
}

export interface IExportSheetProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    /** The chapter (or event) the scopes are drawn from. */
    group: BookGroup;
    /** The story open in the reader; offers "This story" and makes it the default scope. */
    storyId?: string;
}

type RunState = { kind: "idle" } | { kind: "running"; progress: ExportProgress | null } | { kind: "done"; file: string; missing: number; toDisk: boolean } | { kind: "cancelled"; partial: PartialFile | null } | { kind: "error"; message: string; partial: PartialFile | null };

export function ExportSheet({ open, onOpenChange, group, storyId }: IExportSheetProps): React.ReactElement {
    const t: ExportT = useT("story");
    const phone = useMediaQuery("max-sm");
    const running = useRef<AbortController | null>(null);
    const close = (next: boolean) => {
        if (!next) running.current?.abort();
        onOpenChange(next);
    };
    const body = <ExportBody group={group} storyId={storyId} open={open} running={running} />;
    if (phone) {
        return (
            <Sheet open={open} onOpenChange={close}>
                <SheetPopup side="bottom" className="max-h-[92dvh] bg-card" closeProps={{ className: "absolute end-2 top-2 size-11" }}>
                    <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-4 pt-4 pb-4">
                        <SheetTitle className="pe-12 font-semibold text-lg">{t("export.title")}</SheetTitle>
                        <SheetDescription className="mt-1 pe-12 text-muted-foreground text-sm">{t("export.subtitle", { group: group.name })}</SheetDescription>
                        {body}
                    </div>
                </SheetPopup>
            </Sheet>
        );
    }
    return (
        <Dialog open={open} onOpenChange={close}>
            <DialogPopup className="max-w-lg bg-card">
                <div className="max-h-[88dvh] overflow-y-auto overscroll-contain px-6 pt-5 pb-5">
                    <DialogTitle className="pe-10 font-semibold text-lg">{t("export.title")}</DialogTitle>
                    <DialogDescription className="mt-1 pe-10 text-muted-foreground text-sm">{t("export.subtitle", { group: group.name })}</DialogDescription>
                    {body}
                </div>
            </DialogPopup>
        </Dialog>
    );
}

function ExportBody({ group, storyId, open, running }: { group: BookGroup; storyId?: string; open: boolean; running: React.RefObject<AbortController | null> }): React.ReactElement {
    const t: ExportT = useT("story");
    const f = useFormatters();
    const locale = useLocale();
    const server = useGamedataServer();
    const queryClient = useQueryClient();
    const { wpm } = useReadingSpeed();
    const [settings] = useStorySettings();
    const readable = useMemo(() => [...group.stories].filter((s) => s.hasScript).sort((a, b) => a.sort - b.sort), [group.stories]);

    const [scopeKind, setScopeKind] = useState<ExportScopeKind>(storyId ? "story" : "chapter");
    const [selected, setSelected] = useState<ReadonlySet<string>>(() => new Set(readable.map((s) => s.id)));
    const [format, setFormat] = useLocalStorageState<BookFormat>(EXPORT_KEYS.format, "epub", oneOf(FORMATS));
    const [images, setImages] = useLocalStorageState<ImageMode>(EXPORT_KEYS.images, "cg", oneOf(IMAGE_MODES));
    const [branches, setBranches] = useLocalStorageState<BranchMode>(EXPORT_KEYS.branches, "all", oneOf(BRANCH_MODES));
    const [typeface, setTypeface] = useLocalStorageState<Typeface>(EXPORT_KEYS.typeface, "inter", oneOf(TYPEFACES));
    const [paper, setPaper] = useLocalStorageState<PaperSize>(EXPORT_KEYS.paper, "A5", oneOf(PAPERS));
    const [rangeMode, setRangeMode] = useState<(typeof RANGE_MODES)[number]>("release");
    const [rangeFrom, setRangeFrom] = useState(group.id);
    const [rangeTo, setRangeTo] = useState(group.id);
    const [name, setName] = useState(settings.nickname);
    const [run, setRun] = useState<RunState>({ kind: "idle" });

    // The name follows the reader's setting each time the sheet opens.
    useEffect(() => {
        if (open) setName(settings.nickname);
    }, [open, settings.nickname]);

    // The whole library, for the arc, storyline and range scopes. Cached by the
    // library page and the reader, so opening the sheet costs no request there.
    const library = useQuery({ ...storyIndexQueryOptions(server), enabled: open });
    const index: BookIndex = useMemo(() => ({ groups: [group, ...(library.data?.groups ?? []).filter((g) => g.id !== group.id)] }), [group, library.data]);
    const storylines = library.data?.storylines ?? [];
    const arc = useMemo(() => arcScope(storylines, group.id), [storylines, group.id]);
    const shelf = useMemo(() => storylineScope(storylines, group.id), [storylines, group.id]);
    // A reading order as the library's tab derives it, flattened to one run of groups.
    const order = useMemo(() => (library.data ? readingOrder(rangeMode, library.data.groups as unknown as LibGroup[]).flatMap((sec) => sec.rows.map((r) => r.group)) : []), [library.data, rangeMode]);
    const orderIds = useMemo(() => order.map((g) => g.id), [order]);
    const inOrder = orderIds.includes(group.id);
    const illustrations = useQuery({ ...storyIllustrationsQueryOptions(group.id, server), enabled: open });
    // The whole chapter's art, distinct files that resolve, from the listing the estimate already reads.
    const chapterArt = useMemo(() => {
        const d = illustrations.data;
        if (!d) return null;
        const distinct = (items: typeof d.images) => new Set(items.flatMap((i) => (i.url === null ? [] : [i.url]))).size;
        return { cgs: distinct(d.images), scenes: distinct(d.backgrounds) };
    }, [illustrations.data]);
    // Every key is written out at its call so `i18n:extract` sees it used.
    const book = (out: string, key: string): string | null => (out === key || out === `story.${key}` ? null : out);
    const rangeTitle = (() => {
        const from = order.find((g) => g.id === rangeFrom);
        const to = order.find((g) => g.id === rangeTo);
        // The title goes INTO the book and its file name, so a missing catalogue key falls back to English here too.
        const mode = rangeMode === "release" ? (book(t("export.range.release"), "export.range.release") ?? "Release order") : (book(t("export.range.storyline"), "export.range.storyline") ?? "Storyline order");
        return from && to ? (from.id === to.id ? `${mode}: ${from.name}` : `${mode}: ${from.name} – ${to.name}`) : mode;
    })();
    const scope: BookScope = useMemo(() => {
        if (scopeKind === "story" && storyId) return { kind: "story", storyId };
        if (scopeKind === "selection") return { kind: "selection", ids: readable.filter((s) => selected.has(s.id)).map((s) => s.id) };
        if (scopeKind === "arc" && arc) return arc;
        if (scopeKind === "storyline" && shelf) return shelf;
        if (scopeKind === "range") return rangeScope(rangeMode, orderIds, rangeFrom, rangeTo, rangeTitle);
        return { kind: "group", groupId: group.id };
    }, [scopeKind, storyId, readable, selected, arc, shelf, rangeMode, orderIds, rangeFrom, rangeTo, rangeTitle, group.id]);
    const scopeGroups = scope.kind === "groups" ? scope.ids : [group.id];
    // Listings for every group in scope, up to a limit; past it the estimate says it is rough.
    const listings = useQueries({ queries: (scopeGroups.length <= LISTING_LIMIT ? scopeGroups : []).map((id) => ({ ...storyIllustrationsQueryOptions(id, server), enabled: open })) });
    const listingMap = new Map<string, (typeof illustrations)["data"]>([[group.id, illustrations.data]]);
    (scopeGroups.length <= LISTING_LIMIT ? scopeGroups : []).forEach((id, i) => {
        if (listings[i]?.data) listingMap.set(id, listings[i].data);
    });
    const listingKey = listings.map((l) => (l.data ? 1 : 0)).join("");
    // biome-ignore lint/correctness/useExhaustiveDependencies: `listingKey` stands for the listings that have landed; the map is rebuilt from them.
    const estimate = useMemo(() => estimateBook(index, scope, { images, format, typeface, wpm, illustrations: listingMap }), [index, scope, images, format, typeface, wpm, illustrations.data, listingKey]);

    // The words printed INSIDE the book. A key the catalogue does not carry yet
    // comes back from `t` as the key itself, and a raw key in a reader's file
    // is exactly Timo's colophon defect, so any miss falls back to English.
    const quoted = (options: string[]) => options.map((o) => `“${o}”`).join(" / ");
    const labelTemplates = {
        ifChose: book(t("book.ifChose", { options: "{options}" }), "book.ifChose") ?? "If you chose {options}",
        earlier: book(t("book.earlier", { options: "{options}" }), "book.earlier") ?? "If you had chosen {options}",
    };
    const labels: BookLabels = {
        contents: book(t("book.contents"), "book.contents") ?? DEFAULT_LABELS.contents,
        cover: book(t("book.cover"), "book.cover") ?? DEFAULT_LABELS.cover,
        colophon: book(t("book.colophon"), "book.colophon") ?? DEFAULT_LABELS.colophon,
        choice: book(t("book.choice"), "book.choice") ?? DEFAULT_LABELS.choice,
        ifChose: (options) => book(t("book.ifChose", { options: quoted(options) }), "book.ifChose") ?? DEFAULT_LABELS.ifChose(options),
        earlier: (options) => book(t("book.earlier", { options: quoted(options) }), "book.earlier") ?? DEFAULT_LABELS.earlier(options),
        cutscene: book(t("book.cutscene"), "book.cutscene") ?? DEFAULT_LABELS.cutscene,
        figureAlt: book(t("book.figureAlt"), "book.figureAlt") ?? DEFAULT_LABELS.figureAlt,
        synopsis: book(t("book.synopsis"), "book.synopsis") ?? DEFAULT_LABELS.synopsis,
        credit: book(t("book.credit"), "book.credit") ?? DEFAULT_LABELS.credit,
        madeWith: book(t("book.madeWith"), "book.madeWith") ?? DEFAULT_LABELS.madeWith,
        rights: book(t("book.rights"), "book.rights") ?? DEFAULT_LABELS.rights,
    };

    const busy = run.kind === "running";
    const picker = savePicker() !== null;
    // `?exportstream=0` turns the disk stream off and downloads a Blob, the pre-2026-10-01 path.
    const streamOff = typeof window !== "undefined" && new URLSearchParams(window.location.search).get("exportstream") === "0";
    const pdfWorker = typeof window === "undefined" || new URLSearchParams(window.location.search).get("pdfworker") !== "0";
    // `?pdffail=<story id>`: that story's PDF layout throws, to check the error line end to end.
    const pdfFailStory = typeof window === "undefined" ? undefined : (new URLSearchParams(window.location.search).get("pdffail") ?? undefined);
    const start = async () => {
        // The picker FIRST, while the click still counts as a user gesture.
        let target: FileSystemFileHandle | null = null;
        if (picker && !streamOff) {
            target = await pickTarget(scopeFileName(index, scope, format === "markdown" ? "md" : format === "text" ? "txt" : format), format);
            if (!target) return;
        }
        const controller = new AbortController();
        running.current = controller;
        setRun({ kind: "running", progress: null });
        const pos = loadProgress().pos;
        let partial: PartialFile | null = null;
        try {
            const out = await runExport({
                queryClient,
                server,
                index,
                scope,
                options: { nickname: resolveNickname(name), images, branches, choices: Object.fromEntries(Object.entries(pos).map(([id, p]) => [id, p.choices])) },
                format,
                typeface,
                paper,
                labels,
                labelTemplates,
                target,
                pdfWorker,
                pdfFailStory,
                signal: controller.signal,
                onProgress: (progress) => setRun({ kind: "running", progress }),
                onPartial: (state) => {
                    partial = state;
                },
            });
            if (out.pdf) console.info(`[export] PDF laid out on the ${out.pdf.thread} thread in ${out.pdf.renderMs} ms`);
            if (out.blob) saveBlob(out.blob, out.fileName);
            setRun({ kind: "done", file: out.fileName, missing: out.failedImages.length, toDisk: out.blob === null });
        } catch (err) {
            if (controller.signal.aborted) setRun({ kind: "cancelled", partial });
            else {
                const message =
                    // An error line must say what happened even before `i18n:extract` carries a new key: a raw key falls back to English.
                    err instanceof ExportStallError
                        ? (book(t("export.stalled", { seconds: err.seconds, story: err.story || "…" }), "export.stalled") ?? `The export stopped making progress for ${err.seconds} s at ${err.story || "…"} and was stopped. Try a smaller scope.`)
                        : err instanceof PdfStoryError
                          ? (book(t("export.storyFailed", { story: err.story, message: err.message }), "export.storyFailed") ?? `The export failed at ${err.story}: ${err.message}`)
                          : t("export.error", { message: err instanceof Error ? err.message : String(err) });
                setRun({ kind: "error", message, partial });
            }
        } finally {
            if (running.current === controller) running.current = null;
        }
    };

    const mb = (bytes: number) => new Intl.NumberFormat(locale, { maximumFractionDigits: bytes < 10_000_000 ? 1 : 0 }).format(bytes / 1_000_000);
    const size = estimate.bytes < 1_000_000 ? `${f.number(Math.max(1, Math.round(estimate.bytes / 1000)))} KB` : `${mb(estimate.bytes)} MB`;
    const embedsFont = format === "epub" || format === "pdf";
    // A PDF is always built in memory (pdf-lib writes the merged file in one piece); an EPUB only without a picker.
    const blobWarning = (format === "pdf" || !picker || streamOff) && estimate.bytes > BLOB_WARN_BYTES;

    return (
        <div className="mt-5 flex flex-col gap-5">
            <Field label={t("export.scope")}>
                <Segmented<ExportScopeKind>
                    value={scopeKind}
                    onChange={setScopeKind}
                    disabled={busy}
                    items={[
                        ...(storyId ? [{ value: "story" as const, label: t("export.scope.story") }] : []),
                        { value: "chapter", label: t("export.scope.chapter") },
                        { value: "selection", label: t("export.scope.selection") },
                        ...(arc ? [{ value: "arc" as const, label: t("export.scope.arc") }] : []),
                        ...(shelf ? [{ value: "storyline" as const, label: t("export.scope.storyline") }] : []),
                        ...(library.data ? [{ value: "range" as const, label: t("export.scope.range") }] : []),
                    ]}
                />
                {scopeKind === "arc" && arc?.kind === "groups" ? <p className="text-muted-foreground text-xs">{t("export.scope.groups", { title: arc.title, count: arc.ids.length })}</p> : null}
                {scopeKind === "storyline" && shelf?.kind === "groups" ? <p className="text-muted-foreground text-xs">{t("export.scope.groups", { title: shelf.title, count: shelf.ids.length })}</p> : null}
                {scopeKind === "range" ? (
                    <div className="mt-2 flex flex-col gap-2 rounded-lg border border-border p-3">
                        <Segmented<(typeof RANGE_MODES)[number]>
                            value={rangeMode}
                            onChange={(m) => {
                                setRangeMode(m);
                                setRangeFrom(group.id);
                                setRangeTo(group.id);
                            }}
                            disabled={busy}
                            items={[
                                { value: "release", label: t("export.range.release") },
                                { value: "storyline", label: t("export.range.storyline") },
                            ]}
                        />
                        <RangeSelect label={t("export.range.from")} value={inOrder || rangeFrom !== group.id ? rangeFrom : ""} order={order} disabled={busy} onChange={setRangeFrom} />
                        <RangeSelect label={t("export.range.to")} value={inOrder || rangeTo !== group.id ? rangeTo : ""} order={order} disabled={busy} onChange={setRangeTo} />
                        {scope.kind === "groups" ? <p className="text-muted-foreground text-xs">{t("export.scope.groups", { title: scope.title, count: scope.ids.length })}</p> : null}
                    </div>
                ) : null}
                {scopeKind === "selection" ? (
                    <div className="mt-2 rounded-lg border border-border">
                        <div className="flex items-center justify-between gap-2 border-border border-b px-3 py-1.5 text-muted-foreground text-xs">
                            <span className="tabular-nums">{t("export.selection.count", { count: selected.size })}</span>
                            <span className="flex gap-1">
                                <Button variant="ghost" size="sm" className="max-sm:h-11" disabled={busy} onClick={() => setSelected(new Set(readable.map((s) => s.id)))}>
                                    {t("export.selection.all")}
                                </Button>
                                <Button variant="ghost" size="sm" className="max-sm:h-11" disabled={busy} onClick={() => setSelected(new Set())}>
                                    {t("export.selection.none")}
                                </Button>
                            </span>
                        </div>
                        <ul className="max-h-56 overflow-y-auto py-1">
                            {readable.map((s) => (
                                <li key={s.id}>
                                    {/* biome-ignore lint/a11y/noLabelWithoutControl: the base-ui Checkbox inside is the control. */}
                                    <label className="flex min-h-11 cursor-pointer items-center gap-3 px-3 py-1 text-sm hover:bg-secondary/40 sm:min-h-9">
                                        <Checkbox
                                            checked={selected.has(s.id)}
                                            disabled={busy}
                                            onCheckedChange={(checked: boolean) =>
                                                setSelected((prev) => {
                                                    const next = new Set(prev);
                                                    if (checked) next.add(s.id);
                                                    else next.delete(s.id);
                                                    return next;
                                                })
                                            }
                                        />
                                        <span className="min-w-0 flex-1 truncate">{storyLabel(s)}</span>
                                        {s.avgTag ? <span className="shrink-0 text-muted-foreground text-xs">{s.avgTag}</span> : null}
                                    </label>
                                </li>
                            ))}
                        </ul>
                    </div>
                ) : null}
            </Field>

            <Field label={t("export.format")}>
                <Segmented<BookFormat>
                    value={format}
                    onChange={setFormat}
                    disabled={busy}
                    items={[
                        { value: "epub", label: t("export.format.epub") },
                        { value: "pdf", label: t("export.format.pdf") },
                        { value: "markdown", label: t("export.format.markdown") },
                        { value: "text", label: t("export.format.text") },
                    ]}
                />
            </Field>

            {format === "pdf" ? (
                <Field label={t("export.paper")}>
                    <Segmented<PaperSize>
                        value={paper}
                        onChange={setPaper}
                        disabled={busy}
                        items={[
                            { value: "A5", label: t("export.paper.a5") },
                            { value: "A4", label: t("export.paper.a4") },
                            { value: "LETTER", label: t("export.paper.letter") },
                        ]}
                    />
                </Field>
            ) : null}

            <Field label={t("export.images")} hint={[chapterArt ? t("export.images.count", chapterArt) : null, embedsFont ? null : t("export.images.linked")].filter(Boolean).join(" ") || undefined}>
                <Segmented<ImageMode>
                    value={images}
                    onChange={setImages}
                    disabled={busy}
                    items={[
                        { value: "cg", label: t("export.images.cg") },
                        { value: "cg+bg", label: t("export.images.cgbg") },
                        { value: "none", label: t("export.images.none") },
                    ]}
                />
            </Field>

            <Field label={t("export.branches")}>
                <Segmented<BranchMode>
                    value={branches}
                    onChange={setBranches}
                    disabled={busy}
                    items={[
                        { value: "all", label: t("export.branches.all") },
                        { value: "path", label: t("export.branches.path") },
                    ]}
                />
            </Field>

            <Field label={t("export.name")} htmlFor="export-name">
                <Input id="export-name" value={name} maxLength={NICKNAME_MAX} placeholder="Doctor" disabled={busy} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setName(e.target.value)} className="max-w-72 max-sm:h-11 max-sm:max-w-none max-sm:[&_[data-slot=input]]:h-11" />
            </Field>

            <Field label={t("export.typeface")} hint={embedsFont ? undefined : t("export.typeface.packagedOnly")}>
                <Segmented<Typeface>
                    value={typeface}
                    onChange={setTypeface}
                    disabled={busy || !embedsFont}
                    items={[
                        { value: "inter", label: t("export.typeface.inter") },
                        { value: "opendyslexic", label: t("export.typeface.opendyslexic") },
                        { value: "device", label: t("export.typeface.device") },
                    ]}
                />
            </Field>

            <div className="flex flex-col gap-3 border-border border-t pt-4">
                <p className="text-sm tabular-nums" data-export-estimate aria-live="polite">
                    {estimate.stories === 0 ? t("export.empty") : t("export.estimate", { stories: estimate.stories, words: estimate.words, time: humanTime(estimate.minutes), images: estimate.images, size })}
                </p>
                {estimate.stories > 0 && !estimate.exactImages ? <p className="-mt-2 text-muted-foreground text-xs">{t("export.estimate.rough")}</p> : null}
                {blobWarning ? (
                    <output className="text-amber-700 text-xs dark:text-amber-400" data-export-blob-warning>
                        {t("export.blobWarning", { size })}
                    </output>
                ) : null}
                <Status run={run} />
                <div className="flex flex-wrap items-center justify-end gap-2">
                    {storyId && !busy ? (
                        <Button variant="ghost" className="max-sm:h-11 sm:me-auto" render={<Link to="/stories/$storyId" params={{ storyId }} search={{ view: "book" }} />} data-export-print>
                            {t("export.book.open")}
                        </Button>
                    ) : null}
                    {busy ? (
                        <Button variant="outline" className="max-sm:h-11 max-sm:flex-1" onClick={() => running.current?.abort()}>
                            {t("export.cancel")}
                        </Button>
                    ) : null}
                    <Button className="max-sm:h-11 max-sm:flex-1" disabled={busy || estimate.stories === 0} onClick={() => void start()} data-export-download>
                        {t("export.download")}
                    </Button>
                </div>
            </div>
        </div>
    );
}

function Status({ run }: { run: RunState }): React.ReactElement | null {
    const t: ExportT = useT("story");
    if (run.kind === "idle") return null;
    if (run.kind === "running") {
        const p = run.progress;
        // Scripts are the first 40% of the bar, the stories and their pictures the rest; the PDF's layout is the last step and has no count.
        const value = !p || p.total === 0 ? 0 : p.phase === "scripts" ? (0.4 * p.done) / p.total : p.phase === "merge" ? 1 : 0.4 + (0.6 * p.done) / p.total;
        const line = !p
            ? ""
            : p.phase === "scripts"
              ? t("export.progress.scripts", { done: Math.min(p.done + 1, p.total), total: p.total, story: p.label })
              : p.phase === "merge"
                ? t("export.progress.finishing")
                : p.phase === "layout"
                  ? p.label === ""
                      ? t("export.progress.layout")
                      : t("export.progress.layoutStory", { done: Math.min(p.done + 1, p.total), total: p.total, story: p.label })
                  : p.label === ""
                    ? t("export.progress.finishing")
                    : t("export.progress.book", { done: p.done + 1, total: p.total, story: p.label });
        return (
            <div className="flex flex-col gap-1.5" data-export-progress>
                <Progress value={Math.round(value * 100)} aria-label={line} />
                <p className="truncate text-muted-foreground text-xs tabular-nums" aria-live="polite">
                    {line}
                </p>
            </div>
        );
    }
    const partialLine = (partial: PartialFile | null) => (partial === "removed" ? ` ${t("export.partial.removed")}` : partial === "kept" ? ` ${t("export.partial.kept")}` : "");
    if (run.kind === "done") {
        const saved = run.toDisk ? t("export.doneDisk", { file: run.file }) : t("export.done", { file: run.file });
        return (
            <p className="text-muted-foreground text-xs" data-export-done>
                {run.missing > 0 ? `${saved} ${t("export.doneMissing", { count: run.missing })}` : saved}
            </p>
        );
    }
    if (run.kind === "cancelled")
        return (
            <p className="text-muted-foreground text-xs" data-export-cancelled>
                {t("export.cancelled")}
                {partialLine(run.partial)}
            </p>
        );
    return (
        <p className="text-destructive text-xs" role="alert" data-export-error>
            {run.message}
            {partialLine(run.partial)}
        </p>
    );
}

/** A native select over one reading order's groups: long lists scroll and type-ahead for free, and it is 44 px on a phone. */
function RangeSelect({ label, value, order, disabled, onChange }: { label: string; value: string; order: readonly { id: string; name: string }[]; disabled?: boolean; onChange: (id: string) => void }): React.ReactElement {
    return (
        <label className="flex flex-col gap-1 text-muted-foreground text-xs sm:flex-row sm:items-center sm:gap-3">
            <span className="w-10 shrink-0">{label}</span>
            <select value={value} disabled={disabled} onChange={(e) => onChange(e.target.value)} className="h-11 min-w-0 flex-1 rounded-md border border-input bg-background px-2 text-foreground text-sm sm:pointer-fine:h-8">
                {value === "" ? <option value="">…</option> : null}
                {order.map((g) => (
                    <option key={g.id} value={g.id}>
                        {g.name}
                    </option>
                ))}
            </select>
        </label>
    );
}

/**
 * One question on the sheet. A choice is a FIELDSET whose legend names the
 * radio group inside it; the name field is a plain label for its input.
 */
function Field({ label, hint, htmlFor, children }: { label: string; hint?: string; htmlFor?: string; children: React.ReactNode }): React.ReactElement {
    const caption = "font-medium text-muted-foreground text-xs uppercase tracking-wide";
    const hintLine = hint ? <p className="text-muted-foreground text-xs">{hint}</p> : null;
    if (htmlFor) {
        return (
            <div className="flex flex-col gap-1.5">
                <label htmlFor={htmlFor} className={caption}>
                    {label}
                </label>
                {children}
                {hintLine}
            </div>
        );
    }
    return (
        <fieldset className="m-0 flex min-w-0 flex-col gap-1.5 border-0 p-0">
            <legend className={cn(caption, "mb-1.5 p-0")}>{label}</legend>
            {children}
            {hintLine}
        </fieldset>
    );
}

interface SegmentItem<T extends string> {
    value: T;
    label: string;
    tag?: string;
    disabled?: boolean;
}

/**
 * A row of mutually exclusive choices: a radio group drawn as the segmented
 * control the chapter sheet's view switch uses, so arrow keys move within it
 * and a screen reader hears one question with its answers. 44 px tall on a
 * coarse pointer or a phone, 28 px on a fine one.
 */
function Segmented<T extends string>({ value, onChange, items, disabled }: { value: T; onChange: (v: T) => void; items: SegmentItem<T>[]; disabled?: boolean }): React.ReactElement {
    return (
        <RadioGroupPrimitive value={value} onValueChange={(v: unknown) => onChange(v as T)} disabled={disabled} className="flex w-fit max-w-full flex-wrap gap-0.5 rounded-[9px] border border-border bg-secondary/45 p-0.75">
            {items.map((item) => (
                <RadioPrimitive.Root
                    key={item.value}
                    value={item.value}
                    disabled={item.disabled}
                    className={cn(
                        "inline-flex h-11 shrink-0 cursor-pointer items-center rounded-md px-3 font-sans font-semibold text-[12px] text-muted-foreground outline-none transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring sm:pointer-fine:h-7",
                        "data-checked:bg-background data-checked:text-foreground data-checked:shadow-sm/5",
                        "data-disabled:cursor-not-allowed data-disabled:opacity-50 data-disabled:hover:text-muted-foreground",
                    )}
                >
                    {item.label}
                    {item.tag ? <span className="ms-1.5 font-normal text-[10.5px] text-muted-foreground">{item.tag}</span> : null}
                </RadioPrimitive.Root>
            ))}
        </RadioGroupPrimitive>
    );
}
