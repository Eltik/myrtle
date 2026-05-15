import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ActivityIcon, CheckIcon, EditIcon, SearchIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { InputGroup, InputGroupAddon } from "#/components/ui/input-group";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { toastManager } from "#/components/ui/toast";
import { type IUpdateOperatorNoteInput, operatorNoteAuditLogQueryOptions, updateOperatorNoteFn } from "#/lib/api/admin";
import { type IOperatorNote, operatorNoteQueryOptions, operatorNotesListQueryOptions } from "#/lib/api/operator-notes";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import type { IOperatorIndexEntry } from "#/types/operators";
import { HCode, PageHead } from "../AdminShell";
import { RARITY_BG } from "../data";
import { MonoSection } from "../Primitives";

interface INoteCombined {
    operatorId: string;
    op: IOperatorIndexEntry;
    note: IOperatorNote | null;
}

export function OperatorNotes(): React.ReactElement {
    const opsQuery = useQuery(operatorsIndexQueryOptions());
    const notesQuery = useQuery(operatorNotesListQueryOptions());
    const [editingId, setEditingId] = useState<string | null>(null);
    const [search, setSearch] = useState("");

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
            })
            .sort((a, b) => Date.parse(b.note?.updated_at ?? "0") - Date.parse(a.note?.updated_at ?? "0"));
    }, [notesQuery.data, opsQuery.data]);

    const filtered = useMemo(() => combined.filter((c) => !search || c.op.name.toLowerCase().includes(search.toLowerCase()) || c.operatorId.toLowerCase().includes(search.toLowerCase())), [combined, search]);

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
                    <div className="flex items-center gap-2.5 border-b border-border p-3.5">
                        <div className="min-w-[280px] max-w-[380px] flex-1">
                            <InputGroup>
                                <InputGroupAddon>
                                    <SearchIcon />
                                </InputGroupAddon>
                                <Input placeholder="Search by operator name or char_id…" size="sm" value={search} onChange={(e) => setSearch(e.target.value)} />
                            </InputGroup>
                        </div>
                        <div className="flex-1" />
                        <span className="text-[12px] text-muted-foreground">
                            {filtered.length} note{filtered.length === 1 ? "" : "s"}
                        </span>
                    </div>
                    {notesQuery.isPending || opsQuery.isPending ? (
                        <div className="space-y-2 p-4">
                            <Skeleton className="h-12" />
                            <Skeleton className="h-12" />
                            <Skeleton className="h-12" />
                        </div>
                    ) : filtered.length === 0 ? (
                        <div className="px-3.5 py-16 text-center text-[13px] text-muted-foreground">No operator notes yet.</div>
                    ) : (
                        <table className="w-full border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {["Operator", "Branch", "Status", "Updated", "Tags", ""].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-mono font-medium text-[11px] uppercase tracking-[0.08em] text-muted-foreground">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {filtered.map(({ operatorId, op, note }) => (
                                    <tr key={operatorId} onClick={() => setEditingId(operatorId)} onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && setEditingId(operatorId)} className="cursor-pointer border-b border-border last:border-0 hover:bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)]">
                                        <td className="px-3.5 py-2.5">
                                            <span
                                                className="mr-2 inline-flex size-7 items-center justify-center overflow-hidden rounded-md align-[-8px] text-[10px] font-semibold"
                                                style={{
                                                    background: RARITY_BG[op.rarity] ?? RARITY_BG[6],
                                                    boxShadow: "inset 0 1px 0 oklch(1 0 0 / 0.25), inset 0 -1px 0 oklch(0 0 0 / 0.25)",
                                                }}
                                            >
                                                <OperatorAvatar charId={operatorId} name={op.name} />
                                            </span>
                                            <span className="font-medium">{op.name}</span>
                                            <span className="font-mono text-[11.5px] text-muted-foreground"> · {operatorId}</span>
                                        </td>
                                        <td className="px-3.5 py-2.5 text-muted-foreground">{op.subProfessionId}</td>
                                        <td className="px-3.5 py-2.5">{note?.summary || note?.notes ? <Badge variant="success">has content</Badge> : <Badge variant="outline">empty</Badge>}</td>
                                        <td className="px-3.5 py-2.5 text-muted-foreground">{note?.updated_at ? formatRelative(note.updated_at) : "—"}</td>
                                        <td className="px-3.5 py-2.5 text-[12.5px] text-muted-foreground">{(note?.tags ?? []).slice(0, 3).join(", ") || "—"}</td>
                                        <td className="px-3.5 py-2.5">
                                            <Button variant="ghost" size="xs">
                                                <EditIcon />
                                                Edit
                                            </Button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    )}
                </div>
            )}
        </>
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
    const [tagsRaw, setTagsRaw] = useState<string>("");
    const [dirtyBaseline, setDirtyBaseline] = useState<string>("");

    useEffect(() => {
        if (!note) return;
        setSummary(note.summary ?? "");
        setNotes(note.notes ?? "");
        setPros(note.pros ?? "");
        setCons(note.cons ?? "");
        setTrivia(note.trivia ?? "");
        setTagsRaw((note.tags ?? []).join(", "));
        setDirtyBaseline(JSON.stringify({ summary: note.summary ?? "", notes: note.notes ?? "", pros: note.pros ?? "", cons: note.cons ?? "", trivia: note.trivia ?? "", tags: (note.tags ?? []).join(", ") }));
    }, [note]);

    const currentSnapshot = JSON.stringify({ summary, notes, pros, cons, trivia, tags: tagsRaw });
    const dirty = currentSnapshot !== dirtyBaseline && dirtyBaseline !== "";

    const save = useMutation({
        mutationFn: (input: IUpdateOperatorNoteInput) => updateOperatorNoteFn({ data: input }),
        onSuccess: (data) => {
            void queryClient.invalidateQueries({ queryKey: ["operator-notes"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "operator-notes", "audit", operatorId] });
            setDirtyBaseline(JSON.stringify({ summary: data.summary ?? "", notes: data.notes ?? "", pros: data.pros ?? "", cons: data.cons ?? "", trivia: data.trivia ?? "", tags: (data.tags ?? []).join(", ") }));
            toastManager.add({ id: `note-save-${Date.now()}`, title: "Note saved", description: `Updated operator notes for ${op?.name ?? operatorId}.`, type: "success" });
        },
        onError: (err: unknown) => toastManager.add({ id: `note-save-err-${Date.now()}`, title: "Failed to save", description: err instanceof Error ? err.message : String(err), type: "error" }),
    });

    if (noteQuery.isPending || opsQuery.isPending) {
        return <Skeleton className="h-[600px] w-full rounded-2xl" />;
    }

    return (
        <>
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2.5 text-sm">
                        <span
                            className="inline-flex size-8 items-center justify-center overflow-hidden rounded-md text-[12px] font-semibold"
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
                            "New note — nothing saved yet."
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
                                        tags: tagsRaw
                                            .split(",")
                                            .map((t) => t.trim())
                                            .filter((t) => t.length > 0),
                                    })
                                }
                            >
                                <CheckIcon />
                                Save
                            </Button>
                        </div>
                    </CardAction>
                </CardHeader>
                <div className="grid grid-cols-2 border-t border-border">
                    <div className="border-r border-border p-4">
                        <Field label="Summary" hint="One-line synopsis for the operator card.">
                            <Input size="sm" value={summary} onChange={(e) => setSummary(e.target.value)} />
                        </Field>
                        <Field label="Pros" hint="Newline-separated.">
                            <textarea value={pros} onChange={(e) => setPros(e.target.value)} className="h-20 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Cons" hint="Newline-separated.">
                            <textarea value={cons} onChange={(e) => setCons(e.target.value)} className="h-20 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Notes" hint="Long-form guidance. Markdown supported in the public view.">
                            <textarea value={notes} onChange={(e) => setNotes(e.target.value)} className="h-32 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Trivia" hint="Optional lore / community notes.">
                            <textarea value={trivia} onChange={(e) => setTrivia(e.target.value)} className="h-20 w-full resize-none rounded-lg border border-input bg-background p-2 font-mono text-[12.5px] text-foreground outline-none focus:border-ring" />
                        </Field>
                        <Field label="Tags" hint="Comma-separated.">
                            <Input size="sm" value={tagsRaw} onChange={(e) => setTagsRaw(e.target.value)} placeholder="dps, e2-priority, sp-on-attack" />
                        </Field>
                    </div>
                    <div>
                        <div className="flex items-center justify-between border-b border-border px-3.5 py-2">
                            <MonoSection>Preview</MonoSection>
                            {dirty ? <Badge variant="warning">unsaved changes</Badge> : <Badge variant="outline">saved</Badge>}
                        </div>
                        <div className="h-[640px] overflow-auto p-4 font-sans text-[13px] leading-[1.6]">
                            {summary ? <p className="mb-3 font-medium">{summary}</p> : null}
                            {pros ? (
                                <>
                                    <h3 className="mt-2 mb-1.5 font-semibold text-[14px]">Pros</h3>
                                    <ul className="m-0 list-disc pl-4">
                                        {pros
                                            .split("\n")
                                            .filter(Boolean)
                                            .map((p, i) => (
                                                // biome-ignore lint/suspicious/noArrayIndexKey: pros lines are positional within the textarea
                                                <li key={i}>{p}</li>
                                            ))}
                                    </ul>
                                </>
                            ) : null}
                            {cons ? (
                                <>
                                    <h3 className="mt-3 mb-1.5 font-semibold text-[14px]">Cons</h3>
                                    <ul className="m-0 list-disc pl-4">
                                        {cons
                                            .split("\n")
                                            .filter(Boolean)
                                            .map((p, i) => (
                                                // biome-ignore lint/suspicious/noArrayIndexKey: cons lines are positional within the textarea
                                                <li key={i}>{p}</li>
                                            ))}
                                    </ul>
                                </>
                            ) : null}
                            {notes ? (
                                <>
                                    <h3 className="mt-3 mb-1.5 font-semibold text-[14px]">Notes</h3>
                                    <p className="whitespace-pre-wrap">{notes}</p>
                                </>
                            ) : null}
                            {trivia ? (
                                <>
                                    <h3 className="mt-3 mb-1.5 font-semibold text-[14px]">Trivia</h3>
                                    <p className="whitespace-pre-wrap">{trivia}</p>
                                </>
                            ) : null}
                            {!summary && !pros && !cons && !notes && !trivia ? <p className="text-muted-foreground">Nothing yet — start writing on the left.</p> : null}
                        </div>
                    </div>
                </div>
            </Card>

            <div className="h-4" />

            <Card>
                <CardHeader>
                    <CardTitle className="text-sm flex items-center gap-2">
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
                                    <span className={`absolute top-1 left-[-18px] size-2.5 rounded-full border-2 bg-card shadow-[0_0_0_3px_var(--background)] ${i === 0 ? "border-primary" : "border-border"}`} />
                                    <div className="font-mono font-medium text-[10.5px] uppercase tracking-[0.06em] text-muted-foreground">
                                        {formatRelative(rev.changed_at)} · {rev.field_name}
                                    </div>
                                    <div className="mt-1 text-[12.5px] leading-[1.5]">
                                        <span className="font-mono text-muted-foreground">UID&nbsp;{rev.changed_by}</span> updated <span className="font-mono">{rev.field_name}</span>
                                    </div>
                                    {rev.old_value !== null || rev.new_value !== null ? (
                                        <div className="mt-1.5 rounded-[8px] border border-border bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] p-2 font-mono text-[11.5px] leading-[1.5]">
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
