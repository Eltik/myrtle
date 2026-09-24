import { useQuery } from "@tanstack/react-query";
import { ChevronDownIcon, PauseIcon, PlayIcon } from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { asset, audioURL } from "#/components/operators/detail/impl/assets";
import { VOICE_LANGUAGE_LABEL_KEY, VOICE_LANGUAGE_ORDER } from "#/components/operators/detail/impl/constants";
import type { messages as detailConstantsMessages } from "#/components/operators/detail/impl/constants.messages";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Dialog, DialogClose, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Sheet, SheetClose, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Skeleton } from "#/components/ui/skeleton";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { useMediaQuery } from "#/hooks/use-media-query";
import { operatorQueryOptions } from "#/lib/api/operators";
import { operatorVoicesQueryOptions } from "#/lib/api/voices";
import { GameText } from "#/lib/gamedata/GameText";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { cn, formatProfession, getAvatarById, RARITY_HEX, rarityToNumber } from "#/lib/utils";
import type { LangType } from "#/types/voices";
import { type LibRecord, sortedStories } from "./derive";
import type { messages } from "./OperatorDialog.messages";
import type { messages as tabMessages } from "./OperatorsTab.messages";
import { fileSections, type IFileSection, type IModuleSection, type IVoiceLine, moduleSections, type OperatorTabKey, operatorTabs, operatorVoiceLines, voiceLanguages } from "./operatorText";
import { StoryRow } from "./StoryRow";

type DialogT = TypedT<typeof messages & typeof tabMessages>;
/** The voice languages are named in the OPERATORS catalog, so that namespace answers for them. */
type LanguageT = TypedT<typeof detailConstantsMessages>;

export interface IOperatorDialogProps {
    record: LibRecord | null;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    onClose: () => void;
}

/**
 * One operator's four shelves: the Records the library index already carries,
 * and the handbook Files, the Modules and the Voice Lines, which each ride on
 * their own endpoint.
 *
 * Every shelf but Records is LAZY. `/operators/{id}` is 41 KB and
 * `/voices/{id}` is 83 KB for Texas, so fetching either for all 315 record
 * sets to print a count on a card would cost 39 MB; the queries are enabled the
 * first time their tab is opened and the tab labels carry a count only from
 * then on, which is why the operator card shows the read fraction and nothing
 * else.
 */
