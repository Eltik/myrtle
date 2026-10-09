import { type UseQueryResult, useMutation } from "@tanstack/react-query";
import { RotateCcwIcon } from "lucide-react";
import { useState } from "react";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Sheet, SheetDescription, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Skeleton } from "#/components/ui/skeleton";
import { type ITranslationFieldError, type IUpdateTranslationInput, updateTranslationFn } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { Locale } from "#/types/generated/Locale";
import type { UiMessageAuditEntry } from "#/types/generated/UiMessageAuditEntry";
import type { messages } from "./Translations.messages";

export interface IHistorySheetProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    locale: Locale;
    messageKey: string;
    /** The stored translation now, so the row that matches it offers no revert. */
    currentValue: string | null;
    canEdit: boolean;
    /** The editor's per-key audit query, shared so the footer count and this list agree. */
    query: UseQueryResult<UiMessageAuditEntry[]>;
    onReverted: () => void;
}

/**
 * Every earlier version of one translation. Each row keeps the value it
 * replaced, so a revert is an ordinary write of `old_value`.
 */
export function HistorySheet({ open, onOpenChange, locale, messageKey, currentValue, canEdit, query, onReverted }: IHistorySheetProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("admin");
    const fmt = useFormatters();
    const describeError = useErrorMessage();
    // A 422 on a revert belongs on the row it came from, not in a toast.
    const [rejected, setRejected] = useState<{ id: number; details: ITranslationFieldError[] } | null>(null);

    const revert = useMutation({
        mutationFn: (input: IUpdateTranslationInput & { id: number }) => updateTranslationFn({ data: { locale: input.locale, key: input.key, value: input.value } }),
        onSuccess: (result, input) => {
            if (!result.ok) {
                setRejected({ id: input.id, details: result.details });
                return;
            }
            setRejected(null);
            onReverted();
            toastSuccess("translations-revert", t("translations.toast.reverted"), t("translations.toast.reverted.desc", { key: messageKey }));
        },
        onError: (err: unknown) => toastError("translations-revert-err", t("translations.toast.revertFailed"), describeError(err)),
    });

    const revisions = query.data ?? [];

    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetPopup side="right" variant="inset">
                <SheetHeader>
                    <SheetTitle>{t("translations.history.title")}</SheetTitle>
                    <SheetDescription className="[overflow-wrap:anywhere]">{t("translations.history.desc", { key: messageKey, language: locale.english_name })}</SheetDescription>
                </SheetHeader>
                <SheetPanel>
                    {query.isPending ? (
                        <div className="flex flex-col gap-3">
                            <Skeleton className="h-20" />
                            <Skeleton className="h-20" />
                        </div>
                    ) : query.isError ? (
                        <p className="py-8 text-center text-[13.5px] text-muted-foreground">{t("translations.history.error")}</p>
                    ) : revisions.length === 0 ? (
                        <p className="py-8 text-center text-[13.5px] text-muted-foreground">{t("translations.history.empty")}</p>
                    ) : (
                        <ol className="flex flex-col gap-4">
                            {revisions.map((rev) => (
                                <li key={rev.id} className="flex flex-col gap-1.5">
                                    <div className="text-[12.5px] text-muted-foreground">
                                        <time dateTime={rev.changed_at} title={fmt.date(rev.changed_at, { dateStyle: "medium", timeStyle: "short" })}>
                                            {fmt.relativeShort(rev.changed_at)}
                                        </time>{" "}
                                        · {t("translations.history.by", { actor: rev.actor.nickname ?? t("translations.history.deletedAccount") })}
                                    </div>
                                    <div className="flex flex-col gap-1 rounded-[10px] bg-muted px-3 py-2 text-[13px] leading-normal" lang={locale.code}>
                                        {rev.old_value !== null ? <div className="text-muted-foreground line-through">{rev.old_value}</div> : <div className="text-muted-foreground italic">{t("translations.history.wasEmpty")}</div>}
                                        {rev.new_value !== null ? <div>{rev.new_value}</div> : <div className="text-muted-foreground italic">{t("translations.history.cleared")}</div>}
                                    </div>
                                    {canEdit && rev.old_value !== null && rev.old_value !== currentValue ? (
                                        <div>
                                            <Button size="xs" variant="outline" disabled={revert.isPending} loading={revert.isPending && revert.variables?.id === rev.id} onClick={() => revert.mutate({ id: rev.id, locale: locale.code, key: messageKey, value: rev.old_value ?? "" })}>
                                                <RotateCcwIcon />
                                                {t("translations.history.revert")}
                                            </Button>
                                        </div>
                                    ) : null}
                                    {rejected?.id === rev.id ? (
                                        <ul className="flex flex-col gap-1">
                                            {rejected.details.map((d) => (
                                                <li key={`${d.field}-${d.message}`} className="text-[12.5px] text-destructive-foreground leading-snug">
                                                    {d.message}
                                                </li>
                                            ))}
                                        </ul>
                                    ) : null}
                                </li>
                            ))}
                        </ol>
                    )}
                </SheetPanel>
            </SheetPopup>
        </Sheet>
    );
}
