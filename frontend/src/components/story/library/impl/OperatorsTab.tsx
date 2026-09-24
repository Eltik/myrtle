import { CheckIcon, SearchIcon, StarIcon } from "lucide-react";
import type React from "react";
import { useId, useMemo, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Checkbox } from "#/components/ui/checkbox";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryProgress } from "#/lib/story/progress";
import { cn, formatProfession, getAvatarById, RARITY_HEX, rarityToNumber } from "#/lib/utils";
import { filterRecords, type LibRecord, readFraction } from "./derive";
import { OperatorDialog } from "./OperatorDialog";
import type { messages } from "./OperatorsTab.messages";

export interface IOperatorsTabProps {
    records: LibRecord[];
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    /** False inside the browse page's records section, whose own search box and pills already filtered `records`. */
    controls?: boolean;
}

/** The operator's rarity as 1 to 6, or `null` while the backend sends no `rarity`; the wire's own "no such operator" sentinel is 0. */
function rarityOf(record: LibRecord): number | null {
    if (!record.rarity) return null;
    return rarityToNumber(record.rarity);
}

export function OperatorsTab({ records, progress, gameRead, controls = true }: IOperatorsTabProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const [query, setQuery] = useState("");
    const [hideFinished, setHideFinished] = useState(false);
    const [open, setOpen] = useState<LibRecord | null>(null);
    const hideFinishedId = useId();

    const visible = useMemo(() => {
        const searched = filterRecords(query, records);
        return hideFinished ? searched.filter((r) => !readFraction(r.stories, progress, gameRead).done) : searched;
    }, [records, query, hideFinished, progress, gameRead]);

    const done = useMemo(() => records.filter((r) => readFraction(r.stories, progress, gameRead).done).length, [records, progress, gameRead]);

    return (
        <div className={controls ? "pt-4" : undefined}>
            {controls ? (
                <div className="mb-4 flex flex-wrap items-center gap-x-3 gap-y-2.5">
                    <div className="relative min-w-0 flex-1 basis-full sm:max-w-100 sm:basis-auto">
                        <SearchIcon className="pointer-events-none absolute top-1/2 left-3.5 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" aria-hidden="true" />
                        <input
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            placeholder={t("operators.search.placeholder")}
                            aria-label={t("operators.search.aria")}
                            className="h-11 w-full rounded-[9px] border border-border bg-secondary/50 pr-3.5 pl-9 font-sans text-[13px] text-foreground outline-none focus-visible:border-primary focus-visible:ring-2 focus-visible:ring-ring/40 sm:h-9.5"
                        />
                    </div>
                    <label htmlFor={hideFinishedId} className="flex cursor-pointer select-none items-center gap-2 font-sans text-[12px] text-muted-foreground max-sm:min-h-11">
                        <Checkbox id={hideFinishedId} checked={hideFinished} onCheckedChange={(checked) => setHideFinished(checked === true)} />
                        {t("operators.hideFinished")}
                    </label>
                    <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("operators.readCount", { done, total: records.length })}</span>
                </div>
            ) : null}

            {visible.length === 0 ? (
                <div className="rounded-[14px] border border-border border-dashed p-14 text-center font-sans text-[14px] text-muted-foreground">{t("operators.empty")}</div>
            ) : (
                <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
                    {visible.map((record) => (
                        <OperatorCard key={record.charId} record={record} progress={progress} gameRead={gameRead} onOpen={() => setOpen(record)} />
                    ))}
                </div>
            )}

            <OperatorDialog key={open?.charId ?? "none"} record={open} progress={progress} gameRead={gameRead} onClose={() => setOpen(null)} />
        </div>
    );
}

function OperatorCard({ record, progress, gameRead, onOpen }: { record: LibRecord; progress: StoryProgress; gameRead: ReadonlySet<string>; onOpen: () => void }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const fraction = readFraction(record.stories, progress, gameRead);
    const rarity = rarityOf(record);
    const tone = rarity !== null ? (RARITY_HEX[rarity] ?? "var(--border)") : "var(--border)";
    const avatar = record.avatarUrl ? asset(record.avatarUrl) : getAvatarById(record.charId);

    return (
        <button
            type="button"
            onClick={onOpen}
            aria-haspopup="dialog"
            aria-label={t("operators.card.open", { name: record.name })}
            className="group flex cursor-pointer items-center gap-2.5 rounded-[12px] border border-border bg-card p-2 text-left ring-inset transition-colors hover:border-primary/45 focus-visible:ring-2 focus-visible:ring-ring/60 max-sm:min-h-11"
        >
            <span className="relative block size-12 shrink-0 overflow-hidden rounded-[9px] bg-secondary/60 sm:size-13" style={{ borderBottom: `3px solid ${tone}` }}>
                <img src={avatar} alt="" aria-hidden="true" loading="lazy" decoding="async" className="h-full w-full object-cover" />
            </span>
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="truncate font-sans font-semibold text-[12.5px] text-foreground sm:text-[13px]">{record.name}</span>
                {rarity !== null ? (
                    <span className="flex items-center gap-px" role="img" aria-label={t("operators.rarity", { count: rarity })}>
                        {Array.from({ length: rarity }, (_, i) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: a rarity star has no identity beyond its position
                            <StarIcon key={i} className="size-2.5 fill-current" style={{ color: tone }} aria-hidden="true" />
                        ))}
                    </span>
                ) : null}
                {record.profession ? <span className="truncate font-mono text-[10px] text-muted-foreground">{formatProfession(record.profession)}</span> : null}
                <span className={cn("font-mono text-[10px] tabular-nums", fraction.done ? "text-primary" : "text-muted-foreground")}>{t("operators.card.read", { read: fraction.read, total: fraction.total })}</span>
            </span>
            {fraction.done ? <CheckIcon className="size-4 shrink-0 text-primary" aria-hidden="true" /> : null}
        </button>
    );
}