export function OperatorDialog({ record, progress, gameRead, onClose }: IOperatorDialogProps): React.ReactElement | null {
    const t: DialogT = useT("story");
    const phone = useMediaQuery("max-sm");
    const server = useGamedataServer();
    const charId = record?.charId ?? "";

    // Sticky within one operator: a shelf opened once keeps its data and its
    // count in the tab label after the reader moves to another tab. The caller
    // KEYS this component on the operator, so opening the next one remounts it
    // and no state carries Texas's files over to Exusiai's tab.
    const [tab, setTab] = useState<OperatorTabKey>("records");
    const [wantsDetail, setWantsDetail] = useState(false);
    const [wantsVoices, setWantsVoices] = useState(false);

    const openTab = useCallback((next: OperatorTabKey) => {
        setTab(next);
        if (next === "files" || next === "modules") setWantsDetail(true);
        if (next === "voices") setWantsVoices(true);
    }, []);

    const detail = useQuery({ ...operatorQueryOptions(charId, server), enabled: charId !== "" && wantsDetail });
    const voices = useQuery({ ...operatorVoicesQueryOptions(charId, server), enabled: charId !== "" && wantsVoices });

    // `isSuccess`, never `data`: `/operators/{id}` answers 404 with `undefined`
    // for a record set whose character the operator table does not carry, and
    // keying on the data would leave that shelf loading forever instead of
    // saying it is empty.
    const files = useMemo(() => (detail.isSuccess ? fileSections(detail.data?.handbook?.storyTextAudio) : null), [detail.isSuccess, detail.data]);
    const modules = useMemo(() => (detail.isSuccess ? moduleSections(detail.data?.modules) : null), [detail.isSuccess, detail.data]);
    const lines = useMemo(() => (voices.isSuccess ? operatorVoiceLines(voices.data?.charWords, charId) : null), [voices.isSuccess, voices.data, charId]);

    const tabs = useMemo(() => operatorTabs({ records: record?.stories.length ?? 0, files, modules, voices: lines }), [record, files, modules, lines]);
    const shelf = useCallback((key: OperatorTabKey) => tabs.find((entry) => entry.key === key) ?? { key, count: null, words: null }, [tabs]);

    if (!record) return null;

    const rarity = record.rarity ? rarityToNumber(record.rarity) : null;
    const tone = rarity !== null ? (RARITY_HEX[rarity] ?? "var(--border)") : "var(--border)";
    const avatar = record.avatarUrl ? asset(record.avatarUrl) : getAvatarById(record.charId);

    const head = (
        <div className="flex items-center gap-3">
            <span className="block size-11 shrink-0 overflow-hidden rounded-[9px] bg-secondary/60" style={{ borderBottom: `3px solid ${tone}` }}>
                <img src={avatar} alt="" aria-hidden="true" loading="lazy" decoding="async" className="h-full w-full object-cover" />
            </span>
            <span className="flex min-w-0 flex-col">
                <span className="truncate font-sans font-semibold text-[15px] text-foreground">{record.name}</span>
                <span className="flex items-center gap-2 font-mono text-[10.5px] text-muted-foreground">
                    {rarity !== null ? (
                        <span role="img" aria-label={t("operators.rarity", { count: rarity })}>
                            {"★".repeat(rarity)}
                        </span>
                    ) : null}
                    {record.profession ? <span>{formatProfession(record.profession)}</span> : null}
                </span>
            </span>
        </div>
    );

    const body = (
        <Tabs value={tab} onValueChange={(value) => openTab(String(value) as OperatorTabKey)} className="gap-0">
            <TabsList variant="underline" className="msv-scroll w-full max-w-full justify-start overflow-x-auto border-border border-b pb-0">
                {tabs.map((entry) => (
                    <TabsTab key={entry.key} value={entry.key} className="grow-0 px-3 max-sm:min-h-11">
                        {t(`operators.dialog.tab.${entry.key}`)}
                        {entry.count !== null ? <span className="ms-1 font-mono text-[10px] text-muted-foreground tabular-nums">{entry.count}</span> : null}
                    </TabsTab>
                ))}
            </TabsList>

            <TabsPanel value="records" className="pt-3">
                <ul className="m-0 flex list-none flex-col p-0">
                    {sortedStories(record.stories).map((story) => (
                        <li key={story.id}>
                            <StoryRow story={story} progress={progress} gameRead={gameRead} showCode={false} />
                        </li>
                    ))}
                </ul>
                <p className="mt-3 font-sans text-[11.5px] text-muted-foreground">{t("operators.dialog.lazyNote")}</p>
            </TabsPanel>

            <TabsPanel value="files" className="pt-3">
                <Shelf loading={detail.isLoading} failed={detail.isError} rows={files} count={shelf("files").count} words={shelf("files").words} empty={t("operators.dialog.files.empty")}>
                    {(sections: IFileSection[]) => sections.map((section) => <FileRow key={section.key} section={section} />)}
                </Shelf>
            </TabsPanel>

            <TabsPanel value="modules" className="pt-3">
                <Shelf loading={detail.isLoading} failed={detail.isError} rows={modules} count={shelf("modules").count} words={shelf("modules").words} empty={t("operators.dialog.modules.empty")} emptyNote={t("operators.dialog.modules.emptyNote")}>
                    {(sections: IModuleSection[]) => sections.map((section) => <ModuleRow key={section.key} section={section} />)}
                </Shelf>
            </TabsPanel>

            <TabsPanel value="voices" className="pt-3">
                <VoicesShelf loading={voices.isLoading} failed={voices.isError} lines={lines} words={shelf("voices").words} />
            </TabsPanel>
        </Tabs>
    );

    if (phone) {
        return (
            <Sheet open onOpenChange={(open) => !open && onClose()}>
                <SheetPopup side="bottom" className="max-h-[88dvh]" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                    <SheetHeader>
                        <SheetTitle render={<div />}>{head}</SheetTitle>
                    </SheetHeader>
                    <SheetPanel className="overflow-y-auto">{body}</SheetPanel>
                    <SheetFooter>
                        <SheetClose render={<Button variant="outline" className="min-h-11 w-full" />}>{t("operators.dialog.close")}</SheetClose>
                    </SheetFooter>
                </SheetPopup>
            </Sheet>
        );
    }

    return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
            <DialogPopup className="max-w-2xl" closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                <DialogHeader>
                    <DialogTitle render={<div />}>{head}</DialogTitle>
                </DialogHeader>
                <DialogPanel className="max-h-[65dvh] overflow-y-auto">{body}</DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button variant="outline" />}>{t("operators.dialog.close")}</DialogClose>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}

