import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useEffect, useMemo, useRef, useState } from "react";
import { useAdminAccess } from "#/components/admin/shell/access";
import { PageHead } from "#/components/admin/shell/PageHead";
import type { AdminNotesStatus, IAdminNotesSearch } from "#/components/admin/shell/search";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { useScrollTopOnOpen } from "#/components/admin/shell/useScrollTopOnOpen";
import type { IActiveChip } from "#/components/operators/list/impl/components/ActiveFilterChips";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { Skeleton } from "#/components/ui/skeleton";
import { updateOperatorNoteFn } from "#/lib/api/admin";
import { type IOperatorNote, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { NoteEditor } from "./NoteEditor";
import { NotesList, statusLabel } from "./NotesList";
import { DEFAULT_SORT, filterRows, type INoteDraft, type INoteRow, joinRows, type NoteSort, nextUnfinishedOperator, sortRows, statusCounts, tagsInUse, toUpdateInput, upsertNote } from "./notesModel";
import type { messages } from "./OperatorNotes.messages";

type NotesT = TypedT<typeof messages>;

export interface IOperatorNotesProps {
    search: IAdminNotesSearch;
}

/**
 * Operator notes (spec 2.4): a master-detail editor. The list is every
 * operator in the index joined with its note; `op`, `status` and `q` live in
 * the URL, the panel filters (rarity, tags, sort) are local.
 */
export default function OperatorNotes({ search }: IOperatorNotesProps): React.ReactElement {
    const t: NotesT = useT("admin");
    const f = useFormatters();
    const describeError = useErrorMessage();
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const access = useAdminAccess();
    const enabled = access.can.notes;

    const opsQuery = useQuery({ ...operatorsIndexQueryOptions(useGamedataServer()), enabled });
    const notesQuery = useQuery({ ...operatorNotesListQueryOptions(), enabled });

    const status: AdminNotesStatus = search.status ?? "all";
    const [rarities, setRarities] = useState<number[]>([]);
    const [tags, setTags] = useState<string[]>([]);
    const [sort, setSort] = useState<NoteSort>(DEFAULT_SORT);

    const setSearch = (patch: Partial<IAdminNotesSearch>, replace = true) => {
        // `q` comes from the box, which may be a keystroke ahead of the URL.
        const next = { ...search, q: lastWrittenQ.current, ...patch };
        void navigate({
            to: "/admin/operator-notes",
            search: { op: next.op || undefined, status: next.status && next.status !== "all" ? next.status : undefined, q: next.q || undefined },
            replace,
        });
    };

    // The search box keeps its own text so typing never waits on the router;
    // the URL follows on every keystroke and wins when it changes from outside.
    const [q, setQ] = useState(search.q ?? "");
    const lastWrittenQ = useRef(search.q ?? "");
    useEffect(() => {
        const fromUrl = search.q ?? "";
        if (fromUrl !== lastWrittenQ.current) {
            lastWrittenQ.current = fromUrl;
            setQ(fromUrl);
        }
    }, [search.q]);
    const changeQ = (value: string) => {
        setQ(value);
        lastWrittenQ.current = value;
        setSearch({ q: value });
    };

    const compareNames = f.collator.compare;
    const rows = useMemo(() => joinRows(opsQuery.data ?? [], notesQuery.data ?? []), [opsQuery.data, notesQuery.data]);
    const counts = useMemo(() => statusCounts(rows), [rows]);
    const allTags = useMemo(() => tagsInUse(rows, compareNames), [rows, compareNames]);
    const visible = useMemo(() => filterRows(rows, { status, q, rarities, tags, sort }, compareNames), [rows, status, q, rarities, tags, sort, compareNames]);

    // The selected operator is looked up in the whole roster, so one that a
    // save just moved out of the filter stays open. Without `op` the desktop
    // layout opens the first row; the phone layout shows the list instead.
    const selected: INoteRow | null = rows.find((r) => r.id === search.op) ?? visible[0] ?? null;

    // Opening an operator from the stacked layout swaps the list out for the editor; start the page
    // at its top.
    const { listRef, arm: armScrollTop } = useScrollTopOnOpen(search.op);
    const select = (id: string) => {
        armScrollTop();
        setSearch({ op: id }, false);
    };

    const save = useMutation({
        mutationFn: (vars: { row: INoteRow; draft: INoteDraft; next: INoteRow | null }) => updateOperatorNoteFn({ data: toUpdateInput(vars.row.id, vars.draft) }),
        onSuccess: (saved: IOperatorNote, { row, next }) => {
            queryClient.setQueryData(operatorNotesListQueryOptions().queryKey, (old) => upsertNote(old, saved));
            void queryClient.invalidateQueries({ queryKey: ["operator-notes"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "operator-notes", "audit"] });
            if (next) setSearch({ op: next.id }, false);
            toastSuccess("note-save", t("notes.toasts.saved"), next ? t("notes.toasts.savedNext", { name: row.name, next: next.name }) : t("notes.toasts.savedDesc", { name: row.name }));
        },
        onError: (err: unknown) => toastError("note-save-err", t("notes.toasts.failed"), describeError(err)),
    });

    const onSave = async (row: INoteRow, draft: INoteDraft, goNext: boolean): Promise<boolean> => {
        // The queue follows what the list shows; past its end, the whole roster.
        const next = goNext ? (nextUnfinishedOperator(visible, row.id) ?? nextUnfinishedOperator(sortRows(rows, sort, compareNames), row.id)) : null;
        try {
            await save.mutateAsync({ row, draft, next });
            return true;
        } catch {
            return false;
        }
    };

    const clearAll = () => {
        setRarities([]);
        setTags([]);
        setSort(DEFAULT_SORT);
        setQ("");
        lastWrittenQ.current = "";
        setSearch({ q: undefined, status: undefined });
    };

    const activeChips: IActiveChip[] = [
        ...(q ? [{ key: "q", label: t("notes.chips.query", { q }), onRemove: () => changeQ("") }] : []),
        ...(status !== "all" ? [{ key: "status", label: statusLabel(t, status), onRemove: () => setSearch({ status: undefined }) }] : []),
        ...rarities.map((r) => ({ key: `r${r}`, label: t("notes.filters.rarityChip", { rarity: f.number(r) }), onRemove: () => setRarities((cur) => cur.filter((x) => x !== r)) })),
        ...tags.map((tag) => ({ key: `t:${tag}`, label: tag, onRemove: () => setTags((cur) => cur.filter((x) => x !== tag)) })),
        ...(sort !== DEFAULT_SORT ? [{ key: "sort", label: sort === "rarity" ? t("notes.chips.sortedRarity") : t("notes.sortBy.recent"), onRemove: () => setSort(DEFAULT_SORT) }] : []),
    ];

    const loading = enabled && (opsQuery.isPending || notesQuery.isPending);
    const failed = opsQuery.isError || notesQuery.isError;
    const hasOp = !!search.op;

    return (
        <>
            <PageHead kicker={t("notes.head.kicker")} title={t("notes.head.title")} sub={t("notes.head.sub")} />
            <div className="grid grid-cols-[minmax(0,1fr)] items-start gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
                <NotesList
                    listRef={listRef}
                    hidden={hasOp}
                    q={q}
                    onQuery={changeQ}
                    status={status}
                    counts={counts}
                    onStatus={(next) => setSearch({ status: next })}
                    rarities={rarities}
                    setRarities={setRarities}
                    tags={tags}
                    setTags={setTags}
                    sort={sort}
                    setSort={setSort}
                    allTags={allTags}
                    activeChips={activeChips}
                    onClearAll={clearAll}
                    loading={loading}
                    failed={failed}
                    onRetry={() => {
                        void opsQuery.refetch();
                        void notesQuery.refetch();
                    }}
                    rows={visible}
                    selectedId={selected?.id}
                    onSelect={select}
                />

                <Card className={cn("min-w-0", !hasOp && "max-lg:hidden")}>
                    <div className="px-3 pt-3 lg:hidden">
                        <Button size="sm" variant="ghost" onClick={() => setSearch({ op: undefined }, false)}>
                            {t("notes.detail.back")}
                        </Button>
                    </div>
                    {loading ? (
                        <div className="flex flex-col gap-3 p-5">
                            <Skeleton className="h-11 w-60" />
                            <Skeleton className="h-8" />
                            <Skeleton className="h-24" />
                            <Skeleton className="h-24" />
                        </div>
                    ) : selected ? (
                        <NoteEditor key={selected.id} row={selected} enabled={enabled} saving={save.isPending} onSave={onSave} />
                    ) : (
                        <div className="px-5 py-10 text-[13px] text-muted-foreground">{t("notes.detail.none")}</div>
                    )}
                </Card>
            </div>
        </>
    );
}
