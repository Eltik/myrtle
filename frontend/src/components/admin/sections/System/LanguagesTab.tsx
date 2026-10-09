import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { EllipsisIcon, PlusIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { skeletons, stackedCardAction, stackedCardHeader, stickyActionsCell } from "#/components/admin/Primitives";
import { invalidateLocaleQueries } from "#/components/admin/shell/invalidate";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardAction, CardDescription, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { Progress } from "#/components/ui/progress";
import { Switch } from "#/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { type IUpsertLocaleInput, localesQueryOptions, translationProgressQueryOptions, upsertLocaleFn } from "#/lib/api/admin";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Locale } from "#/types/generated/Locale";
import type { LocaleProgress } from "#/types/generated/LocaleProgress";
import { LocaleSheet } from "./LocaleSheet";
import { useServerLabels } from "./localeShared";
import { isGamedataServerCode, nextSortOrder, sortLocales, toggleInput, translatedPercent } from "./locales";
import type { messages } from "./System.messages";

type SystemT = TypedT<typeof messages>;

/**
 * Site languages. Anyone on System can read the table; only a super-admin can
 * flip Public, add a language or edit one (the backend enforces the same).
 */
export function LanguagesTab({ enabled, canEdit }: { enabled: boolean; canEdit: boolean }): React.ReactElement {
    const t: SystemT = useT("admin");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const localesQuery = useQuery({ ...localesQueryOptions(enabled), enabled });
    const progressQuery = useQuery({ ...translationProgressQueryOptions(enabled), enabled });
    // The target outlives `open` so the sheet keeps its title while it animates out;
    // `nonce` gives every opening a fresh form.
    const [sheet, setSheet] = useState<{ open: boolean; locale: Locale | null; nonce: number }>({ open: false, locale: null, nonce: 0 });
    const openSheet = (locale: Locale | null): void => setSheet((prev) => ({ open: true, locale, nonce: prev.nonce + 1 }));

    const locales = localesQuery.data ?? [];
    const rows = useMemo(() => sortLocales(localesQuery.data ?? []), [localesQuery.data]);
    const progress = progressQuery.data ?? [];

    const toggle = useMutation({
        mutationFn: ({ input }: { input: IUpsertLocaleInput; name: string }) => upsertLocaleFn({ data: input }),
        onSuccess: (_saved, { input, name }) => {
            invalidateLocaleQueries(queryClient);
            toastSuccess("locale-toggle", input.enabled ? t("system.languages.toast.enabled.title") : t("system.languages.toast.hidden.title"), input.enabled ? t("system.languages.toast.enabled.desc", { name }) : t("system.languages.toast.hidden.desc", { name }));
        },
        onError: (err: unknown, { name }) => toastError("locale-toggle-err", t("system.languages.toast.toggleFailed", { name }), describeError(err)),
    });

    return (
        <>
            <Card>
                {/* Below sm the action drops under the description instead of squeezing it into a narrow column. */}
                <CardHeader className={stackedCardHeader}>
                    <CardTitle>{t("system.languages.title")}</CardTitle>
                    <CardDescription>{t("system.languages.desc")}</CardDescription>
                    <CardAction className={stackedCardAction}>
                        <Button size="sm" disabled={!canEdit || !localesQuery.data} onClick={() => openSheet(null)}>
                            <PlusIcon />
                            {t("system.languages.add")}
                        </Button>
                    </CardAction>
                </CardHeader>
                {localesQuery.isPending ? (
                    <CardPanel className="flex flex-col gap-2">{skeletons(4, "h-11")}</CardPanel>
                ) : localesQuery.isError ? (
                    <CardPanel className="text-[13px] text-destructive-foreground">{t("system.languages.loadError", { message: describeError(localesQuery.error) })}</CardPanel>
                ) : rows.length === 0 ? (
                    <CardPanel className="text-[13px] text-muted-foreground">{t("system.languages.empty")}</CardPanel>
                ) : (
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>{t("system.languages.th.language")}</TableHead>
                                <TableHead className="max-md:hidden">{t("system.languages.th.code")}</TableHead>
                                <TableHead className="max-md:hidden">{t("system.languages.th.fallback")}</TableHead>
                                <TableHead className="max-md:hidden">{t("system.languages.th.server")}</TableHead>
                                <TableHead>{t("system.languages.th.translated")}</TableHead>
                                <TableHead>{t("system.languages.th.public")}</TableHead>
                                <TableHead className={stickyActionsCell}>
                                    <span className="sr-only">{t("system.languages.th.actions")}</span>
                                </TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {rows.map((l) => (
                                <LocaleRow
                                    key={l.code}
                                    locale={l}
                                    progress={progress.find((p) => p.locale === l.code)}
                                    canEdit={canEdit}
                                    pending={toggle.isPending && toggle.variables?.input.code === l.code}
                                    onToggle={(next) => toggle.mutate({ input: toggleInput(l, next), name: l.english_name })}
                                    onEdit={() => openSheet(l)}
                                />
                            ))}
                        </TableBody>
                    </Table>
                )}
            </Card>
            <LocaleSheet key={sheet.nonce} open={sheet.open && canEdit} locale={sheet.locale} locales={locales} nextSortOrder={nextSortOrder(locales)} onClose={() => setSheet((prev) => ({ ...prev, open: false }))} />
        </>
    );
}

interface ILocaleRowProps {
    locale: Locale;
    progress: LocaleProgress | undefined;
    canEdit: boolean;
    pending: boolean;
    onToggle: (next: boolean) => void;
    onEdit: () => void;
}

function LocaleRow({ locale, progress, canEdit, pending, onToggle, onEdit }: ILocaleRowProps): React.ReactElement {
    const t: SystemT = useT("admin");
    const f = useFormatters();
    const serverLabels = useServerLabels();
    const pct = translatedPercent(progress);
    const server = isGamedataServerCode(locale.gamedata_server) ? serverLabels[locale.gamedata_server] : locale.gamedata_server;

    return (
        <TableRow>
            <TableCell>
                {locale.is_source ? (
                    <span className="inline-flex items-center gap-2">
                        <span className="font-medium">{locale.english_name}</span>
                        <Badge variant="secondary" size="sm">
                            {t("system.languages.source")}
                        </Badge>
                    </span>
                ) : (
                    <div className="flex flex-col gap-[3px]">
                        <span className="font-medium">{locale.english_name}</span>
                        <span className="text-[12px] text-muted-foreground">{locale.native_name}</span>
                    </div>
                )}
            </TableCell>
            <TableCell className="max-md:hidden">
                <span className="font-mono">{locale.code}</span>
            </TableCell>
            <TableCell className="max-md:hidden">{locale.is_source ? <span className="text-muted-foreground">{t("system.languages.noFallback")}</span> : locale.fallback_locale ? t("system.languages.fallback.chain", { code: locale.fallback_locale }) : t("system.languages.fallback.english")}</TableCell>
            <TableCell className="max-md:hidden">{server}</TableCell>
            <TableCell>
                {locale.is_source ? (
                    <span className="text-muted-foreground">{t("system.languages.sourceLanguage")}</span>
                ) : pct === null ? (
                    <span className="text-muted-foreground">{t("system.languages.translated.unknown")}</span>
                ) : (
                    <div className="flex w-32 items-center gap-2.5 md:w-[200px]">
                        <div className="flex-1">
                            <Progress value={pct} aria-label={t("system.languages.th.translated")} />
                        </div>
                        <span className="w-11 text-right text-[12.5px] tabular-nums">{f.percent(pct / 100)}</span>
                    </div>
                )}
            </TableCell>
            <TableCell>
                <Switch checked={locale.is_source || locale.enabled} disabled={locale.is_source || !canEdit || pending} onCheckedChange={onToggle} aria-label={t("system.languages.public.label", { name: locale.english_name })} />
            </TableCell>
            <TableCell className={cn("w-px text-right", stickyActionsCell)}>
                <Menu>
                    <MenuTrigger disabled={!canEdit} render={<Button variant="ghost" size="icon-xs" />} aria-label={t("system.languages.menu.label", { name: locale.english_name })}>
                        <EllipsisIcon />
                    </MenuTrigger>
                    <MenuPopup align="end">
                        <MenuItem onClick={onEdit}>{t("system.languages.menu.edit")}</MenuItem>
                    </MenuPopup>
                </Menu>
            </TableCell>
        </TableRow>
    );
}