/** Loading, failure, empty and the summary line, so all three text shelves read the same. */
function Shelf<T>({ loading, failed, rows, count, words, empty, emptyNote, children }: { loading: boolean; failed: boolean; rows: T[] | null; count: number | null; words: number | null; empty: string; emptyNote?: string; children: (rows: T[]) => React.ReactNode }): React.ReactElement {
    const t: DialogT = useT("story");
    const f = useFormatters();

    if (failed) return <p className="py-8 text-center font-sans text-[13px] text-muted-foreground">{t("operators.dialog.failed")}</p>;
    if (loading || rows === null) {
        return (
            <output className="flex flex-col gap-2" aria-label={t("operators.dialog.loading")}>
                {[0, 1, 2].map((i) => (
                    <Skeleton key={i} className="h-11 w-full rounded-[10px]" />
                ))}
            </output>
        );
    }
    if (rows.length === 0) {
        return (
            <div className="rounded-[12px] border border-border border-dashed p-8 text-center">
                <p className="m-0 font-sans text-[13px] text-muted-foreground">{empty}</p>
                {emptyNote ? <p className="mt-1.5 mb-0 font-sans text-[11.5px] text-muted-foreground/80">{emptyNote}</p> : null}
            </div>
        );
    }

    return (
        <>
            <p className="mt-0 mb-2 font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("operators.dialog.shelfWords", { count: count ?? rows.length, words: f.number(words ?? 0) })}</p>
            <div className="flex flex-col gap-1.5">{children(rows)}</div>
        </>
    );
}

/** One expandable section: the title row, the word count, and the text under it. */
function Section({ title, meta, badge, icon, words, children }: { title: string; meta?: string; badge?: React.ReactNode; icon?: React.ReactNode; words: number; children: React.ReactNode }): React.ReactElement {
    const t: DialogT = useT("story");
    const f = useFormatters();
    const [open, setOpen] = useState(false);

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger
                aria-label={open ? t("operators.dialog.section.collapse", { name: title }) : t("operators.dialog.section.expand", { name: title })}
                className="flex w-full cursor-pointer items-center gap-2.5 rounded-[10px] border border-border bg-secondary/35 px-3 py-2 text-left transition-colors hover:border-primary/45 max-sm:min-h-11"
            >
                {icon}
                <span className="flex min-w-0 flex-1 flex-col">
                    <span className="truncate font-sans font-semibold text-[12.5px] text-foreground">{title}</span>
                    {meta ? <span className="truncate font-mono text-[10px] text-muted-foreground uppercase tracking-wide">{meta}</span> : null}
                </span>
                {badge}
                <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">{t("operators.dialog.words", { count: f.number(words) })}</span>
                <ChevronDownIcon className={cn("size-3.5 shrink-0 text-muted-foreground transition-transform", open && "rotate-180")} aria-hidden="true" />
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="mt-1.5 rounded-[10px] border border-border/60 bg-card/40 p-3">{children}</div>
            </CollapsibleContent>
        </Collapsible>
    );
}

