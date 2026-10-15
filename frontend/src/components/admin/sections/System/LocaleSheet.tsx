import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useId, useMemo, useState } from "react";
import { invalidateLocaleQueries } from "#/components/admin/shell/invalidate";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Sheet, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { type IUpsertLocaleInput, upsertLocaleFn } from "#/lib/api/admin";
import { GAMEDATA_SERVERS, type GamedataServer } from "#/lib/api/gamedata";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Locale } from "#/types/generated/Locale";
import { useServerLabels } from "./localeShared";
import { emptyLocaleForm, type ILocaleForm, type LocaleFormError, type LocaleFormField, localeForm, NO_FALLBACK, upsertInput, validateLocaleForm } from "./locales";
import type { messages } from "./System.messages";

type SystemT = TypedT<typeof messages>;

function errorText(t: SystemT, error: LocaleFormError, code: string): string {
    switch (error) {
        case "codeRequired":
            return t("system.languages.err.codeRequired");
        case "codeShape":
            return t("system.languages.err.codeShape");
        case "codeExists":
            return t("system.languages.err.codeExists", { code });
        case "englishNameRequired":
            return t("system.languages.err.englishNameRequired");
        case "nativeNameRequired":
            return t("system.languages.err.nativeNameRequired");
        case "fallbackSource":
            return t("system.languages.err.fallbackSource");
        case "fallbackSelf":
            return t("system.languages.err.fallbackSelf");
        case "fallbackCycle":
            return t("system.languages.err.fallbackCycle");
        case "gamedataServer":
            return t("system.languages.err.gamedataServer");
        case "sortOrder":
            return t("system.languages.err.sortOrder");
    }
}

export interface ILocaleSheetProps {
    open: boolean;
    /** Null to add a language. */
    locale: Locale | null;
    locales: readonly Locale[];
    nextSortOrder: number;
    onClose: () => void;
}

/**
 * Add or edit a language: names, fallback (refusing a cycle), game data
 * server and sort order. Public stays on the table's switch: a new language
 * starts hidden and an edit keeps whatever the switch says.
 */
