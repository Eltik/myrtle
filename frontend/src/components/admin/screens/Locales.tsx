import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckIcon, GlobeIcon, PencilIcon, PlusIcon, TriangleAlertIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { Switch } from "#/components/ui/switch";
import { toastManager } from "#/components/ui/toast";
import { type IUpsertLocaleInput, isSuperAdmin, localesQueryOptions, translationProgressQueryOptions, upsertLocaleFn } from "#/lib/api/admin";
import { GAMEDATA_SERVERS, type GamedataServer } from "#/lib/api/gamedata";
import { LOCALE_SEGMENT, type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Locale } from "#/types/generated/Locale";
import type { LocaleProgress } from "#/types/generated/LocaleProgress";
import { HCode } from "../AdminShell";
import { MonoSection } from "../Primitives";
import type { messages } from "./Locales.messages";

type LocalesT = TypedT<typeof messages>;
type LocalesRichT = TypedRichT<typeof messages>;

/**
 * What each game region means to a reader who is picking one. The value set is
 * the backend's `Server` enum; these labels are only for the picker.
 */
function serverLabels(t: LocalesT): Record<GamedataServer, string> {
    return {
        en: t("locales.server.en"),
        jp: t("locales.server.jp"),
        kr: t("locales.server.kr"),
        cn: t("locales.server.cn"),
        tw: t("locales.server.tw"),
        bili: t("locales.server.bili"),
    };
}

const NO_FALLBACK = "__none__";

/**
 * The shape `LOCALE_SEGMENT` accepts, written out for a reader rather than for
 * the regex engine - it is quoted into the validation message as a value, so
 * ICU never sees its braces.
 */
const CODE_PATTERN = "^[a-z]{2}(-[A-Za-z]{2,4})?$";

interface IFormState {
    code: string;
    englishName: string;
    nativeName: string;
    fallbackLocale: string;
    gamedataServer: GamedataServer;
    enabled: boolean;
    sortOrder: string;
}

type FormField = keyof IFormState;

type Errors = Partial<Record<FormField, string>>;

function emptyForm(nextSortOrder: number): IFormState {
    return {
        code: "",
        englishName: "",
        nativeName: "",
        fallbackLocale: NO_FALLBACK,
        gamedataServer: "en",
        enabled: false,
        sortOrder: String(nextSortOrder),
    };
}

function formFromLocale(locale: Locale): IFormState {
    return {
        code: locale.code,
        englishName: locale.english_name,
        nativeName: locale.native_name,
        fallbackLocale: locale.fallback_locale ?? NO_FALLBACK,
        gamedataServer: (GAMEDATA_SERVERS as readonly string[]).includes(locale.gamedata_server) ? (locale.gamedata_server as GamedataServer) : "en",
        enabled: locale.enabled,
        sortOrder: String(locale.sort_order),
    };
}

/**
 * Walk the fallback chain from `start` and report whether it comes back to
 * `target`. The source locale has no fallback, so every well-formed chain
 * terminates - this
 * exists to stop an ill-formed one being written in the first place, because a
 * cycle in `locales.fallback_locale` is a resolver that never terminates.
 */
function fallbackCycles(target: string, start: string, locales: Locale[]): boolean {
    const byCode = new Map(locales.map((l) => [l.code, l]));
    const seen = new Set<string>([target]);
    let cursor: string | null = start;
    while (cursor) {
        if (seen.has(cursor)) return true;
        seen.add(cursor);
        cursor = byCode.get(cursor)?.fallback_locale ?? null;
    }
    return false;
}

/**
 * Validation that encodes what the rest of the system actually requires, not
 * just what the column types allow. Each message names the consequence, because
 * "invalid" on a locale code is indistinguishable from a typo until you learn
 * that the router never routes it.
 */
