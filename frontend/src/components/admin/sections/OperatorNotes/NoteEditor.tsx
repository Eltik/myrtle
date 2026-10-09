import { useQuery } from "@tanstack/react-query";
import { useId, useState } from "react";
import { changeKind } from "#/components/admin/shell/model";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Skeleton } from "#/components/ui/skeleton";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { Textarea } from "#/components/ui/textarea";
import { type IOperatorNoteAuditEntry, operatorNoteAuditLogQueryOptions } from "#/lib/api/admin";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { Markdown } from "#/lib/markdown";
import { cn } from "#/lib/utils";
import { addTag, draftFromNote, draftHasContent, draftStatus, draftsEqual, type INoteDraft, type INoteRow, isNoteField, missingFields, type NoteField, SUMMARY_LIMIT } from "./notesModel";
import type { messages } from "./OperatorNotes.messages";
import { RarityTile } from "./RarityTile";

type NotesT = TypedT<typeof messages>;

const REVISIONS_SHOWN = 4;

export interface INoteEditorProps {
    row: INoteRow;
    /** Gate for the revision query (the role may edit notes). */
    enabled: boolean;
    saving: boolean;
    /** Saves the draft; resolves true on success. `next` asks for Save & next (the next note not done). */
    onSave: (row: INoteRow, draft: INoteDraft, next: boolean) => Promise<boolean>;
}

function fieldName(t: NotesT, field: NoteField): string {
    switch (field) {
        case "summary":
            return t("notes.fieldName.summary");
        case "pros":
            return t("notes.fieldName.pros");
        case "cons":
            return t("notes.fieldName.cons");
        case "notes":
            return t("notes.fieldName.notes");
        case "trivia":
            return t("notes.fieldName.trivia");
        case "tags":
            return t("notes.fieldName.tags");
    }
}

/** "Pros, Cons" in the UI locale; a plain comma join where Intl.ListFormat is missing. */
function formatList(locale: string, items: string[]): string {
    try {
        return new Intl.ListFormat(locale, { type: "unit", style: "short" }).format(items);
    } catch {
        return items.join(", ");
    }
}

/**
 * The detail pane: one operator's note as a draft, saved with a single PUT.
 * Mounted with `key={operatorId}`, so switching operators drops the draft.
 */