export function LocaleSheet({ open, locale, locales, nextSortOrder, onClose }: ILocaleSheetProps): React.ReactElement {
    const t: SystemT = useT("admin");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const serverLabels = useServerLabels();
    const baseId = useId();
    const isNew = locale === null;
    const [form, setForm] = useState<ILocaleForm>(() => (locale ? localeForm(locale) : emptyLocaleForm(nextSortOrder)));
    const [touched, setTouched] = useState(false);

    const code = form.code.trim();
    const isSource = locales.some((l) => l.is_source && l.code === code);
    const errors = validateLocaleForm(form, locales, isNew);
    const invalid = Object.keys(errors).length > 0;
    const fallbackOptions = useMemo(() => locales.filter((l) => l.code !== code), [locales, code]);
    const err = (field: LocaleFormField): string | undefined => {
        const e = touched ? errors[field] : undefined;
        return e ? errorText(t, e, code) : undefined;
    };
    const set = <K extends LocaleFormField>(key: K, value: ILocaleForm[K]): void => setForm((prev) => ({ ...prev, [key]: value }));

    const save = useMutation({
        mutationFn: (input: IUpsertLocaleInput) => upsertLocaleFn({ data: input }),
        onSuccess: (saved) => {
            invalidateLocaleQueries(queryClient);
            toastSuccess("locale-save", isNew ? t("system.languages.toast.created") : t("system.languages.toast.saved"), t("system.languages.toast.savedDesc", { name: saved.english_name, code: saved.code }));
            onClose();
        },
        onError: (e: unknown) => toastError("locale-save-err", t("system.languages.toast.saveFailed"), describeError(e)),
    });

    const submit = (event: React.FormEvent): void => {
        event.preventDefault();
        setTouched(true);
        if (invalid) return;
        save.mutate(upsertInput(form, locale?.enabled ?? false));
    };

    const nameOf = (c: string): string => locales.find((l) => l.code === c)?.english_name ?? c;

    return (
        <Sheet open={open} onOpenChange={(next) => (next ? undefined : onClose())}>
            <SheetPopup side="right" variant="inset">
                <form onSubmit={submit} className="flex min-h-0 flex-1 flex-col" noValidate>
                    <SheetHeader>
                        <SheetTitle>{isNew ? t("system.languages.sheet.addTitle") : t("system.languages.sheet.editTitle", { name: locale.english_name })}</SheetTitle>
                        <SheetDescription>{isNew ? t("system.languages.sheet.addDesc") : t("system.languages.sheet.editDesc")}</SheetDescription>
                    </SheetHeader>
                    <SheetPanel className="flex flex-col gap-4">
                        <FormRow id={`${baseId}-code`} label={t("system.languages.form.code")} error={err("code")} hint={isNew ? t("system.languages.form.codeHint") : t("system.languages.form.codeLocked")}>
                            {/* Placeholder examples are language tags, not copy. */}
                            <Input id={`${baseId}-code`} className="font-mono" aria-invalid={err("code") ? true : undefined} disabled={!isNew} placeholder="ja, pt-BR" value={form.code} onChange={(e) => set("code", e.target.value)} />
                        </FormRow>
                        <FormRow id={`${baseId}-en`} label={t("system.languages.form.englishName")} error={err("englishName")} hint={t("system.languages.form.englishNameHint")}>
                            <Input id={`${baseId}-en`} aria-invalid={err("englishName") ? true : undefined} placeholder={t("system.languages.form.englishNamePlaceholder")} value={form.englishName} onChange={(e) => set("englishName", e.target.value)} />
                        </FormRow>
                        <FormRow id={`${baseId}-native`} label={t("system.languages.form.nativeName")} error={err("nativeName")} hint={t("system.languages.form.nativeNameHint")}>
                            <Input id={`${baseId}-native`} aria-invalid={err("nativeName") ? true : undefined} placeholder={t("system.languages.form.nativeNamePlaceholder")} value={form.nativeName} onChange={(e) => set("nativeName", e.target.value)} />
                        </FormRow>
                        <FormRow id={`${baseId}-fallback`} label={t("system.languages.form.fallback")} error={err("fallbackLocale")} hint={isSource ? t("system.languages.form.fallbackHintSource") : t("system.languages.form.fallbackHint")}>
                            <Select disabled={isSource} value={isSource ? NO_FALLBACK : form.fallbackLocale} onValueChange={(v) => v && set("fallbackLocale", String(v))}>
                                <SelectTrigger id={`${baseId}-fallback`} aria-invalid={err("fallbackLocale") ? true : undefined}>
                                    <SelectValue>{(value) => (value === NO_FALLBACK ? t("system.languages.form.fallbackNone") : nameOf(String(value)))}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value={NO_FALLBACK}>{t("system.languages.form.fallbackNone")}</SelectItem>
                                    {fallbackOptions.map((l) => (
                                        <SelectItem key={l.code} value={l.code}>
                                            {l.english_name}
                                            <span className="ml-1.5 font-mono text-[12px] text-muted-foreground">{l.code}</span>
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </FormRow>
                        <FormRow id={`${baseId}-server`} label={t("system.languages.form.server")} error={err("gamedataServer")} hint={t("system.languages.form.serverHint")}>
                            <Select value={form.gamedataServer} onValueChange={(v) => v && set("gamedataServer", String(v) as GamedataServer)}>
                                <SelectTrigger id={`${baseId}-server`}>
                                    <SelectValue>{(value) => serverLabels[value as GamedataServer] ?? String(value)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {GAMEDATA_SERVERS.map((s) => (
                                        <SelectItem key={s} value={s}>
                                            {serverLabels[s]}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </FormRow>
                        <FormRow id={`${baseId}-sort`} label={t("system.languages.form.sortOrder")} error={err("sortOrder")} hint={t("system.languages.form.sortOrderHint")}>
                            <Input id={`${baseId}-sort`} className="tabular-nums" inputMode="numeric" aria-invalid={err("sortOrder") ? true : undefined} value={form.sortOrder} onChange={(e) => set("sortOrder", e.target.value)} />
                        </FormRow>
                    </SheetPanel>
                    <SheetFooter>
                        {touched && invalid ? <span className="self-center text-[12.5px] text-destructive-foreground sm:mr-auto">{t("system.languages.form.fixFields")}</span> : null}
                        <Button type="button" variant="ghost" onClick={onClose}>
                            {t("system.languages.form.cancel")}
                        </Button>
                        <Button type="submit" loading={save.isPending} disabled={save.isPending || (touched && invalid)}>
                            {isNew ? t("system.languages.add") : t("system.languages.form.save")}
                        </Button>
                    </SheetFooter>
                </form>
            </SheetPopup>
        </Sheet>
    );
}

function FormRow({ id, label, hint, error, children }: { id: string; label: string; hint: string; error?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5">
            <Label htmlFor={id} className={cn(error && "text-destructive-foreground")}>
                {label}
            </Label>
            {children}
            <p className={cn("text-[12px] leading-normal", error ? "text-destructive-foreground" : "text-muted-foreground")}>{error ?? hint}</p>
        </div>
    );
}
