import { useState } from "react";
import { skeletons } from "#/components/admin/Primitives";
import { type AdminNotesStatus, NOTES_STATUSES } from "#/components/admin/shell/search";
import { ActiveFilterChips, type IActiveChip } from "#/components/operators/list/impl/components/ActiveFilterChips";
import { FilterToggleButton } from "#/components/operators/list/impl/components/FilterToggleButton";
import { SearchField } from "#/components/SearchField";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { FilterChip } from "#/components/ui/filter-chip";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { activeFilterCount, type INoteRow, NOTE_SORTS, type NoteSort, type NoteStatus, toggle } from "./notesModel";
import type { messages } from "./OperatorNotes.messages";
import { RarityTile } from "./RarityTile";

type NotesT = TypedT<typeof messages>;

const STATUS_BADGE = { empty: "warning", incomplete: "info", filled: "secondary" } as const satisfies Record<NoteStatus, string>;
const RARITY_CHIPS = [6, 5, 4, 3] as const;

function sortLabel(t: NotesT, sort: NoteSort): string {
    switch (sort) {
        case "name":
            return t("notes.sortBy.name");
        case "rarity":
            return t("notes.sortBy.rarity");
        case "recent":
            return t("notes.sortBy.recent");
    }
}

export function statusLabel(t: NotesT, status: AdminNotesStatus): string {
    switch (status) {
        case "all":
            return t("notes.status.all");
        case "empty":
            return t("notes.status.empty");
        case "incomplete":
            return t("notes.status.incomplete");
        case "filled":
            return t("notes.status.filled");
    }
}

interface INotesListProps {
    listRef: React.RefObject<HTMLDivElement | null>;
    /** Below lg, an open operator hides the list. */
    hidden: boolean;
    q: string;
    onQuery: (value: string) => void;
    status: AdminNotesStatus;
    counts: Record<AdminNotesStatus, number>;
    onStatus: (status: AdminNotesStatus) => void;
    rarities: number[];
    setRarities: React.Dispatch<React.SetStateAction<number[]>>;
    tags: string[];
    setTags: React.Dispatch<React.SetStateAction<string[]>>;
    sort: NoteSort;
    setSort: (sort: NoteSort) => void;
    allTags: string[];
    activeChips: IActiveChip[];
    onClearAll: () => void;
    loading: boolean;
    failed: boolean;
    onRetry: () => void;
    rows: INoteRow[];
    selectedId: string | undefined;
    onSelect: (id: string) => void;
}

