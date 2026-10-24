import { useQuery } from "@tanstack/react-query";
import { Pencil, Plus, Trash2 } from "lucide-react";
import * as React from "react";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Sheet, SheetDescription, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "#/components/ui/table";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { releaseBannersQueryOptions, releaseEventsQueryOptions } from "#/lib/api/release";
import { useFormatters, useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { ReleaseOverride } from "#/types/generated/ReleaseOverride";
import type { Resolution } from "#/types/generated/Resolution";
import { useAutoTranslate } from "../../autoTranslate";
import { formatDate, useReleaseTimeZone } from "../../helpers";
import { isOverrideKind } from "../../overrides";
import { useReleaseOverrides } from "../../useReleaseOverrides";
import { ReleaseEmpty, ReleaseError, ReleaseLoading, resolveName } from "../shared";
import { type IOverrideTarget, OverrideSheet, RemoveOverrideDialog, useOverrideFallbackName, useOverrideKindLabel } from "./OverrideSheet";
import type { messages as sheetMessages } from "./OverrideSheet.messages";
import type { messages } from "./OverridesListSheet.messages";

/** The list renders its own chrome plus the toast wording the override sheet owns. */
type ListT = TypedT<typeof messages & typeof sheetMessages>;

/** What the list knows about an override's row from the planner's own queries. */
interface IRowInfo {
    name: string;
    resolution: Resolution;
}

/**
 * The header entry for super admins: a button with the stored-override count, opening the
 * full list. ReleasePlanner mounts it only behind the role check.
 */
export function OverridesManager(): React.ReactElement {
    const t: ListT = useT("tools");
    const { query } = useReleaseOverrides();
    const [open, setOpen] = React.useState(false);
    const count = query.data?.length ?? 0;
    return (
        <>
            <Button variant="outline" aria-label={t("release.overrides.openLabel", { count })} onClick={() => setOpen(true)}>
                <Pencil />
                {t("release.overrides.open")}
                {query.data && (
                    <Badge variant="secondary" size="sm" className="tabular-nums">
                        {count}
                    </Badge>
                )}
            </Button>
            <OverridesListSheet open={open} onClose={() => setOpen(false)} />
        </>
    );
}

function OverridesListSheet({ open, onClose }: { open: boolean; onClose: () => void }): React.ReactElement {
    const t: ListT = useT("tools");
    const describeError = useErrorMessage();
    const autoOn = useAutoTranslate();
    const fallbackName = useOverrideFallbackName();
    const { query, remove } = useReleaseOverrides();
    const events = useQuery({ ...releaseEventsQueryOptions(), enabled: open });
    const banners = useQuery({ ...releaseBannersQueryOptions(), enabled: open });
    const [editing, setEditing] = React.useState<IOverrideTarget | null>(null);
    const [removing, setRemoving] = React.useState<ReleaseOverride | null>(null);

    // Names and current resolutions come from the planner's own queries, which the tabs have
    // usually loaded already; a kind they do not cover falls back to its CN id.
    const rowInfo = React.useMemo(() => {
        const map = new Map<string, IRowInfo>();
        for (const e of events.data?.events ?? []) map.set(`activity:${e.cnId}`, { name: resolveName(e.nameCn, e.nameEn, e.nameEnAuto, autoOn).text, resolution: e.resolution });
        for (const b of banners.data?.banners ?? []) map.set(`pool:${b.cnPoolId}`, { name: resolveName(b.nameCn, null, b.nameEnAuto, autoOn).text, resolution: b.resolution });
        return map;
    }, [events.data, banners.data, autoOn]);
    const nameOf = (o: ReleaseOverride): string => rowInfo.get(`${o.kind}:${o.cnId}`)?.name ?? o.enName ?? fallbackName(o.kind, o.cnId);

    // Undated rows last; the backend allows a null start even though this editor never saves one.
    const rows = React.useMemo(() => [...(query.data ?? [])].sort((a, b) => (a.enStart ?? Number.POSITIVE_INFINITY) - (b.enStart ?? Number.POSITIVE_INFINITY)), [query.data]);

    const edit = (o: ReleaseOverride): void => {
        if (!isOverrideKind(o.kind)) return;
        const info = rowInfo.get(`${o.kind}:${o.cnId}`);
        setEditing({ mode: "row", kind: o.kind, cnId: o.cnId, name: nameOf(o), resolution: info?.resolution ?? null });
    };

    const confirmRemove = (): void => {
        if (!removing) return;
        const name = nameOf(removing);
        remove.mutate(
            { kind: removing.kind, cnId: removing.cnId },
            {
                onSuccess: () => {
                    toastSuccess("release-override-remove", t("release.override.toast.removed"), t("release.override.toast.removedDesc", { name }));
                    setRemoving(null);
                },
                onError: (e: unknown) => toastError("release-override-remove-err", t("release.override.toast.removeFailed"), describeError(e)),
            },
        );
    };

    return (
        <Sheet open={open} onOpenChange={(next) => (next ? undefined : onClose())}>
            <SheetPopup side="right" variant="inset" className="max-w-5xl">
                <SheetHeader>
                    <SheetTitle>{t("release.overrides.title")}</SheetTitle>
                    <SheetDescription>{t("release.overrides.description")}</SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-3">
                    <div>
                        <Button size="sm" onClick={() => setEditing({ mode: "new", kind: "activity", cnId: "" })}>
                            <Plus />
                            {t("release.overrides.add")}
                        </Button>
                    </div>
                    {query.isPending ? (
                        <ReleaseLoading />
                    ) : query.isError ? (
                        <ReleaseError error={query.error} onRetry={() => query.refetch()} />
                    ) : rows.length === 0 ? (
                        <ReleaseEmpty title={t("release.overrides.empty.title")} description={t("release.overrides.empty.description")} />
                    ) : (
                        <OverridesTable rows={rows} nameOf={nameOf} onEdit={edit} onRemove={setRemoving} t={t} />
                    )}
                </SheetPanel>
                <OverrideSheet target={editing} onClose={() => setEditing(null)} />
                <RemoveOverrideDialog open={removing !== null} pending={remove.isPending} onConfirm={confirmRemove} onClose={() => setRemoving(null)} />
            </SheetPopup>
        </Sheet>
    );
}

interface IOverridesTableProps {
    rows: readonly ReleaseOverride[];
    nameOf: (o: ReleaseOverride) => string;
    onEdit: (o: ReleaseOverride) => void;
    onRemove: (o: ReleaseOverride) => void;
    t: ListT;
}

function OverridesTable({ rows, nameOf, onEdit, onRemove, t }: IOverridesTableProps): React.ReactElement {
    const locale = useLocale();
    const timeZone = useReleaseTimeZone();
    const f = useFormatters();
    const kindLabel = useOverrideKindLabel();
    return (
        <Table>
            <TableHeader>
                <TableRow>
                    <TableHead>{t("release.overrides.col.kind")}</TableHead>
                    <TableHead>{t("release.overrides.col.item")}</TableHead>
                    <TableHead>{t("release.overrides.col.start")}</TableHead>
                    <TableHead>{t("release.overrides.col.end")}</TableHead>
                    <TableHead className="max-md:hidden">{t("release.overrides.col.source")}</TableHead>
                    <TableHead className="max-lg:hidden">{t("release.overrides.col.note")}</TableHead>
                    <TableHead className="max-md:hidden">{t("release.overrides.col.updated")}</TableHead>
                    <TableHead>
                        <span className="sr-only">{t("release.overrides.col.actions")}</span>
                    </TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {rows.map((o) => {
                    const name = nameOf(o);
                    const editable = isOverrideKind(o.kind);
                    return (
                        <TableRow key={`${o.kind}:${o.cnId}`}>
                            <TableCell className="whitespace-nowrap">{kindLabel(o.kind)}</TableCell>
                            <TableCell>
                                <div className="flex max-w-[28ch] flex-col gap-0.5">
                                    {name !== o.cnId && (
                                        <span className="truncate font-medium" title={name}>
                                            {name}
                                        </span>
                                    )}
                                    <span className="truncate font-mono text-[11.5px] text-muted-foreground" title={o.cnId}>
                                        {o.cnId}
                                    </span>
                                </div>
                            </TableCell>
                            <TableCell className="whitespace-nowrap tabular-nums">{o.enStart === null ? t("release.overrides.none") : formatDate(o.enStart, locale, timeZone)}</TableCell>
                            <TableCell className="whitespace-nowrap tabular-nums">{o.enEnd === null ? <span className="text-muted-foreground">{t("release.overrides.none")}</span> : formatDate(o.enEnd, locale, timeZone)}</TableCell>
                            <TableCell className="max-w-[16ch] truncate max-md:hidden" title={o.source}>
                                {o.source}
                            </TableCell>
                            <TableCell className="max-lg:hidden">
                                {o.note ? (
                                    <Tooltip>
                                        <TooltipTrigger render={<span className="block max-w-[24ch] truncate" />}>{o.note}</TooltipTrigger>
                                        <TooltipPopup className="max-w-xs">{o.note}</TooltipPopup>
                                    </Tooltip>
                                ) : null}
                            </TableCell>
                            <TableCell className="whitespace-nowrap text-muted-foreground max-md:hidden">{f.relative(new Date(o.updatedAt * 1000).toISOString())}</TableCell>
                            <TableCell>
                                <div className="flex justify-end gap-0.5">
                                    {editable && (
                                        <Button variant="ghost" size="icon" aria-label={t("release.overrides.edit", { name })} onClick={() => onEdit(o)}>
                                            <Pencil />
                                        </Button>
                                    )}
                                    <Button variant="ghost" size="icon" className="text-destructive-foreground" aria-label={t("release.overrides.remove", { name })} onClick={() => onRemove(o)}>
                                        <Trash2 />
                                    </Button>
                                </div>
                            </TableCell>
                        </TableRow>
                    );
                })}
            </TableBody>
        </Table>
    );
}
