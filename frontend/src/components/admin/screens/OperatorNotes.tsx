import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ActivityIcon, ArrowUpDownIcon, CheckIcon, ChevronRightIcon, PlusIcon, SearchIcon, XIcon } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "#/components/ui/menu";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { toastManager } from "#/components/ui/toast";
import { type IUpdateOperatorNoteInput, operatorNoteAuditLogQueryOptions, updateOperatorNoteFn } from "#/lib/api/admin";
import { type IOperatorNote, noteHasContent, operatorNoteQueryOptions, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { cn, formatSubProfession } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { HCode, PageHead } from "../AdminShell";
import { MonoSection, RARITY_BG } from "../Primitives";

interface INoteCombined {
    operatorId: string;
    op: IOperatorIndexEntry;
    note: IOperatorNote | null;
}

type StatusFilter = "all" | "has-content" | "empty";
type SortMode = "recent" | "name" | "rarity";

const SORT_LABELS: Record<SortMode, string> = {
    recent: "Recently updated",
    name: "Name (A→Z)",
    rarity: "Rarity (high→low)",
};

export function OperatorNotes(): React.ReactElement {
    const opsQuery = useQuery(operatorsIndexQueryOptions());
    const notesQuery = useQuery(operatorNotesListQueryOptions());
    const [editingId, setEditingId] = useState<string | null>(null);
    const [search, setSearch] = useState("");
    const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");
    const [sortMode, setSortMode] = useState<SortMode>("recent");

    const combined: INoteCombined[] = useMemo(() => {
        const notes = notesQuery.data ?? [];
        const ops = opsQuery.data ?? [];
        const opMap = new Map(ops.map((o) => [o.id, o]));
        return notes
            .filter((n) => opMap.has(n.operator_id))
            .map((n): INoteCombined => {
                const op = opMap.get(n.operator_id);
                if (!op) throw new Error("unreachable");
                return { operatorId: n.operator_id, op, note: n };
            });
    }, [notesQuery.data, opsQuery.data]);

    const counts = useMemo(() => {
        let withContent = 0;
        for (const c of combined) if (c.note && noteHasContent(c.note)) withContent++;
        return { all: combined.length, withContent, empty: combined.length - withContent };
    }, [combined]);

    const filtered = useMemo(() => {
        const q = search.trim().toLowerCase();
        const rows = combined.filter((c) => {
            if (statusFilter === "has-content" && !(c.note && noteHasContent(c.note))) return false;
            if (statusFilter === "empty" && c.note && noteHasContent(c.note)) return false;
            if (!q) return true;
            return c.op.name.toLowerCase().includes(q) || c.operatorId.toLowerCase().includes(q) || (c.note?.summary ?? "").toLowerCase().includes(q) || (c.note?.tags ?? []).some((t) => t.toLowerCase().includes(q));
        });
        const sorted = [...rows];
        if (sortMode === "recent") sorted.sort((a, b) => Date.parse(b.note?.updated_at ?? "0") - Date.parse(a.note?.updated_at ?? "0"));
        else if (sortMode === "name") sorted.sort((a, b) => a.op.name.localeCompare(b.op.name));
        else if (sortMode === "rarity") sorted.sort((a, b) => b.op.rarity - a.op.rarity || a.op.name.localeCompare(b.op.name));
        return sorted;
    }, [combined, search, statusFilter, sortMode]);

    return (
        <>
            <PageHead
                kicker="Manage"
                title="Operator notes"
                sub={
                    <>
                        Community guidance attached to each operator. Edits write to <HCode>operator_notes</HCode> and append a diff to <HCode>operator_notes_audit</HCode>.
                    </>
                }
            />

            {editingId ? (
                <NoteEditor operatorId={editingId} onClose={() => setEditingId(null)} />
            ) : (
                <div className="relative overflow-hidden rounded-2xl border border-border bg-card shadow-xs/5 before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-2xl)-1px)] before:shadow-[0_1px_--theme(--color-black/4%)] dark:before:shadow-[0_-1px_--theme(--color-white/6%)]">
                    <div className="flex flex-wrap items-center gap-2.5 border-border border-b p-3.5">
                        <div className="w-full min-w-0 max-w-95 sm:min-w-70 sm:flex-1">
                            <InputGroup>
                                <InputGroupAddon>
                                    <SearchIcon />
                                </InputGroupAddon>
                                <Input placeholder="Search name, char_id, summary, or tag…" size="sm" value={search} onChange={(e) => setSearch(e.target.value)} />
                            </InputGroup>
                        </div>
                        <div className="inline-flex max-w-full gap-px overflow-x-auto rounded-[9px] border border-border bg-card p-0.75">
                            {(
                                [
                                    { value: "all", label: `All · ${counts.all}` },
                                    { value: "has-content", label: `Has content · ${counts.withContent}` },
                                    { value: "empty", label: `Empty · ${counts.empty}` },
                                ] as { value: StatusFilter; label: string }[]
                            ).map((it) => (
                                <button
                                    key={it.value}
                                    type="button"
                                    onClick={() => setStatusFilter(it.value)}
                                    className={cn("inline-flex h-6.5 shrink-0 cursor-pointer items-center whitespace-nowrap rounded-md px-3 font-medium text-[12.5px] transition-colors", statusFilter === it.value ? "bg-background text-foreground shadow-xs/5" : "text-muted-foreground hover:text-foreground")}
                                >
                                    {it.label}
                                </button>
                            ))}
                        </div>
                        <DropdownMenu>
                            <DropdownMenuTrigger
                                render={(triggerProps) => (
                                    <button {...triggerProps} type="button" className="inline-flex h-7 shrink-0 cursor-pointer items-center gap-1.5 rounded-md border border-border bg-card px-2.5 font-medium text-[12.5px] text-foreground hover:bg-accent">
                                        <ArrowUpDownIcon className="size-3.5 opacity-70" strokeWidth={1.9} />
                                        <span>{SORT_LABELS[sortMode]}</span>
                                    </button>
                                )}
                            />
                            <DropdownMenuContent align="end" className="w-48">
                                {(Object.keys(SORT_LABELS) as SortMode[]).map((m) => (
                                    <DropdownMenuItem key={m} className="cursor-pointer" onClick={() => setSortMode(m)}>
                                        {sortMode === m ? <CheckIcon className="mr-2 h-4 w-4 text-primary" /> : <span className="mr-2 inline-block h-4 w-4" />}
                                        {SORT_LABELS[m]}
                                    </DropdownMenuItem>
                                ))}
                            </DropdownMenuContent>
                        </DropdownMenu>
                        <div className="flex-1" />
                        <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                            {filtered.length} of {counts.all}
                        </span>
                    </div>
                    {notesQuery.isPending || opsQuery.isPending ? (
                        <div className="space-y-2 p-4">
                            <Skeleton className="h-14" />
                            <Skeleton className="h-14" />
                            <Skeleton className="h-14" />
                        </div>
                    ) : filtered.length === 0 ? (
                        <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">
                            {search || statusFilter !== "all" ? (
                                <>
                                    No notes match these filters.{" "}
                                    <button
                                        type="button"
                                        className="cursor-pointer text-foreground underline underline-offset-2 hover:text-primary"
                                        onClick={() => {
                                            setSearch("");
                                            setStatusFilter("all");
                                        }}
                                    >
                                        Reset
                                    </button>
                                </>
                            ) : (
                                "No operator notes yet."
                            )}
                        </div>
                    ) : (
                        <ul className="divide-y divide-border">
                            {filtered.map(({ operatorId, op, note }) => (
                                <li key={operatorId}>
                                    <NoteRow op={op} note={note} onOpen={() => setEditingId(operatorId)} />
                                </li>
                            ))}
                        </ul>
                    )}
                </div>
            )}
        </>
    );
}

function NoteRow({ op, note, onOpen }: { op: IOperatorIndexEntry; note: IOperatorNote | null; onOpen: () => void }): React.ReactElement {
    const hasContent = note ? noteHasContent(note) : false;
    const tags = note?.tags ?? [];
    const summary = note?.summary?.trim() || note?.notes?.trim().split("\n")[0] || null;
    return (
        <button type="button" onClick={onOpen} className="group flex w-full min-w-0 cursor-pointer items-start gap-3 px-3.5 py-3 text-left transition-colors hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] sm:px-4 sm:py-3.5">
            <span
                className="relative mt-0.5 inline-flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-md font-semibold text-[10px] sm:size-10"
                style={{
                    background: RARITY_BG[op.rarity] ?? RARITY_BG[6],
                    boxShadow: "inset 0 1px 0 oklch(1 0 0 / 0.25), inset 0 -1px 0 oklch(0 0 0 / 0.25)",
                }}
            >
                <OperatorAvatar charId={op.id} name={op.name} />
                <span aria-hidden className={cn("absolute right-0 bottom-0 left-0 h-0.5", hasContent ? "bg-success" : "bg-transparent")} />
            </span>

            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <div className="flex min-w-0 flex-wrap items-baseline gap-x-2 gap-y-0.5">
                    <span className="truncate font-medium text-[14px] text-foreground">{op.name}</span>
                    <span className="truncate font-mono text-[11px] text-muted-foreground">{op.id}</span>
                    <span className="hidden text-[11.5px] text-muted-foreground sm:inline">· {formatSubProfession(op.subProfessionId)}</span>
                </div>

                {summary ? <p className="line-clamp-2 text-[12.5px] text-muted-foreground leading-snug">{summary}</p> : <p className="text-[12px] text-muted-foreground/70 italic">No summary yet — click to add one.</p>}

                {tags.length > 0 ? (
                    <div className="mt-0.5 flex flex-wrap gap-1">
                        {tags.slice(0, 4).map((t) => (
                            <span key={t} className="inline-flex h-4.5 items-center rounded-sm border border-border bg-muted px-1.5 font-mono text-[10.5px] text-muted-foreground leading-none">
                                {t}
                            </span>
                        ))}
                        {tags.length > 4 ? <span className="font-mono text-[10.5px] text-muted-foreground">+{tags.length - 4}</span> : null}
                    </div>
                ) : null}
            </div>

            <div className="flex shrink-0 flex-col items-end gap-1.5 self-stretch">
                {hasContent ? <Badge variant="success">filled</Badge> : <Badge variant="outline">empty</Badge>}
                {note?.updated_at ? <span className="whitespace-nowrap font-mono text-[10.5px] text-muted-foreground">{formatRelative(note.updated_at)}</span> : null}
                <ChevronRightIcon className="mt-auto hidden size-4 text-muted-foreground opacity-0 transition-opacity group-hover:opacity-100 sm:block" strokeWidth={1.9} />
            </div>
        </button>
    );
}

function formatRelative(iso: string): string {
    const t = Date.parse(iso);
    if (!Number.isFinite(t)) return iso;
    const diff = (Date.now() - t) / 1000;
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)} h ago`;
    if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} d ago`;
    return new Date(t).toLocaleDateString();
}

