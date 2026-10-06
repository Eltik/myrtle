import { BookOpenIcon, CheckIcon, ChevronDownIcon, FilterXIcon, SearchIcon, XIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "#/components/ui/input-group";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import { type ICategoryOption, type IGroupOption, searchGroups } from "../../gallery";
import type { messages } from "../ArtBrowser.messages";
import { COUNT, chip } from "../chips";

type T = TypedT<typeof messages>;

/** A library category's label, each key written out so the extractor sees it. */
function categoryLabel(t: T, category: StoryCategory): string {
    switch (category) {
        case "main":
            return t("profile.background.gallery.category.main");
        case "side":
            return t("profile.background.gallery.category.side");
        case "vignette":
            return t("profile.background.gallery.category.vignette");
        case "is":
            return t("profile.background.gallery.category.is");
        case "reclamation":
            return t("profile.background.gallery.category.reclamation");
        case "sideContent":
            return t("profile.background.gallery.category.sideContent");
        case "record":
            return t("profile.background.gallery.category.record");
    }
}

/** The category toggles as one row of chips, each with its muted count; several can be on. A category the other filters leave empty dims. */
export function CategoryChips({ options, chosen, onToggle }: { options: readonly ICategoryOption[]; chosen: readonly StoryCategory[]; onToggle: (id: StoryCategory) => void }) {
    const t: T = useT("user");
    return (
        <fieldset className="m-0 flex min-w-0 flex-wrap gap-1.5 border-0 p-0" aria-label={t("profile.background.gallery.categoriesLabel")}>
            {options.map((option) => {
                const on = chosen.includes(option.id);
                const label = categoryLabel(t, option.id);
                return (
                    <button key={option.id} type="button" aria-pressed={on} onClick={() => onToggle(option.id)} aria-label={t("profile.background.gallery.categoryOption", { label, count: option.count })} className={cn(chip(on), option.count === 0 && !on && "opacity-50")}>
                        {on && <CheckIcon className="size-3.5" strokeWidth={2.5} aria-hidden="true" />}
                        {label}
                        <span className={COUNT}>{option.count}</span>
                    </button>
                );
            })}
        </fieldset>
    );
}

interface IStoryPickerProps {
    groups: readonly IGroupOption[];
    chosen: string | null;
    chosenName: string | null;
    /** How many tiles "All stories" leaves. */
    total: number;
    onChoose: (group: string | null) => void;
}

interface ISearchFieldProps {
    value: string;
    onChange: (value: string) => void;
    placeholder: string;
    /** The input's accessible label. */
    label: string;
    className?: string;
}

/** A search box: the magnifier, then the input. The gallery's search and both story searches are this. */
export function SearchField({ value, onChange, placeholder, label, className }: ISearchFieldProps) {
    return (
        <InputGroup className={className}>
            <InputGroupAddon>
                <SearchIcon aria-hidden="true" />
            </InputGroupAddon>
            <InputGroupInput value={value} onChange={(e) => onChange((e.target as HTMLInputElement).value)} placeholder={placeholder} type="search" aria-label={label} />
        </InputGroup>
    );
}

const storyRow = (active: boolean) =>
    cn("flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-start font-sans text-[13px] leading-tight outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring", active ? "bg-primary/12 font-medium text-foreground" : "text-muted-foreground hover:bg-accent hover:text-foreground");

/**
 * The story list under a story search: "All stories" first while the search is blank, then
 * each story `query` leaves (through `fuzzy.ts`) with its count, or a line saying none is
 * left. Choosing the chosen story again clears it.
 */
function StoryRows({ groups, query, chosen, total, onChoose }: Omit<IStoryPickerProps, "chosenName"> & { query: string }) {
    const t: T = useT("user");
    const listed = useMemo(() => searchGroups(groups, query), [groups, query]);
    return (
        <>
            {query.trim().length === 0 && (
                <button type="button" aria-pressed={chosen === null} onClick={() => onChoose(null)} className={storyRow(chosen === null)}>
                    <span className="min-w-0 flex-1 truncate">{t("profile.background.gallery.allGroups")}</span>
                    <span className={COUNT}>{total}</span>
                </button>
            )}
            {listed.map((group) => (
                <button key={group.id} type="button" aria-pressed={chosen === group.id} onClick={() => onChoose(chosen === group.id ? null : group.id)} title={group.name} className={storyRow(chosen === group.id)}>
                    <span className="min-w-0 flex-1 truncate">{group.name}</span>
                    <span className={COUNT}>{group.count}</span>
                </button>
            ))}
            {listed.length === 0 && <p className="m-0 px-2 py-1.5 font-sans text-muted-foreground text-xs">{t("profile.background.gallery.storyEmpty")}</p>}
        </>
    );
}

/**
 * The story filter: a button naming the chosen story that opens a bounded panel with its
 * own search (through `fuzzy.ts`), "All stories" first, then each story with its count.
 * The search sits on an opaque bar above the list, never over it; the list scrolls inside
 * the panel and fades at both edges.
 */
export function StoryPicker({ groups, chosen, chosenName, total, onChoose }: IStoryPickerProps) {
    const t: T = useT("user");
    const [open, setOpen] = useState(false);
    const [query, setQuery] = useState("");
    const choose = (group: string | null) => {
        onChoose(group);
        setOpen(false);
    };
    return (
        <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger render={<Button type="button" variant="outline" className="min-w-0 max-w-[50%] shrink sm:max-w-72" />}>
                <BookOpenIcon aria-hidden="true" />
                <span className="min-w-0 truncate">{chosenName ?? t("profile.background.gallery.allGroups")}</span>
                <ChevronDownIcon aria-hidden="true" className="opacity-60" />
            </PopoverTrigger>
            <PopoverPopup align="end" className="w-80 max-w-[calc(100vw-2rem)] [&_[data-slot=popover-viewport]]:p-0">
                <div className="flex h-96 max-h-[60dvh] flex-col">
                    <div className="border-b p-2">
                        <SearchField value={query} onChange={setQuery} placeholder={t("profile.background.gallery.storySearchPlaceholder")} label={t("profile.background.gallery.storySearchLabel")} />
                    </div>
                    <div className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto p-1.5 [mask-image:linear-gradient(to_bottom,transparent,black_10px,black_calc(100%-16px),transparent)]">
                        <StoryRows groups={groups} query={query} chosen={chosen} total={total} onChoose={choose} />
                    </div>
                </div>
            </PopoverPopup>
        </Popover>
    );
}

/**
 * The rail's story filter, a bounded panel of its own: its search on an opaque bar at the
 * top, outside the scroll, so no row ever shows through it; under it "All stories", then
 * each story with its muted count, scrolling inside the panel and fading at both edges.
 * It fills whatever height the rail leaves it.
 */
export function StoryPanel({ groups, chosen, total, onChoose }: Omit<IStoryPickerProps, "chosenName">) {
    const t: T = useT("user");
    const [query, setQuery] = useState("");
    return (
        <div className="flex min-h-48 flex-1 flex-col overflow-hidden rounded-lg border bg-card/40">
            <div className="shrink-0 border-b bg-card p-1.5">
                <SearchField value={query} onChange={setQuery} placeholder={t("profile.background.gallery.storySearchPlaceholder")} label={t("profile.background.gallery.storySearchLabel")} className="h-8" />
            </div>
            <div className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto p-1.5 [mask-image:linear-gradient(to_bottom,transparent,black_10px,black_calc(100%-18px),transparent)]">
                <StoryRows groups={groups} query={query} chosen={chosen} total={total} onChoose={onChoose} />
            </div>
        </div>
    );
}

/** The chosen story as a removable chip, then Clear all. Rendered only while something filters. */
export function ActiveFilters({ groupName, onRemoveGroup, onClear }: { groupName: string | null; onRemoveGroup: () => void; onClear: () => void }) {
    const t: T = useT("user");
    return (
        <>
            {groupName !== null && (
                <span className="inline-flex h-7 max-w-64 items-center gap-1 rounded-full border border-primary/40 bg-primary/10 ps-2.5 pe-0.5 font-sans text-foreground text-xs">
                    <BookOpenIcon className="size-3.5 shrink-0 opacity-70" aria-hidden="true" />
                    <span className="min-w-0 truncate">{groupName}</span>
                    <button type="button" onClick={onRemoveGroup} aria-label={t("profile.background.gallery.removeFilter", { name: groupName })} className="inline-flex size-6 cursor-pointer items-center justify-center rounded-full text-muted-foreground hover:bg-accent hover:text-foreground">
                        <XIcon className="size-3" />
                    </button>
                </span>
            )}
            <Button type="button" variant="ghost" size="xs" onClick={onClear}>
                <FilterXIcon />
                {t("profile.background.gallery.clearFilters")}
            </Button>
        </>
    );
}