function validate(t: LocalesT, form: IFormState, locales: Locale[], isNew: boolean): Errors {
    const errors: Errors = {};
    const code = form.code.trim();
    // Which locale the source text is written in is a row in `locales`, not a
    // constant - `DEFAULT_LOCALE` is only the floor for the bundled catalog.
    const sourceCode = locales.find((l) => l.is_source)?.code ?? null;
    const isSource = sourceCode !== null && code === sourceCode;

    if (code.length === 0) {
        errors.code = t("locales.err.code.required");
    } else if (!LOCALE_SEGMENT.test(code)) {
        errors.code = t("locales.err.code.shape", { pattern: CODE_PATTERN });
    } else if (isNew && locales.some((l) => l.code === code)) {
        errors.code = t("locales.err.code.exists", { code });
    }

    if (form.englishName.trim().length === 0) errors.englishName = t("locales.err.englishName");
    if (form.nativeName.trim().length === 0) errors.nativeName = t("locales.err.nativeName");

    if (form.fallbackLocale !== NO_FALLBACK) {
        if (isSource && sourceCode !== null) {
            errors.fallbackLocale = t("locales.err.fallback.source", { locale: sourceCode });
        } else if (form.fallbackLocale === code) {
            errors.fallbackLocale = t("locales.err.fallback.self");
        } else if (fallbackCycles(code, form.fallbackLocale, locales)) {
            errors.fallbackLocale = t("locales.err.fallback.cycle");
        }
    }

    if (!(GAMEDATA_SERVERS as readonly string[]).includes(form.gamedataServer)) {
        errors.gamedataServer = t("locales.err.gamedataServer", { servers: GAMEDATA_SERVERS.join(", ") });
    }

    if (isSource && sourceCode !== null && !form.enabled) {
        errors.enabled = t("locales.err.enabled", { locale: sourceCode });
    }

    const sortOrder = Number(form.sortOrder);
    if (form.sortOrder.trim().length === 0 || !Number.isInteger(sortOrder) || sortOrder < 0) {
        errors.sortOrder = t("locales.err.sortOrder");
    }

    return errors;
}

// ---------------------------------------------------------------- section