/** The list pane: search, status chips, the panel filters and the operator rows. */
export function NotesList({ listRef, hidden, q, onQuery, status, counts, onStatus, rarities, setRarities, tags, setTags, sort, setSort, allTags, activeChips, onClearAll, loading, failed, onRetry, rows, selectedId, onSelect }: INotesListProps): React.ReactElement {
    const t: NotesT = useT("admin");
    const f = useFormatters();
    const [filtersOpen, setFiltersOpen] = useState(false);

    return (
        <Card ref={listRef} className={cn("min-w-0 overflow-hidden lg:sticky lg:top-20 lg:h-[calc(100dvh-6rem)]", hidden && "max-lg:hidden")}>
            <div className="flex flex-col gap-2.5 border-border border-b p-3">
                <div className="flex items-center gap-2">
                    <div className="min-w-0 flex-1">
                        <SearchField value={q} onChange={onQuery} placeholder={t("notes.list.search")} label={t("notes.list.search")} />
                    </div>
                    {/* The shared toggle is a 24px chevron below md; a coarse pointer gets a 44px hit area around it. */}
                    <span className="inline-flex shrink-0 pointer-coarse:[&>button]:after:absolute pointer-coarse:[&>button]:after:-inset-2.5">
                        <FilterToggleButton visible={filtersOpen} onToggle={() => setFiltersOpen((v) => !v)} activeCount={activeFilterCount({ rarities, tags, sort })} />
                    </span>
                </div>
                <div className="flex flex-wrap gap-1.5">
                    {NOTES_STATUSES.map((s) => (
                        <FilterChip key={s} label={statusLabel(t, s)} count={counts[s]} active={status === s} onSelect={() => onStatus(s)} />
                    ))}
                </div>
                {filtersOpen ? (
                    <div className="flex flex-col gap-3 rounded-xl bg-muted p-3">
                        <FilterGroup label={t("notes.filters.rarity")}>
                            {RARITY_CHIPS.map((r) => (
                                <FilterChip key={r} label={t("notes.filters.rarityChip", { rarity: f.number(r) })} active={rarities.includes(r)} onSelect={() => setRarities((cur) => toggle(cur, r))} />
                            ))}
                        </FilterGroup>
                        <FilterGroup label={t("notes.filters.tags")}>
                            {allTags.length > 0 ? allTags.map((tag) => <FilterChip key={tag} label={tag} active={tags.includes(tag)} onSelect={() => setTags((cur) => toggle(cur, tag))} />) : <span className="text-[12.5px] text-muted-foreground">{t("notes.filters.noTags")}</span>}
                        </FilterGroup>
                        <FilterGroup label={t("notes.filters.sort")}>
                            {NOTE_SORTS.map((s) => (
                                <FilterChip key={s} label={sortLabel(t, s)} active={sort === s} onSelect={() => setSort(s)} />
                            ))}
                        </FilterGroup>
                    </div>
                ) : null}
                <ActiveFilterChips chips={activeChips} onClearAll={onClearAll} />
            </div>
            <div className="h-[min(480px,60dvh)] overflow-auto p-1.5 lg:h-auto lg:min-h-0 lg:flex-1">
                {loading ? (
                    <div className="flex flex-col gap-1.5 p-1">{skeletons(8, "h-[50px] rounded-lg")}</div>
                ) : failed ? (
                    <div className="flex flex-col items-start gap-2 px-2 py-5 text-[13px] text-muted-foreground">
                        <span>{t("notes.list.loadFailed")}</span>
                        <Button size="sm" variant="outline" onClick={onRetry}>
                            {t("notes.list.retry")}
                        </Button>
                    </div>
                ) : rows.length === 0 ? (
                    <div className="px-2 py-5 text-[13px] text-muted-foreground">{t("notes.list.noMatch")}</div>
                ) : (
                    rows.map((row) => <NoteListRow key={row.id} row={row} selected={row.id === selectedId} onSelect={() => onSelect(row.id)} />)
                )}
            </div>
        </Card>
    );
}

function FilterGroup({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5">
            <span className="font-medium text-[12px] text-muted-foreground">{label}</span>
            <div className="flex flex-wrap gap-1.5">{children}</div>
        </div>
    );
}

function NoteListRow({ row, selected, onSelect }: { row: INoteRow; selected: boolean; onSelect: () => void }): React.ReactElement {
    const t: NotesT = useT("admin");
    const f = useFormatters();
    const rarity = f.number(row.rarity);
    const meta = row.tags.length > 0 ? t("notes.list.metaTags", { rarity, tags: row.tags.join(", ") }) : t("notes.list.meta", { rarity });
    return (
        <button type="button" onClick={onSelect} aria-current={selected || undefined} className={cn("flex w-full cursor-pointer items-center gap-2.5 rounded-lg p-2 text-left transition-colors hover:bg-accent", selected && "bg-accent")}>
            <RarityTile id={row.id} name={row.name} rarity={row.rarity} size="sm" />
            <span className="flex min-w-0 flex-1 flex-col">
                <span className="truncate font-medium text-[13.5px]" title={row.name}>
                    {row.name}
                </span>
                <span className="truncate text-[12px] text-muted-foreground" title={meta}>
                    {meta}
                </span>
            </span>
            <Badge size="sm" variant={STATUS_BADGE[row.status]}>
                {statusLabel(t, row.status)}
            </Badge>
        </button>
    );
}