function FileRow({ section }: { section: IFileSection }): React.ReactElement {
    return (
        <Section title={section.title} words={section.words}>
            <HandbookText text={section.text} />
        </Section>
    );
}

function ModuleRow({ section }: { section: IModuleSection }): React.ReactElement {
    const t: DialogT = useT("story");
    return (
        <Section
            title={section.name}
            meta={section.designator}
            words={section.words}
            icon={<img src={asset(section.iconPath)} alt="" aria-hidden="true" loading="lazy" decoding="async" className="size-8 shrink-0 rounded-md bg-secondary/60 object-contain" />}
            badge={
                section.original ? (
                    <Badge variant="outline" className="shrink-0 text-[10px]">
                        {t("operators.dialog.original")}
                    </Badge>
                ) : null
            }
        >
            <HandbookText text={section.description} />
        </Section>
    );
}

/**
 * The handbook's line-oriented markup, the way the operator detail page's Lore
 * tab reads it: a `[Header] value` row is a labelled line, everything else is a
 * paragraph, and both go through `GameText` so a rich-text span renders instead
 * of printing its tags.
 */
function HandbookText({ text }: { text: string }): React.ReactElement {
    const lines = text.split("\n");
    return (
        <div className="flex flex-col gap-1.5">
            {lines.map((raw, index) => {
                const line = raw.trim();
                if (line === "") return null;
                const key = `${index}:${line.slice(0, 24)}`;
                if (line.startsWith("[") && line.includes("]")) {
                    const close = line.indexOf("]");
                    const header = line.slice(1, close).trim();
                    const value = line.slice(close + 1).trim();
                    return (
                        <div key={key} className="flex flex-col gap-0.5 sm:flex-row sm:items-baseline sm:gap-3">
                            <span className="shrink-0 font-mono text-[10px] text-primary uppercase tracking-wide sm:w-36 sm:text-right">{header}</span>
                            <span className="min-w-0 flex-1 font-sans text-[12.5px] text-foreground leading-relaxed">{value ? <GameText text={value} /> : "-"}</span>
                        </div>
                    );
                }
                return (
                    <p key={key} className="m-0 font-sans text-[12.5px] text-muted-foreground leading-relaxed">
                        <GameText text={line} />
                    </p>
                );
            })}
        </div>
    );
}

// =============================================================================
// Voice lines
// =============================================================================

/** One `<audio>` element for the whole shelf, so opening a second line stops the first. */
function useOneShotPlayer() {
    const ref = useRef<HTMLAudioElement | null>(null);
    const [playing, setPlaying] = useState<string | null>(null);

    const stop = useCallback(() => {
        const element = ref.current;
        if (element) {
            element.pause();
            element.removeAttribute("src");
            element.load();
        }
        setPlaying(null);
    }, []);

    const toggle = useCallback(
        (key: string, url: string) => {
            const element = ref.current;
            if (!element) return;
            if (playing === key) {
                element.pause();
                setPlaying(null);
                return;
            }
            element.src = url;
            setPlaying(key);
            element.play().catch(() => setPlaying(null));
        },
        [playing],
    );

    return { ref, playing, toggle, stop };
}

