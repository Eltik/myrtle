import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { type IPersonOption, PersonCombobox } from "#/components/user/PersonCombobox";
import { useDebounce } from "#/hooks/use-debounce";
import type { ITierListBrowseItem } from "#/lib/api/tier-lists";
import { searchUsersQueryOptions } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { searchAndRank } from "#/lib/search/fuzzy";
import type { messages } from "./AuthorFilter.messages";

/** Suggestions shown at once: authors with the most lists, before any typing. */
const SUGGESTIONS = 8;

interface IAuthor {
    id: string;
    name: string;
    avatarId: string | null;
    lists: number;
}

/** Every author in `lists`, most lists first. */
export function authorsOf(lists: readonly ITierListBrowseItem[]): IAuthor[] {
    const byId = new Map<string, IAuthor>();
    for (const list of lists) {
        const id = list.author.id;
        if (!id) continue;
        const known = byId.get(id);
        if (known) known.lists += 1;
        else byId.set(id, { id, name: list.author.name, avatarId: list.author.avatarId, lists: 1 });
    }
    return [...byId.values()].sort((a, b) => b.lists - a.lists || a.name.localeCompare(b.name));
}

interface IAuthorFilterProps {
    /** The index the suggestions come from (not the author-filtered view). */
    lists: readonly ITierListBrowseItem[];
    /** The filtered author's id, or empty. */
    value: string;
    onChange: (author: { id: string; name: string } | null) => void;
}

/**
 * Narrows the index to one player's lists. Suggests the index's own authors
 * (ranked through `fuzzy.ts`) and, once something is typed, every player the
 * site search finds, so an author past the loaded window is still reachable.
 */
export function AuthorFilter({ lists, value, onChange }: IAuthorFilterProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const [query, setQuery] = useState("");
    const [picked, setPicked] = useState<IPersonOption | null>(null);
    const term = useDebounce(query.trim(), 300);
    const remoteEnabled = term.length > 0;
    const remote = useQuery({ ...searchUsersQueryOptions({ q: term, limit: SUGGESTIONS }), enabled: remoteEnabled });

    const authors = useMemo(() => authorsOf(lists), [lists]);
    const counts = useMemo(() => new Map(authors.map((a) => [a.id, a.lists])), [authors]);

    const options = useMemo<IPersonOption[]>(() => {
        const listsHint = (n: number | undefined): string | null => (n ? t("browse.author.lists", { count: n }) : null);
        const ranked = term.length > 0 ? searchAndRank(term, authors, (a) => ({ name: a.name }), SUGGESTIONS).map((s) => s.item) : authors.slice(0, SUGGESTIONS);
        const local = ranked.map((a) => ({ id: a.id, name: a.name, avatarId: a.avatarId, hint: listsHint(a.lists) }));
        if (term.length === 0) return local;
        const seen = new Set(local.map((o) => o.id));
        const found = (remote.data?.entries ?? []).filter((e) => !seen.has(e.id)).map((e) => ({ id: e.id, name: e.nickname?.trim() || e.uid, avatarId: e.avatar_id, hint: listsHint(counts.get(e.id)) }));
        return [...local, ...found];
    }, [authors, counts, term, remote.data, t]);

    const selected = useMemo<IPersonOption | null>(() => {
        if (!value) return null;
        if (picked?.id === value) return picked;
        const author = authors.find((a) => a.id === value);
        return author ? { id: author.id, name: author.name, avatarId: author.avatarId } : { id: value, name: t("browse.author.unknown") };
    }, [value, picked, authors, t]);

    return (
        <div className="w-full min-w-0 sm:w-48">
            <PersonCombobox
                value={selected}
                onChange={(next) => {
                    setPicked(next);
                    onChange(next ? { id: next.id, name: next.name } : null);
                }}
                options={options}
                onQueryChange={setQuery}
                loading={remoteEnabled && (remote.isPending || term !== query.trim())}
                label={t("browse.author.label")}
                placeholder={t("browse.author.placeholder")}
                emptyText={t("browse.author.empty")}
                loadingText={t("browse.author.searching")}
            />
        </div>
    );
}