export function NoteEditor({ row, enabled, saving, onSave }: INoteEditorProps): React.ReactElement {
    const t: NotesT = useT("admin");
    const f = useFormatters();
    const ids = useId();
    const base = draftFromNote(row.note);
    const [edit, setEdit] = useState<INoteDraft | null>(null);
    const [tagInput, setTagInput] = useState("");
    const [mode, setMode] = useState<"write" | "preview">("write");
    const draft = edit ?? base;
    const dirty = edit !== null && !draftsEqual(edit, base);
    const summaryOver = draft.summary.length > SUMMARY_LIMIT;
    const locale = useLocale();
    // Follows the draft, so the line shrinks as fields are written. Empty and
    // done notes show nothing: the list badge already says so.
    const missing = draftStatus(draft) === "incomplete" ? missingFields(draft) : [];
    const missingList = formatList(
        locale,
        missing.map((field) => fieldName(t, field)),
    );

    const set = <K extends keyof INoteDraft>(field: K, value: INoteDraft[K]) => setEdit({ ...draft, [field]: value });
    const reset = () => {
        setEdit(null);
        setTagInput("");
    };
    const submit = async (next: boolean) => {
        if (await onSave(row, draft, next)) reset();
    };

    const rarity = f.number(row.rarity);
    const meta = row.note ? t("notes.detail.metaUpdated", { rarity, id: row.id, when: f.relativeShort(row.note.updated_at) }) : t("notes.detail.meta", { rarity, id: row.id });
    const actionsDisabled = !dirty || saving;
    const saveDisabled = actionsDisabled || summaryOver;

    const actions = (
        <>
            <Button size="sm" variant="ghost" disabled={actionsDisabled} onClick={reset}>
                {t("notes.detail.reset")}
            </Button>
            <Button size="sm" variant="outline" disabled={saveDisabled} onClick={() => void submit(false)}>
                {t("notes.detail.save")}
            </Button>
            <Button size="sm" disabled={saveDisabled} loading={saving} onClick={() => void submit(true)}>
                {t("notes.detail.saveNext")}
            </Button>
        </>
    );

    // A size container: the Pros/Cons pair goes side by side when the pane (not the viewport) has room.
    return (
        <div className="@container">
            <div className="flex flex-wrap items-center gap-3 border-border border-b px-5 py-4">
                <RarityTile id={row.id} name={row.name} rarity={row.rarity} size="lg" />
                <div className="flex min-w-40 flex-1 flex-col gap-[3px]">
                    <span className="font-semibold text-[17px] leading-[1.2]">{row.name}</span>
                    <span className="text-[12.5px] text-muted-foreground">{meta}</span>
                    {missing.length > 0 ? <span className="text-[12.5px] text-muted-foreground">{t("notes.detail.missing", { fields: missingList })}</span> : null}
                </div>
                <div className="flex items-center gap-2 max-lg:hidden">
                    {dirty ? <span className="whitespace-nowrap text-[12.5px] text-warning-foreground">{t("notes.detail.unsaved")}</span> : null}
                    {actions}
                </div>
            </div>

            <div className="flex flex-col gap-3.5 px-5 pt-4 pb-5">
                <Tabs value={mode} onValueChange={(v) => setMode(v as "write" | "preview")} className="self-end">
                    <TabsList aria-label={t("notes.mode.label")}>
                        <TabsTab value="write" className="h-6 pointer-coarse:h-9 pointer-coarse:px-3.5 px-2.5 text-[12.5px]">
                            {t("notes.mode.write")}
                        </TabsTab>
                        <TabsTab value="preview" className="h-6 pointer-coarse:h-9 pointer-coarse:px-3.5 px-2.5 text-[12.5px]">
                            {t("notes.mode.preview")}
                        </TabsTab>
                    </TabsList>
                </Tabs>

                {mode === "write" ? (
                    <>
                        <div className="flex flex-col gap-1.5">
                            <div className="flex justify-between gap-2">
                                <Label htmlFor={`${ids}-summary`}>{t("notes.form.summary")}</Label>
                                <span className={cn("text-[12px] tabular-nums", summaryOver ? "text-destructive" : "text-muted-foreground")}>{t("notes.form.summaryCount", { count: f.number(draft.summary.length), limit: f.number(SUMMARY_LIMIT) })}</span>
                            </div>
                            <Input id={`${ids}-summary`} value={draft.summary} onChange={(e) => set("summary", e.target.value)} placeholder={t("notes.form.summaryPlaceholder")} aria-invalid={summaryOver || undefined} />
                        </div>
                        <div className="grid @xl:grid-cols-[repeat(2,minmax(0,1fr))] grid-cols-[minmax(0,1fr)] gap-3.5">
                            <TextField id={`${ids}-pros`} label={t("notes.form.pros")} value={draft.pros} rows={3} placeholder={t("notes.form.prosPlaceholder")} onChange={(v) => set("pros", v)} />
                            <TextField id={`${ids}-cons`} label={t("notes.form.cons")} value={draft.cons} rows={3} placeholder={t("notes.form.consPlaceholder")} onChange={(v) => set("cons", v)} />
                        </div>
                        <TextField id={`${ids}-notes`} label={t("notes.form.notes")} value={draft.notes} rows={3} placeholder={t("notes.form.notesPlaceholder")} onChange={(v) => set("notes", v)} />
                        <TextField id={`${ids}-trivia`} label={t("notes.form.trivia")} value={draft.trivia} rows={2} placeholder={t("notes.form.triviaPlaceholder")} onChange={(v) => set("trivia", v)} />
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor={`${ids}-tag`}>{t("notes.form.tags")}</Label>
                            <div className="flex flex-wrap items-center gap-1.5">
                                {draft.tags.map((tag) => (
                                    <button
                                        key={tag}
                                        type="button"
                                        title={t("notes.form.removeTag")}
                                        aria-label={t("notes.form.removeTagNamed", { tag })}
                                        onClick={() =>
                                            set(
                                                "tags",
                                                draft.tags.filter((x) => x !== tag),
                                            )
                                        }
                                        className="flex h-[26px] pointer-coarse:h-9 cursor-pointer items-center gap-1.5 pointer-coarse:rounded-[18px] rounded-[13px] border border-border bg-card pointer-coarse:pr-3 pr-2 pl-2.5 pointer-coarse:pl-3.5 text-[12.5px] transition-colors hover:border-destructive"
                                    >
                                        {tag} <span className="text-muted-foreground">×</span>
                                    </button>
                                ))}
                                <div className="w-[180px]">
                                    <Input
                                        id={`${ids}-tag`}
                                        size="sm"
                                        value={tagInput}
                                        onChange={(e) => setTagInput(e.target.value)}
                                        onKeyDown={(e) => {
                                            if (e.key !== "Enter" || e.nativeEvent.isComposing) return;
                                            e.preventDefault();
                                            const tags = addTag(draft.tags, tagInput);
                                            if (tags !== draft.tags) set("tags", tags);
                                            setTagInput("");
                                        }}
                                        placeholder={t("notes.form.tagPlaceholder")}
                                    />
                                </div>
                            </div>
                        </div>
                    </>
                ) : (
                    <NotePreview draft={draft} />
                )}

                <Revisions operatorId={row.id} enabled={enabled} />

                {/* Below lg (where the editor fills the screen) the actions stick to the bottom, spanning the card and clearing the iOS home bar. */}
                <div className="sticky bottom-0 z-1 -mx-5 flex flex-wrap items-center gap-2 border-border border-t bg-card px-5 pt-3 pb-[max(0.25rem,env(safe-area-inset-bottom))] lg:hidden">
                    {dirty ? <span className="w-full text-[12.5px] text-warning-foreground">{t("notes.detail.unsaved")}</span> : null}
                    {actions}
                </div>
            </div>
        </div>
    );
}

