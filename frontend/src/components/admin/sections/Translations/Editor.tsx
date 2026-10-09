import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeftIcon, EraserIcon, HistoryIcon, MoreHorizontalIcon, RotateCcwIcon } from "lucide-react";
import { useId, useMemo, useState } from "react";
import { invalidateTranslationQueries } from "#/components/admin/shell/invalidate";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { Kbd } from "#/components/ui/kbd";
import { Label } from "#/components/ui/label";
import { Menu, MenuItem, MenuPopup, MenuSeparator, MenuTrigger } from "#/components/ui/menu";
import { Textarea } from "#/components/ui/textarea";
import { useAuth } from "#/hooks/use-auth";
import { useIsMac } from "#/hooks/use-is-mac";
import { clearTranslationFn, type ITranslationFieldError, type IUpdateTranslationInput, translationEntryAuditQueryOptions, updateTranslationFn } from "#/lib/api/admin";
import { describeMessage, type IMessageArgument, useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Locale } from "#/types/generated/Locale";
import type { TranslationEntry } from "#/types/generated/TranslationEntry";
import { ArgumentCard } from "./ArgumentCard";
import { HistorySheet } from "./HistorySheet";
import { entryStatus, isBranching, missingPlaceholders, sourceArguments } from "./model";
import { StatusBadge } from "./StatusBadge";
import { diffWords } from "./sourceDiff";
import type { messages } from "./Translations.messages";

type T = TypedT<typeof messages>;

export interface IEditorProps {
    entry: TranslationEntry;
    locale: Locale;
    canEdit: boolean;
    /** Where the entry sits in the loaded list; `null` when it was opened from a link and is not listed. */
    position: { index: number; total: number } | null;
    /** Where Save & next and Skip land. */
    nextKey: string | undefined;
    skipKey: string | undefined;
    onGo: (key: string | undefined) => void;
    /** Phones only: back to the list. */
    onBack: () => void;
}

/**
 * The detail card. Mounted with `key={locale:key}` by the section, so the
 * draft resets whenever the selection moves.
 */