function NoteEditor({ operatorId, onClose }: { operatorId: string; onClose: () => void }): React.ReactElement {
    const queryClient = useQueryClient();
    const opsQuery = useQuery(operatorsIndexQueryOptions());
    const noteQuery = useQuery(operatorNoteQueryOptions(operatorId));
    const auditQuery = useQuery(operatorNoteAuditLogQueryOptions(operatorId));

    const op = opsQuery.data?.find((o) => o.id === operatorId);
    const note = noteQuery.data ?? null;

    const [summary, setSummary] = useState<string>("");
    const [notes, setNotes] = useState<string>("");
    const [pros, setPros] = useState<string>("");
    const [cons, setCons] = useState<string>("");
    const [trivia, setTrivia] = useState<string>("");
    const [tags, setTags] = useState<string[]>([]);
    const [tagInput, setTagInput] = useState<string>("");
    const [dirtyBaseline, setDirtyBaseline] = useState<string>("");

    useEffect(() => {
        if (!note) return;
        setSummary(note.summary ?? "");
        setNotes(note.notes ?? "");
        setPros(note.pros ?? "");
        setCons(note.cons ?? "");
        setTrivia(note.trivia ?? "");
        setTags(note.tags ?? []);
        setDirtyBaseline(JSON.stringify({ summary: note.summary ?? "", notes: note.notes ?? "", pros: note.pros ?? "", cons: note.cons ?? "", trivia: note.trivia ?? "", tags: note.tags ?? [] }));
    }, [note]);

    const currentSnapshot = JSON.stringify({ summary, notes, pros, cons, trivia, tags });
    const dirty = currentSnapshot !== dirtyBaseline && dirtyBaseline !== "";

    const addTag = useCallback(() => {
        const trimmed = tagInput.trim();
        if (!trimmed) return;
        setTags((prev) => (prev.includes(trimmed) ? prev : [...prev, trimmed]));
        setTagInput("");
    }, [tagInput]);

    const removeTag = useCallback((tag: string) => {
        setTags((prev) => prev.filter((t) => t !== tag));
    }, []);

    const save = useMutation({
        mutationFn: (input: IUpdateOperatorNoteInput) => updateOperatorNoteFn({ data: input }),
        onSuccess: (data) => {
            void queryClient.invalidateQueries({ queryKey: ["operator-notes"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "operator-notes", "audit", operatorId] });
            setDirtyBaseline(JSON.stringify({ summary: data.summary ?? "", notes: data.notes ?? "", pros: data.pros ?? "", cons: data.cons ?? "", trivia: data.trivia ?? "", tags: data.tags ?? [] }));
            toastManager.add({ id: `note-save-${Date.now()}`, title: "Note saved", description: `Updated operator notes for ${op?.name ?? operatorId}.`, type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `note-save-err-${Date.now()}`, title: "Failed to save", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    if (noteQuery.isPending || opsQuery.isPending) {
        return <Skeleton className="h-150 w-full rounded-2xl" />;
    }

    return (
        <>
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2.5 text-sm">
                        <span
                            className="inline-flex size-8 items-center justify-center overflow-hidden rounded-md font-semibold text-[12px]"
                            style={{
                                background: RARITY_BG[op?.rarity ?? 6] ?? RARITY_BG[6],
                                boxShadow: "inset 0 1px 0 oklch(1 0 0 / 0.25), inset 0 -1px 0 oklch(0 0 0 / 0.25)",
                            }}
                        >
                            <OperatorAvatar charId={operatorId} name={op?.name ?? "?"} />
                        </span>
                        <span className="font-mono">{operatorId}</span> · {op?.name ?? "Operator"}
                    </CardTitle>
                    <CardDescription className="text-xs">
                        {note ? (
                            <>
                                Last updated {formatRelative(note.updated_at)} · {auditQuery.data?.length ?? 0} revision{(auditQuery.data?.length ?? 0) === 1 ? "" : "s"}
                            </>
                        ) : (
                            "New note - nothing saved yet."
                        )}
                    </CardDescription>
                    <CardAction>
                        <div className="flex items-center gap-2">
                            <Button variant="ghost" size="sm" onClick={onClose}>
                                Back
                            </Button>
                            <Button
                                size="sm"
                                disabled={!dirty || save.isPending}
                                loading={save.isPending}
                                onClick={() =>
                                    save.mutate({
                                        operatorId,
                                        summary: summary || null,
                                        notes: notes || null,
                                        pros: pros || null,
                                        cons: cons || null,
                                        trivia: trivia || null,
                                        tags,
                                    })
                                }
                            >
                                <CheckIcon />
                                Save
                            </Button>
                        </div>
                    </CardAction>
                </CardHeader>
                <div className="grid grid-cols-1 border-border border-t lg:grid-cols-2">
                    <div className="border-border border-b p-4 lg:border-r lg:border-b-0">
                        <Field label="Summary" hint="One-line synopsis for the operator card.">
                            <Input size="sm" value={summary} onChange={(e) => setSummary(e.target.value)} />
                        </Field>
                        <Field label="Pros">
                            <textarea value={pros} onChange={(e) => setPros(e.target.value)} className="h-20 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Cons">
                            <textarea value={cons} onChange={(e) => setCons(e.target.value)} className="h-20 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Notes">
                            <textarea value={notes} onChange={(e) => setNotes(e.target.value)} className="h-32 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Trivia">
                            <textarea value={trivia} onChange={(e) => setTrivia(e.target.value)} className="h-20 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Tags">
                            <div className="flex gap-2">
                                <Input
                                    size="sm"
                                    value={tagInput}
                                    onChange={(e) => setTagInput(e.target.value)}
                                    onKeyDown={(e) => {
                                        if (e.key === "Enter") {
                                            e.preventDefault();
                                            addTag();
                                        }
                                    }}
                                    placeholder="Add a tag…"
                                />
                                <Button type="button" variant="outline" size="sm" onClick={addTag} disabled={!tagInput.trim()}>
                                    <PlusIcon />
                                    Add
                                </Button>
                            </div>
                            {tags.length > 0 ? (
                                <div className="flex flex-wrap gap-1.5 pt-1.5">
                                    {tags.map((tag) => (
                                        <Badge key={tag} variant="secondary" className="gap-1 pr-1">
                                            <span>{tag}</span>
                                            <button type="button" onClick={() => removeTag(tag)} className="-mr-0.5 inline-flex size-3.5 cursor-pointer items-center justify-center rounded-sm hover:bg-foreground/10" aria-label={`Remove ${tag}`}>
                                                <XIcon className="size-3" strokeWidth={2.2} />
                                            </button>
                                        </Badge>
                                    ))}
                                </div>
                            ) : null}
                        </Field>
                    </div>
                    <div>
                        <div className="flex items-center justify-between border-border border-b px-3.5 py-2">
                            <MonoSection>Preview</MonoSection>
                            {dirty ? <Badge variant="warning">unsaved changes</Badge> : <Badge variant="outline">saved</Badge>}
                        </div>
                        <div className="max-h-[60vh] overflow-auto p-4 font-sans text-[13px] leading-[1.6] lg:h-160 lg:max-h-none">
                            {summary ? <p className="mb-3 whitespace-pre-line font-medium">{summary}</p> : null}
                            {pros ? (
                                <>
                                    <h3 className="mt-2 mb-1.5 font-semibold text-[14px]">Pros</h3>
                                    <p className="whitespace-pre-line text-muted-foreground">{pros}</p>
                                </>
                            ) : null}
                            {cons ? (
                                <>
                                    <h3 className="mt-3 mb-1.5 font-semibold text-[14px]">Cons</h3>
                                    <p className="whitespace-pre-line text-muted-foreground">{cons}</p>
                                </>
                            ) : null}
                            {notes ? (
                                <>
                                    <h3 className="mt-3 mb-1.5 font-semibold text-[14px]">Notes</h3>
                                    <p className="whitespace-pre-line text-muted-foreground">{notes}</p>
                                </>
                            ) : null}
                            {trivia ? (
                                <>
                                    <h3 className="mt-3 mb-1.5 font-semibold text-[14px]">Trivia</h3>
                                    <p className="whitespace-pre-line text-muted-foreground">{trivia}</p>
                                </>
                            ) : null}
                            {tags.length > 0 ? (
                                <div className="mt-4 flex flex-wrap gap-2">
                                    {tags.map((tag) => (
                                        <Badge key={tag} variant="secondary">
                                            {tag}
                                        </Badge>
                                    ))}
                                </div>
                            ) : null}
                            {!summary && !pros && !cons && !notes && !trivia && tags.length === 0 ? <p className="text-muted-foreground">Nothing yet - start writing on the left.</p> : null}
                        </div>
                    </div>
                </div>
            </Card>

            <div className="h-4" />

            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2 text-sm">
                        <ActivityIcon className="size-4" />
                        Revision history
                    </CardTitle>
                    <CardDescription className="text-xs">
                        Each save appends a row to <HCode>operator_notes_audit</HCode> with a JWT-signed actor stamp.
                    </CardDescription>
                </CardHeader>
                <div className="p-4">
                    {auditQuery.isPending ? (
                        <Skeleton className="h-32 w-full" />
                    ) : (auditQuery.data?.length ?? 0) === 0 ? (
                        <div className="py-6 text-center text-[13px] text-muted-foreground">No revisions yet.</div>
                    ) : (
                        <div className="relative pl-4">
                            <div className="absolute top-1 bottom-1 left-1 w-0.5 rounded bg-border" />
                            {(auditQuery.data ?? []).slice(0, 20).map((rev, i) => (
                                <div key={rev.id} className="relative pb-3.5 last:pb-0">
                                    <span className={`absolute top-1 -left-4.5 size-2.5 rounded-full border-2 bg-card shadow-[0_0_0_3px_var(--background)] ${i === 0 ? "border-primary" : "border-border"}`} />
                                    <div className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.06em]">
                                        {formatRelative(rev.changed_at)} · {rev.field_name}
                                    </div>
                                    <div className="mt-1 text-[12.5px] leading-normal">
                                        <span className="font-mono text-muted-foreground">UID&nbsp;{rev.changed_by}</span> updated <span className="font-mono">{rev.field_name}</span>
                                    </div>
                                    {rev.old_value !== null || rev.new_value !== null ? (
                                        <div className="mt-1.5 rounded-lg border border-border bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] p-2 font-mono text-[11.5px] leading-normal">
                                            {rev.old_value ? <div className="rounded-[3px] bg-destructive/8 px-1 text-destructive-foreground line-through decoration-destructive/40">{rev.old_value.slice(0, 240)}</div> : null}
                                            {rev.new_value ? <div className="rounded-[3px] bg-emerald-500/8 px-1 text-emerald-700 dark:text-emerald-300">{rev.new_value.slice(0, 240)}</div> : null}
                                        </div>
                                    ) : null}
                                </div>
                            ))}
                        </div>
                    )}
                </div>
            </Card>
        </>
    );
}

function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="mb-3 flex flex-col gap-1">
            <span className="font-medium text-[12px]">{label}</span>
            {children}
            {hint ? <span className="text-[11px] text-muted-foreground">{hint}</span> : null}
        </div>
    );
}