function TextField({ id, label, value, rows, placeholder, onChange }: { id: string; label: string; value: string; rows: number; placeholder: string; onChange: (value: string) => void }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5">
            <Label htmlFor={id}>{label}</Label>
            <Textarea id={id} value={value} rows={rows} placeholder={placeholder} onChange={(e) => onChange(e.target.value)} />
        </div>
    );
}

/** The note rendered as Markdown, the way the operator's page shows it. */
function NotePreview({ draft }: { draft: INoteDraft }): React.ReactElement {
    const t: NotesT = useT("admin");
    if (!draftHasContent(draft)) return <p className="py-6 text-[13px] text-muted-foreground">{t("notes.previewPane.nothing")}</p>;
    const sections: [string, string][] = [
        [t("notes.form.pros"), draft.pros],
        [t("notes.form.cons"), draft.cons],
        [t("notes.form.notes"), draft.notes],
        [t("notes.form.trivia"), draft.trivia],
    ];
    return (
        <div className="flex flex-col gap-3 rounded-xl border border-border p-4 text-[13px] leading-[1.6]">
            {draft.summary.trim() ? <p className="font-medium">{draft.summary}</p> : null}
            {sections.map(([label, text]) =>
                text.trim() ? (
                    <div key={label} className="flex flex-col gap-1">
                        <h3 className="font-semibold text-[14px]">{label}</h3>
                        <Markdown text={text} flush className="text-muted-foreground" />
                    </div>
                ) : null,
            )}
            {draft.tags.length > 0 ? (
                <div className="flex flex-wrap gap-1.5">
                    {draft.tags.map((tag) => (
                        <Badge key={tag} variant="secondary">
                            {tag}
                        </Badge>
                    ))}
                </div>
            ) : null}
        </div>
    );
}

function Revisions({ operatorId, enabled }: { operatorId: string; enabled: boolean }): React.ReactElement {
    const t: NotesT = useT("admin");
    const auditQuery = useQuery({ ...operatorNoteAuditLogQueryOptions(operatorId), enabled });
    const entries = auditQuery.data ?? [];
    return (
        <div className="flex flex-col gap-2 border-border border-t pt-3.5">
            <span className="font-medium text-[13px]">{entries.length > 0 ? t("notes.revisions.titleCount", { count: entries.length }) : t("notes.revisions.title")}</span>
            {auditQuery.isPending && enabled ? (
                <Skeleton className="h-5 w-64" />
            ) : auditQuery.isError ? (
                <span className="text-[13px] text-muted-foreground">{t("notes.revisions.loadFailed")}</span>
            ) : entries.length === 0 ? (
                <span className="text-[13px] text-muted-foreground">{t("notes.revisions.empty")}</span>
            ) : (
                entries.slice(0, REVISIONS_SHOWN).map((entry) => <RevisionRow key={entry.id} entry={entry} />)
            )}
        </div>
    );
}

function RevisionRow({ entry }: { entry: IOperatorNoteAuditEntry }): React.ReactElement {
    const t: NotesT = useT("admin");
    const f = useFormatters();
    const verb = changeKind(entry.old_value, entry.new_value);
    return (
        <div className="flex min-w-0 items-center gap-2.5 text-[13px]">
            <span className="w-[70px] shrink-0 text-muted-foreground">{f.relativeShort(entry.changed_at)}</span>
            <span className="min-w-0 truncate font-medium">{entry.actor.nickname ?? t("notes.revisions.unknownActor")}</span>
            <span className="shrink-0 text-muted-foreground">{verb === "added" ? t("notes.revisions.added") : verb === "cleared" ? t("notes.revisions.cleared") : t("notes.revisions.changed")}</span>
            <Badge variant="outline" size="sm">
                {isNoteField(entry.field_name) ? fieldName(t, entry.field_name) : entry.field_name}
            </Badge>
        </div>
    );
}