export function Editor({ entry, locale, canEdit, position, nextKey, skipKey, onGo, onBack }: IEditorProps): React.ReactElement {
    const t: T = useT("admin");
    const fmt = useFormatters();
    const uiLocale = useLocale();
    const isMac = useIsMac();
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const { isAuthenticated } = useAuth();
    const fieldId = useId();

    const saved = entry.value ?? "";
    const [draft, setDraft] = useState(saved);
    const [fieldErrors, setFieldErrors] = useState<ITranslationFieldError[]>([]);
    const [historyOpen, setHistoryOpen] = useState(false);

    // A revert or a clear changes the stored value under the open editor. An
    // untouched draft follows it; one with unsaved typing keeps that typing.
    const [lastSaved, setLastSaved] = useState(saved);
    if (saved !== lastSaved) {
        setLastSaved(saved);
        if (draft === lastSaved) setDraft(saved);
    }

    const status = entryStatus(entry);
    const dirty = draft !== saved;
    const args = useMemo(() => sourceArguments(entry.source_text, entry.placeholders), [entry.source_text, entry.placeholders]);
    const names = useMemo(() => args.map((a) => a.name), [args]);
    const missing = useMemo(() => missingPlaceholders(names, draft), [names, draft]);
    const branching = useMemo(() => args.filter(isBranching), [args]);
    const written = useMemo(() => (branching.length > 0 ? new Map(describeMessage(draft).map((a) => [a.name, a])) : new Map<string, IMessageArgument>()), [branching.length, draft]);
    const started = draft.trim().length > 0;
    const canSave = canEdit && started && missing.length === 0;

    const auditQuery = useQuery({ ...translationEntryAuditQueryOptions({ locale: locale.code, key: entry.key }, isAuthenticated), enabled: isAuthenticated });
    const revisions = auditQuery.data?.length ?? 0;

    const invalidate = () => invalidateTranslationQueries(queryClient);

    const save = useMutation({
        mutationFn: (input: IUpdateTranslationInput) => updateTranslationFn({ data: input }),
        onSuccess: (result) => {
            if (!result.ok) {
                setFieldErrors(result.details);
                return;
            }
            invalidate();
            toastSuccess("translations-save", t("translations.toast.saved"), t("translations.toast.saved.desc", { key: entry.key, language: locale.english_name }));
            onGo(nextKey);
        },
        onError: (err: unknown) => toastError("translations-save-err", t("translations.toast.saveFailed"), describeError(err)),
    });

    const clear = useMutation({
        mutationFn: () => clearTranslationFn({ data: { locale: locale.code, key: entry.key } }),
        onSuccess: () => {
            setDraft("");
            setFieldErrors([]);
            invalidate();
            toastSuccess("translations-clear", t("translations.toast.cleared"), t("translations.toast.cleared.desc", { key: entry.key, language: locale.english_name }));
        },
        onError: (err: unknown) => toastError("translations-clear-err", t("translations.toast.clearFailed"), describeError(err)),
    });

    const saveNext = () => {
        if (!canSave || save.isPending) return;
        save.mutate({ locale: locale.code, key: entry.key, value: draft });
    };

    const missingList = useMemo(() => {
        const braced = missing.map((n) => `{${n}}`);
        try {
            return new Intl.ListFormat(uiLocale, { type: "conjunction" }).format(braced);
        } catch {
            return braced.join(", ");
        }
    }, [missing, uiLocale]);

    const footer = [
        position ? t("translations.editor.position", { index: fmt.number(position.index + 1), total: fmt.number(position.total) }) : null,
        entry.updated_at ? t("translations.editor.lastSaved", { when: fmt.relativeShort(entry.updated_at) }) : t("translations.editor.never"),
        revisions > 0 ? t("translations.editor.revisions", { count: revisions }) : null,
    ]
        .filter(Boolean)
        .join(" · ");

    return (
        <Card>
            <div className="px-3 pt-3 lg:hidden">
                <Button size="sm" variant="ghost" onClick={onBack}>
                    <ArrowLeftIcon />
                    {t("translations.editor.back")}
                </Button>
            </div>
            <CardHeader>
                <CardDescription className="min-w-0 font-mono text-[12.5px] [overflow-wrap:anywhere]">{entry.key}</CardDescription>
                <CardTitle className="text-base">{t("translations.editor.title", { english: locale.english_name, native: locale.native_name })}</CardTitle>
                <CardAction className="items-center gap-1">
                    <StatusBadge status={status} />
                    <Menu>
                        <MenuTrigger render={<Button size="icon-sm" variant="ghost" aria-label={t("translations.editor.more")} />}>
                            <MoreHorizontalIcon />
                        </MenuTrigger>
                        <MenuPopup align="end" className="w-52">
                            <MenuItem onClick={() => setHistoryOpen(true)}>
                                <HistoryIcon />
                                {t("translations.editor.history")}
                            </MenuItem>
                            {canEdit ? (
                                <>
                                    <MenuItem
                                        disabled={!dirty}
                                        onClick={() => {
                                            setDraft(saved);
                                            setFieldErrors([]);
                                        }}
                                    >
                                        <RotateCcwIcon />
                                        {t("translations.editor.discard")}
                                    </MenuItem>
                                    <MenuSeparator />
                                    <MenuItem variant="destructive" disabled={entry.value === null || clear.isPending} onClick={() => clear.mutate()}>
                                        <EraserIcon />
                                        {t("translations.editor.clear")}
                                    </MenuItem>
                                </>
                            ) : null}
                        </MenuPopup>
                    </Menu>
                </CardAction>
            </CardHeader>
            <CardPanel>
                <div className="flex flex-col gap-[18px]">
                    <div className="flex flex-col gap-1.5">
                        <Label render={<span />}>{t("translations.editor.english")}</Label>
                        <div className="whitespace-pre-line rounded-[10px] bg-muted px-3.5 py-3 text-[15px] leading-normal">{entry.source_text}</div>
                        {status === "stale" ? <SourceChange t={t} before={entry.translated_source_text} after={entry.source_text} /> : null}
                        {entry.description ? (
                            <div className="text-[12.5px] text-muted-foreground leading-normal">
                                <span className="font-medium text-foreground">{t("translations.editor.context")}</span> {entry.description}
                            </div>
                        ) : null}
                    </div>

                    {names.length > 0 ? (
                        <div className="flex flex-wrap items-center gap-1.5">
                            <span className="mr-1 text-[12.5px] text-muted-foreground">{t("translations.editor.keep")}</span>
                            {names.map((name) => {
                                const ok = !missing.includes(name);
                                return (
                                    <span key={name} className={cn("rounded-md px-2 py-0.5 font-mono text-[12px]", ok ? "bg-[color-mix(in_oklch,var(--success)_14%,transparent)] text-success-foreground" : "bg-muted text-muted-foreground")}>
                                        {`{${name}}`}
                                    </span>
                                );
                            })}
                        </div>
                    ) : null}

                    {branching.length > 0 ? (
                        <div className="flex flex-col gap-1.5">
                            <Label render={<span />}>{t("translations.editor.forms")}</Label>
                            <div className="flex flex-col gap-2">
                                {branching.map((arg) => (
                                    <ArgumentCard key={arg.name} arg={arg} written={written.get(arg.name) ?? null} started={started} locale={locale.code} language={locale.english_name} />
                                ))}
                            </div>
                        </div>
                    ) : null}

                    <div className="flex flex-col gap-1.5">
                        <Label htmlFor={fieldId}>{t("translations.editor.translation")}</Label>
                        <Textarea
                            id={fieldId}
                            rows={4}
                            lang={locale.code}
                            value={draft}
                            disabled={!canEdit}
                            aria-invalid={fieldErrors.length > 0 || (canEdit && started && missing.length > 0) || undefined}
                            placeholder={canEdit ? t("translations.editor.placeholder") : t("translations.editor.placeholderReadOnly")}
                            onChange={(e) => {
                                setDraft(e.target.value);
                                if (fieldErrors.length > 0) setFieldErrors([]);
                            }}
                            onKeyDown={(e) => {
                                if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                                    e.preventDefault();
                                    saveNext();
                                }
                            }}
                        />
                        {canEdit && started && missing.length > 0 ? <span className="text-[12.5px] text-destructive-foreground">{t("translations.editor.missing", { count: missing.length, names: missingList })}</span> : null}
                        {fieldErrors.length > 0 ? (
                            <ul className="flex flex-col gap-1">
                                {fieldErrors.map((fe) => (
                                    <li key={`${fe.field}-${fe.message}`} className="text-[12.5px] text-destructive-foreground leading-snug">
                                        {fe.message}
                                    </li>
                                ))}
                            </ul>
                        ) : null}
                    </div>

                    {/* Below lg the editor fills the screen, so the actions stick to the bottom (clear of the iOS home bar) while the fields scroll. */}
                    <div className="flex flex-wrap items-center gap-2 max-lg:sticky max-lg:bottom-0 max-lg:z-1 max-lg:-mx-6 max-lg:border-border max-lg:border-t max-lg:bg-card max-lg:px-6 max-lg:pt-3 max-lg:pb-[max(0.75rem,env(safe-area-inset-bottom))]">
                        <span className="flex-[1_1_200px] text-[12.5px] text-muted-foreground">{footer}</span>
                        {canEdit ? (
                            <>
                                <Button size="sm" variant="ghost" disabled={skipKey === undefined || skipKey === entry.key} onClick={() => onGo(skipKey)}>
                                    {t("translations.editor.skip")}
                                </Button>
                                <Button size="sm" disabled={!canSave} loading={save.isPending} onClick={saveNext}>
                                    {status === "stale" && !dirty ? t("translations.editor.markCurrent") : t("translations.editor.saveNext")}
                                    <Kbd className="pointer-coarse:hidden bg-primary-foreground/16 text-primary-foreground">{isMac ? "⌘↵" : "Ctrl↵"}</Kbd>
                                </Button>
                            </>
                        ) : null}
                    </div>
                </div>
            </CardPanel>
            <HistorySheet open={historyOpen} onOpenChange={setHistoryOpen} locale={locale} messageKey={entry.key} currentValue={entry.value} canEdit={canEdit} query={auditQuery} onReverted={invalidate} />
        </Card>
    );
}