function VoicesShelf({ loading, failed, lines, words }: { loading: boolean; failed: boolean; lines: IVoiceLine[] | null; words: number | null }): React.ReactElement {
    const t: DialogT = useT("story");
    const f = useFormatters();
    const tLang: LanguageT = useT("operators");
    const { ref, playing, toggle, stop } = useOneShotPlayer();

    const languages = useMemo(() => voiceLanguages(lines ?? [], VOICE_LANGUAGE_ORDER), [lines]);
    const [language, setLanguage] = useState<LangType>("JP");

    useEffect(() => {
        if (languages.length > 0 && !languages.includes(language)) setLanguage(languages[0]);
    }, [languages, language]);

    // biome-ignore lint/correctness/useExhaustiveDependencies: a clip of the old language must not keep playing
    useEffect(() => stop(), [language]);

    if (failed) return <p className="py-8 text-center font-sans text-[13px] text-muted-foreground">{t("operators.dialog.failed")}</p>;
    if (loading || lines === null) {
        return (
            <output className="flex flex-col gap-2" aria-label={t("operators.dialog.loading")}>
                {[0, 1, 2].map((i) => (
                    <Skeleton key={i} className="h-11 w-full rounded-[10px]" />
                ))}
            </output>
        );
    }
    if (lines.length === 0) return <div className="rounded-[12px] border border-border border-dashed p-8 text-center font-sans text-[13px] text-muted-foreground">{t("operators.dialog.voices.empty")}</div>;

    return (
        <>
            {/* biome-ignore lint/a11y/useMediaCaption: the game ships no transcript track with a voice clip */}
            <audio ref={ref} preload="none" onEnded={stop} />
            <div className="mb-2.5 flex flex-wrap items-center gap-2.5">
                <Select value={language} onValueChange={(value) => setLanguage(String(value) as LangType)}>
                    <SelectTrigger className="h-11 w-44 sm:h-9" aria-label={t("operators.dialog.voices.language")}>
                        <SelectValue placeholder={t("operators.dialog.voices.language")} />
                    </SelectTrigger>
                    <SelectContent>
                        {languages.map((lang) => (
                            <SelectItem key={lang} value={lang}>
                                {languageLabel(lang, tLang)}
                            </SelectItem>
                        ))}
                    </SelectContent>
                </Select>
                <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{t("operators.dialog.shelfWords", { count: lines.length, words: f.number(words ?? 0) })}</span>
            </div>
            <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
                {lines.map((line) => (
                    <VoiceRow key={line.key} line={line} language={language} playing={playing === line.key} onToggle={(url) => toggle(line.key, url)} />
                ))}
            </ul>
        </>
    );
}

function VoiceRow({ line, language, playing, onToggle }: { line: IVoiceLine; language: LangType; playing: boolean; onToggle: (url: string) => void }): React.ReactElement {
    const t: DialogT = useT("story");
    const f = useFormatters();
    const clip = line.clips[language];

    return (
        <li className={cn("flex items-start gap-2.5 rounded-[10px] border p-2.5", playing ? "border-primary bg-primary/10" : "border-border bg-card/40")}>
            <button
                type="button"
                disabled={!clip}
                onClick={() => clip && onToggle(audioURL(clip))}
                aria-label={clip ? (playing ? t("operators.dialog.voices.pause", { name: line.title }) : t("operators.dialog.voices.play", { name: line.title })) : t("operators.dialog.voices.noClip")}
                className={cn("flex size-11 shrink-0 cursor-pointer items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-45 sm:size-9", playing ? "bg-primary text-primary-foreground" : "bg-secondary text-foreground hover:bg-secondary/80")}
            >
                {playing ? <PauseIcon className="size-4" aria-hidden="true" /> : <PlayIcon className="size-4" aria-hidden="true" />}
            </button>
            <div className="min-w-0 flex-1">
                <div className="flex items-baseline justify-between gap-2">
                    <span className="truncate font-sans font-semibold text-[12px] text-foreground">{line.title}</span>
                    <span className="shrink-0 font-mono text-[10px] text-muted-foreground tabular-nums">{t("operators.dialog.words", { count: f.number(line.words) })}</span>
                </div>
                <p className="m-0 mt-0.5 font-sans text-[12.5px] text-muted-foreground leading-relaxed">
                    <GameText text={line.text} />
                </p>
            </div>
        </li>
    );
}

/** The language's translated name. The 11 names are already in the operators catalog; a second copy in `story` would be two catalogs of one word, free to drift. */
function languageLabel(lang: LangType, t: LanguageT): string {
    const key = VOICE_LANGUAGE_LABEL_KEY[lang];
    return key ? t(key) : lang;
}