export function LocalesSection({ role, authed }: { role: string | null; authed: boolean }): React.ReactElement {
    const t: LocalesT = useT("admin");
    const rt: LocalesRichT = useRichT("admin");
    const localesQuery = useQuery(localesQueryOptions(authed));
    const progressQuery = useQuery(translationProgressQueryOptions(authed));

    const [editing, setEditing] = useState<{ locale: Locale | null } | null>(null);

    const locales = localesQuery.data ?? [];
    const progress = progressQuery.data ?? [];
    const canWrite = isSuperAdmin(role);

    const rows = useMemo(() => [...locales].sort((a, b) => a.sort_order - b.sort_order || a.code.localeCompare(b.code)), [locales]);
    const nextSortOrder = useMemo(() => (locales.length === 0 ? 0 : Math.max(...locales.map((l) => l.sort_order)) + 10), [locales]);

    return (
        <>
            <Card>
                <CardHeader>
                    <CardTitle className="flex items-center gap-2 text-sm">
                        <GlobeIcon className="size-3.5 opacity-70" strokeWidth={1.9} />
                        {t("locales.title")}
                    </CardTitle>
                    <CardDescription className="text-xs">
                        {rt("locales.desc", { table: <HCode>locales</HCode>, column: <HCode>gamedata_server</HCode> })}
                        {canWrite ? null : t("locales.desc.readOnly")}
                    </CardDescription>
                    {canWrite ? (
                        <CardAction>
                            <Button size="sm" onClick={() => setEditing({ locale: null })}>
                                <PlusIcon />
                                {t("locales.add")}
                            </Button>
                        </CardAction>
                    ) : null}
                </CardHeader>

                {localesQuery.isPending ? (
                    <CardContent className="border-border border-t">
                        <Skeleton className="h-28 w-full" />
                    </CardContent>
                ) : rows.length === 0 ? (
                    <CardContent className="border-border border-t py-10 text-center text-[13px] text-muted-foreground">{t("locales.empty")}</CardContent>
                ) : (
                    <div className="overflow-x-auto border-border border-t">
                        <table className="w-full min-w-200 border-collapse text-[13px]">
                            <thead>
                                <tr>
                                    {[t("locales.th.code"), t("locales.th.name"), t("locales.th.fallback"), t("locales.th.gameData"), t("locales.th.sort"), t("locales.th.translated"), t("locales.th.state"), ""].map((h) => (
                                        <th key={h} className="bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_1.5%)] px-3.5 py-2.5 text-left font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                            {h}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {rows.map((l) => (
                                    <LocaleRow key={l.code} locale={l} progress={progress.find((p) => p.locale === l.code)} canWrite={canWrite} onEdit={() => setEditing({ locale: l })} />
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </Card>

            {editing ? <LocaleDialog locale={editing.locale} locales={locales} nextSortOrder={nextSortOrder} onClose={() => setEditing(null)} /> : null}
        </>
    );
}

function LocaleRow({ locale, progress, canWrite, onEdit }: { locale: Locale; progress: LocaleProgress | undefined; canWrite: boolean; onEdit: () => void }): React.ReactElement {
    const t: LocalesT = useT("admin");
    const pct = progress && progress.total > 0 ? Math.round((progress.translated / progress.total) * 100) : null;
    return (
        <tr className="border-border border-b last:border-0">
            <td className="px-3.5 py-2.5 font-mono text-[12.5px]">
                {locale.code}
                {locale.is_source ? (
                    <Badge variant="outline" size="sm" className="ml-1.5 font-mono">
                        {t("locales.source")}
                    </Badge>
                ) : null}
            </td>
            <td className="px-3.5 py-2.5">
                <span className="font-medium">{locale.native_name}</span>
                <span className="ml-1.5 text-[12px] text-muted-foreground">{locale.english_name}</span>
            </td>
            <td className="px-3.5 py-2.5 font-mono text-[12px] text-muted-foreground">{locale.fallback_locale ?? "-"}</td>
            <td className="px-3.5 py-2.5">
                <Badge variant="outline">{locale.gamedata_server}</Badge>
            </td>
            <td className="px-3.5 py-2.5 font-mono text-[12px] text-muted-foreground tabular-nums">{locale.sort_order}</td>
            <td className="px-3.5 py-2.5">
                {pct === null ? (
                    <span className="text-[12px] text-muted-foreground">-</span>
                ) : (
                    <span className="inline-flex items-center gap-2">
                        <span className="inline-block h-1.5 w-14 overflow-hidden rounded-[3px] bg-muted">
                            <span className="block h-full bg-success-foreground/70" style={{ width: `${pct}%` }} />
                        </span>
                        <span className="font-mono text-[11.5px] text-muted-foreground tabular-nums">{pct}%</span>
                    </span>
                )}
            </td>
            <td className="px-3.5 py-2.5">{locale.enabled ? <Badge variant="success">{t("locales.badge.enabled")}</Badge> : <Badge variant="outline">{t("locales.badge.hidden")}</Badge>}</td>
            <td className="px-3.5 py-2.5">
                {canWrite ? (
                    <Button variant="outline" size="xs" onClick={onEdit}>
                        <PencilIcon />
                        {t("locales.edit")}
                    </Button>
                ) : null}
            </td>
        </tr>
    );
}

// ---------------------------------------------------------------- editor

function LocaleDialog({ locale, locales, nextSortOrder, onClose }: { locale: Locale | null; locales: Locale[]; nextSortOrder: number; onClose: () => void }): React.ReactElement {
    const t: LocalesT = useT("admin");
    const describeError = useErrorMessage();
    const rt: LocalesRichT = useRichT("admin");
    const queryClient = useQueryClient();
    const serverLabel = serverLabels(t);
    const isNew = locale === null;
    const [form, setForm] = useState<IFormState>(() => (locale ? formFromLocale(locale) : emptyForm(nextSortOrder)));
    const [touched, setTouched] = useState(false);
    // Enabling is the consequential half of this form, so it gets its own
    // confirm step rather than riding along with Save.
    const [confirmEnable, setConfirmEnable] = useState(false);

    const code = form.code.trim();
    // The backend force-keeps the source locale enabled with no fallback, so
    // both controls are held to what it will write back anyway.
    const isSource = locales.some((l) => l.is_source && l.code === code);
    const errors = validate(t, form, locales, isNew);
    const invalid = Object.keys(errors).length > 0;

    const set = <K extends FormField>(key: K, value: IFormState[K]) => {
        setTouched(true);
        setForm((prev) => ({ ...prev, [key]: value }));
    };

    const fallbackOptions = useMemo(() => locales.filter((l) => l.code !== code), [locales, code]);

    const save = useMutation({
        mutationFn: (input: IUpsertLocaleInput) => upsertLocaleFn({ data: input }),
        onSuccess: (saved) => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "locales"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "i18n", "progress"] });
            toastManager.add({ id: `locale-save-${Date.now()}`, title: isNew ? t("locales.toast.created") : t("locales.toast.saved"), description: t("locales.toast.saved.desc", { code: saved.code, name: saved.native_name }), type: "success" });
            onClose();
        },
        onError: (err: unknown) => toastManager.add({ id: `locale-save-err-${Date.now()}`, title: t("locales.toast.failed"), description: describeError(err), type: "error" }),
    });

    const submit = () => {
        setTouched(true);
        if (invalid) return;
        save.mutate({
            code,
            englishName: form.englishName.trim(),
            nativeName: form.nativeName.trim(),
            fallbackLocale: form.fallbackLocale === NO_FALLBACK ? null : form.fallbackLocale,
            gamedataServer: form.gamedataServer,
            enabled: form.enabled,
            sortOrder: Number(form.sortOrder),
        });
    };

    const err = (field: FormField) => (touched ? errors[field] : undefined);

    return (
        <>
            <button type="button" className="fixed inset-0 z-55 cursor-default bg-black/36 backdrop-blur-[2px]" onClick={onClose} aria-label={t("locales.close")} />
            <div className="pointer-events-none fixed inset-0 z-60 grid place-items-center p-3 max-sm:items-end max-sm:p-0">
                <div className="pointer-events-auto flex max-h-[92dvh] w-140 max-w-[92vw] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-[0_30px_60px_oklch(0_0_0/0.35),0_8px_18px_oklch(0_0_0/0.2)] max-sm:w-full max-sm:max-w-none max-sm:rounded-b-none max-sm:border-x-0 max-sm:border-b-0">
                    <div className="shrink-0 border-border border-b px-5 pt-4 pb-3.5">
                        <span className="font-bold text-[10px] text-primary uppercase tracking-[0.22em]">{t("locales.dialog.kicker")}</span>
                        <div className="mt-1.5 font-semibold text-[18px] leading-tight tracking-[-0.01em]">{isNew ? t("locales.dialog.add") : t("locales.dialog.edit", { code: locale?.code })}</div>
                    </div>

                    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-5 [-webkit-overflow-scrolling:touch]">
                        {/* The code examples in the placeholder are locale tags, not copy. */}
                        <FormRow label={t("locales.form.code")} error={err("code")} hint={rt("locales.form.code.hint", { pattern: <HCode>^[a-z]&#123;2&#125;(-[A-Za-z]&#123;2,4&#125;)?$</HCode> })}>
                            <Input aria-invalid={err("code") ? true : undefined} disabled={!isNew} size="sm" placeholder="ja, pt-BR, zh-Hans" value={form.code} onChange={(e) => set("code", e.target.value)} />
                            {isNew ? null : <p className="mt-1 text-[11px] text-muted-foreground">{t("locales.form.code.locked")}</p>}
                        </FormRow>

                        <FormRow label={t("locales.form.englishName")} error={err("englishName")} hint={t("locales.form.englishName.hint")}>
                            <Input aria-invalid={err("englishName") ? true : undefined} size="sm" placeholder={t("locales.form.englishName.placeholder")} value={form.englishName} onChange={(e) => set("englishName", e.target.value)} />
                        </FormRow>

                        <FormRow label={t("locales.form.nativeName")} error={err("nativeName")} hint={t("locales.form.nativeName.hint")}>
                            <Input aria-invalid={err("nativeName") ? true : undefined} size="sm" placeholder={t("locales.form.nativeName.placeholder")} value={form.nativeName} onChange={(e) => set("nativeName", e.target.value)} />
                        </FormRow>

                        <FormRow label={t("locales.form.fallback")} error={err("fallbackLocale")} hint={isSource ? rt("locales.form.fallback.hintSource", { locale: <HCode>{code}</HCode> }) : t("locales.form.fallback.hint")}>
                            <Select disabled={isSource} onValueChange={(v) => v && set("fallbackLocale", String(v))} value={isSource ? NO_FALLBACK : form.fallbackLocale}>
                                <SelectTrigger className="w-full" size="sm">
                                    <SelectValue placeholder={t("locales.form.fallback.none")}>{(value) => (value === NO_FALLBACK ? t("locales.form.fallback.none") : String(value))}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value={NO_FALLBACK}>{t("locales.form.fallback.none")}</SelectItem>
                                    {fallbackOptions.map((l) => (
                                        <SelectItem key={l.code} value={l.code}>
                                            <span className="font-mono text-[12px]">{l.code}</span>
                                            <span className="ml-1.5 text-muted-foreground">{l.native_name}</span>
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </FormRow>

                        <FormRow label={t("locales.form.gamedataServer")} error={err("gamedataServer")} hint={t("locales.form.gamedataServer.hint")}>
                            <Select onValueChange={(v) => v && set("gamedataServer", String(v) as GamedataServer)} value={form.gamedataServer}>
                                <SelectTrigger className="w-full" size="sm">
                                    <SelectValue placeholder={t("locales.form.gamedataServer.placeholder")}>{(value) => serverLabel[value as GamedataServer] ?? String(value)}</SelectValue>
                                </SelectTrigger>
                                <SelectContent>
                                    {GAMEDATA_SERVERS.map((s) => (
                                        <SelectItem key={s} value={s}>
                                            <span className="font-mono text-[12px]">{s}</span>
                                            <span className="ml-1.5 text-muted-foreground">{serverLabel[s]}</span>
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </FormRow>

                        <FormRow label={t("locales.form.sortOrder")} error={err("sortOrder")} hint={t("locales.form.sortOrder.hint")}>
                            <Input aria-invalid={err("sortOrder") ? true : undefined} size="sm" inputMode="numeric" value={form.sortOrder} onChange={(e) => set("sortOrder", e.target.value)} />
                        </FormRow>

                        <div className="flex flex-col gap-2 rounded-xl border border-border bg-[color-mix(in_srgb,var(--card),oklch(0_0_0)_2%)] p-3">
                            <div className="flex items-start justify-between gap-3">
                                <div className="flex min-w-0 flex-col gap-0.5">
                                    <MonoSection>{t("locales.form.enabled")}</MonoSection>
                                    <p className="text-[12px] text-muted-foreground leading-normal">
                                        {rt("locales.enable.desc", {
                                            code: <span className="font-mono">{code || t("locales.thisLocale")}</span>,
                                            path: <HCode>/{code || "xx"}/…</HCode>,
                                            server: <span className="font-mono">{form.gamedataServer}</span>,
                                        })}
                                    </p>
                                    {isSource ? <p className="text-[11px] text-muted-foreground leading-normal">{rt("locales.form.enabled.hintSource", { locale: <HCode>{code}</HCode> })}</p> : null}
                                </div>
                                <Switch
                                    checked={form.enabled}
                                    disabled={isSource}
                                    onCheckedChange={(next) => {
                                        if (next && !form.enabled) {
                                            setConfirmEnable(true);
                                            return;
                                        }
                                        set("enabled", next);
                                    }}
                                />
                            </div>
                            {err("enabled") ? <FieldError>{err("enabled")}</FieldError> : null}
                            <p className="flex items-start gap-1.5 text-[11px] text-muted-foreground leading-normal">
                                <TriangleAlertIcon className="mt-px size-3.5 shrink-0 opacity-70" strokeWidth={1.9} />
                                <span>
                                    {rt("locales.warn", {
                                        server: <span className="font-mono">{form.gamedataServer}</span>,
                                        env: <HCode>SERVERS</HCode>,
                                        path: <HCode>/{form.gamedataServer}/…</HCode>,
                                    })}
                                </span>
                            </p>
                        </div>

                        {confirmEnable ? (
                            <div className="flex flex-col gap-2.5 rounded-xl border border-warning/32 bg-warning/8 p-3">
                                <span className="font-medium text-[12.5px]">{t("locales.confirm.title", { code: code || t("locales.thisLocale") })}</span>
                                <p className="text-[12px] text-muted-foreground leading-normal">
                                    {rt("locales.confirm.body", {
                                        path: <HCode>/{code || "xx"}</HCode>,
                                        fallback: form.fallbackLocale === NO_FALLBACK ? t("locales.confirm.fallbackEnglish") : t("locales.confirm.fallbackChain", { locale: form.fallbackLocale }),
                                    })}
                                </p>
                                <div className="flex flex-wrap gap-2">
                                    <Button
                                        size="xs"
                                        onClick={() => {
                                            set("enabled", true);
                                            setConfirmEnable(false);
                                        }}
                                    >
                                        <CheckIcon />
                                        {t("locales.confirm.yes")}
                                    </Button>
                                    <Button variant="outline" size="xs" onClick={() => setConfirmEnable(false)}>
                                        {t("locales.confirm.no")}
                                    </Button>
                                </div>
                            </div>
                        ) : null}
                    </div>

                    <div className="flex shrink-0 items-center justify-end gap-2 border-border border-t p-3.5">
                        {touched && invalid ? <span className="mr-auto text-[12px] text-destructive-foreground">{t("locales.fixFields")}</span> : null}
                        <Button variant="outline" size="sm" onClick={onClose}>
                            {t("locales.cancel")}
                        </Button>
                        <Button size="sm" disabled={save.isPending || (touched && invalid)} loading={save.isPending} onClick={submit}>
                            <CheckIcon />
                            {isNew ? t("locales.create") : t("locales.save")}
                        </Button>
                    </div>
                </div>
            </div>
        </>
    );
}

function FormRow({ label, hint, error, children }: { label: string; hint?: React.ReactNode; error?: string; children: React.ReactNode }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5">
            <span className={cn("font-medium text-[12px]", error && "text-destructive-foreground")}>{label}</span>
            {children}
            {error ? <FieldError>{error}</FieldError> : hint ? <p className="text-[11px] text-muted-foreground leading-normal">{hint}</p> : null}
        </div>
    );
}

function FieldError({ children }: { children: React.ReactNode }): React.ReactElement {
    return <p className="rounded-lg border border-destructive/32 bg-destructive/8 px-2 py-1.5 text-[11.5px] text-destructive-foreground leading-snug">{children}</p>;
}