/**
 * What the English said when this was translated, marked up against what it
 * says now. `translated_source_text` is `null` for rows saved before the
 * snapshot column existed; say so rather than render an empty comparison.
 */
function SourceChange({ t, before, after }: { t: T; before: string | null; after: string }): React.ReactElement {
    const spans = useMemo(() => (before === null ? [] : diffWords(before, after)), [before, after]);
    if (before === null) return <div className="text-[12.5px] text-muted-foreground leading-normal">{t("translations.editor.writtenAgainstUnknown")}</div>;
    return (
        <div className="flex flex-col gap-1 text-[12.5px] text-muted-foreground leading-normal">
            <div>
                {t("translations.editor.writtenAgainst")}{" "}
                <span className="whitespace-pre-line">
                    {spans.map((span, i) => (
                        // Spans have no identity of their own and the list is rebuilt whole on every source change, so the index is the key.
                        // biome-ignore lint/suspicious/noArrayIndexKey: see above
                        <span key={i} className={cn(span.op === "removed" && "text-destructive-foreground line-through", span.op === "added" && "text-success-foreground underline underline-offset-2")}>
                            {span.text}
                        </span>
                    ))}
                </span>
            </div>
            <div className="text-[11.5px]">{t("translations.editor.diffNote")}</div>
        </div>
    );
}
