import { TriangleAlert } from "lucide-react";
import * as React from "react";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Sheet, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { Spinner } from "#/components/ui/spinner";
import { Textarea } from "#/components/ui/textarea";
import { useLastDefined } from "#/hooks/use-last-defined";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Resolution } from "#/types/generated/Resolution";
import { formatDate, formatDateRange, useReleaseTimeZone } from "../../helpers";
import type { messages as helperMessages } from "../../helpers.messages";
import { type IOverrideForm, OVERRIDE_KINDS, type OverrideFormError, type OverrideFormField, type OverrideKind, overrideForm, overrideKey, putOverrideInput, validateOverrideForm } from "../../overrides";
import { useReleaseOverrides } from "../../useReleaseOverrides";
import type { messages } from "./OverrideSheet.messages";

/** The sheet renders its own chrome plus the date wording `helpers.ts` derives. */
type OverrideT = TypedT<typeof messages & typeof helperMessages>;

const KIND_LABEL_KEYS: Record<OverrideKind, keyof typeof messages & string> = {
    activity: "release.override.kind.activity",
    pool: "release.override.kind.pool",
    skin: "release.override.kind.skin",
    review: "release.override.kind.review",
};

const CN_ID_HINT_KEYS: Record<OverrideKind, keyof typeof messages & string> = {
    activity: "release.override.field.cnIdHint.activity",
    pool: "release.override.field.cnIdHint.pool",
    skin: "release.override.field.cnIdHint.skin",
    review: "release.override.field.cnIdHint.review",
};

const ERROR_KEYS: Record<OverrideFormError, keyof typeof messages & string> = {
    cnIdRequired: "release.override.err.cnIdRequired",
    startRequired: "release.override.err.startRequired",
    startInvalid: "release.override.err.startInvalid",
    endInvalid: "release.override.err.endInvalid",
    endBeforeStart: "release.override.err.endBeforeStart",
};

const ALL_FIELDS: readonly OverrideFormField[] = ["cnId", "enStart", "enEnd"];

/** Kind labels, shared with the override list so both name a kind the same way. */
export function useOverrideKindLabel(): (kind: string) => string {
    const t: OverrideT = useT("tools");
    return (kind) => (kind in KIND_LABEL_KEYS ? t(KIND_LABEL_KEYS[kind as OverrideKind]) : kind);
}

/**
 * A name for an override's item when the caller has none: a Fashion Review's CN id is its CN
 * start in seconds, so it reads as that date; anything else falls back to the id.
 */
export function useOverrideFallbackName(): (kind: string, cnId: string) => string {
    const t: OverrideT = useT("tools");
    const locale = useLocale();
    const timeZone = useReleaseTimeZone();
    return (kind, cnId) => {
        const seconds = Number(cnId);
        if (kind === "review" && Number.isFinite(seconds) && seconds > 0) return t("release.override.reviewItem", { date: formatDate(seconds, locale, timeZone) });
        return cnId;
    };
}

export interface IOverrideTarget {
    /** `new` lets the editor pick the kind and CN id; a row fixes both. */
    mode: "row" | "new";
    kind: OverrideKind;
    cnId: string;
    /** The row's display name, when the caller has one. */
    name?: string | null;
    /** What the row shows today, when the caller has it. */
    resolution?: Resolution | null;
}

interface IOverrideSheetProps {
    /** Null closes the sheet. */
    target: IOverrideTarget | null;
    onClose: () => void;
}

/**
 * Sets, edits or removes the manual EN date of one release row. Super admins only: callers
 * mount it behind `canEdit`. The existing override is read from the shared list, so a row and
 * the manage list always edit the same stored row.
 */
export function OverrideSheet({ target, onClose }: IOverrideSheetProps): React.ReactElement {
    const shown = useLastDefined(target);
    const { query } = useReleaseOverrides();
    return (
        <Sheet open={target !== null} onOpenChange={(next) => (next ? undefined : onClose())}>
            <SheetPopup side="right" variant="inset">
                {shown && (query.isPending ? <OverrideSheetLoading /> : <OverrideForm key={`${shown.mode}:${overrideKey(shown.kind, shown.cnId)}`} target={shown} onClose={onClose} />)}
            </SheetPopup>
        </Sheet>
    );
}

function OverrideSheetLoading(): React.ReactElement {
    const t: OverrideT = useT("tools");
    return (
        <>
            <SheetHeader>
                <SheetTitle>{t("release.override.title.add")}</SheetTitle>
            </SheetHeader>
            <SheetPanel className="flex items-center justify-center py-10">
                <Spinner className="size-5 text-muted-foreground" />
            </SheetPanel>
        </>
    );
}

function OverrideForm({ target, onClose }: { target: IOverrideTarget; onClose: () => void }): React.ReactElement {
    const t: OverrideT = useT("tools");
    const locale = useLocale();
    const timeZone = useReleaseTimeZone();
    const describeError = useErrorMessage();
    const kindLabel = useOverrideKindLabel();
    const fallbackName = useOverrideFallbackName();
    const { overrides, save, remove } = useReleaseOverrides();
    const baseId = React.useId();
    const isNew = target.mode === "new";
    const [form, setForm] = React.useState<IOverrideForm>(() => overrideForm(target.kind, target.cnId, overrides.get(overrideKey(target.kind, target.cnId)) ?? null));
    const [touched, setTouched] = React.useState<ReadonlySet<OverrideFormField>>(() => new Set());
    const [confirming, setConfirming] = React.useState(false);

    const cnId = form.cnId.trim();
    // Looked up on what the form says, so a new override typed onto a stored row still carries
    // that row's hidden fields across the full-replace PUT.
    const existing = overrides.get(overrideKey(form.kind, cnId)) ?? null;
    const name = target.name || existing?.enName || fallbackName(form.kind, cnId);
    const errors = validateOverrideForm(form);
    const invalid = Object.keys(errors).length > 0;
    const err = (field: OverrideFormField): string | undefined => {
        const e = touched.has(field) ? errors[field] : undefined;
        return e ? t(ERROR_KEYS[e]) : undefined;
    };
    const set = <K extends keyof IOverrideForm>(key: K, value: IOverrideForm[K]): void => setForm((prev) => ({ ...prev, [key]: value }));
    const touch = (field: OverrideFormField): void => setTouched((prev) => (prev.has(field) ? prev : new Set(prev).add(field)));

    const submit = (event: React.FormEvent): void => {
        event.preventDefault();
        setTouched(new Set(ALL_FIELDS));
        if (invalid) return;
        save.mutate(putOverrideInput(form, existing), {
            onSuccess: (saved) => {
                const date = saved.enStart === null ? "" : formatDateRange(saved.enStart, saved.enEnd, locale, t, timeZone);
                toastSuccess("release-override-save", t("release.override.toast.saved"), t("release.override.toast.savedDesc", { name, date }));
                onClose();
            },
            onError: (e: unknown) => toastError("release-override-save-err", t("release.override.toast.saveFailed"), describeError(e)),
        });
    };

    const removeExisting = (): void => {
        if (!existing) return;
        remove.mutate(
            { kind: existing.kind, cnId: existing.cnId },
            {
                onSuccess: () => {
                    toastSuccess("release-override-remove", t("release.override.toast.removed"), t("release.override.toast.removedDesc", { name }));
                    setConfirming(false);
                    onClose();
                },
                onError: (e: unknown) => toastError("release-override-remove-err", t("release.override.toast.removeFailed"), describeError(e)),
            },
        );
    };

    return (
        <>
            <form onSubmit={submit} className="flex min-h-0 flex-1 flex-col" noValidate>
                <SheetHeader>
                    <SheetTitle>{existing && !isNew ? t("release.override.title.edit") : t("release.override.title.add")}</SheetTitle>
                    <SheetDescription>
                        {name !== cnId ? t("release.override.subtitle", { name, kind: kindLabel(form.kind) }) : kindLabel(form.kind)}
                        {cnId && <span className="ml-1.5 break-all font-mono text-[12px]">{cnId}</span>}
                    </SheetDescription>
                </SheetHeader>
                <SheetPanel className="flex flex-col gap-4">
                    {target.resolution && <CurrentResolution resolution={target.resolution} t={t} />}
                    {isNew && existing && <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">{t("release.override.replacesExisting")}</p>}
                    {isNew && (
                        <>
                            <FormRow id={`${baseId}-kind`} label={t("release.override.field.kind")}>
                                <Select value={form.kind} onValueChange={(v) => v && set("kind", v as OverrideKind)}>
                                    <SelectTrigger id={`${baseId}-kind`}>
                                        <SelectValue>{(value) => kindLabel(String(value))}</SelectValue>
                                    </SelectTrigger>
                                    <SelectContent>
                                        {OVERRIDE_KINDS.map((k) => (
                                            <SelectItem key={k} value={k}>
                                                {kindLabel(k)}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </FormRow>
                            <FormRow id={`${baseId}-cn`} label={t("release.override.field.cnId")} hint={t(CN_ID_HINT_KEYS[form.kind])} error={err("cnId")}>
                                {(describedBy) => <Input id={`${baseId}-cn`} className="font-mono" autoComplete="off" aria-invalid={err("cnId") ? true : undefined} aria-describedby={describedBy} value={form.cnId} onChange={(e) => set("cnId", e.target.value)} onBlur={() => touch("cnId")} />}
                            </FormRow>
                        </>
                    )}
                    <FormRow id={`${baseId}-start`} label={t("release.override.field.enStart")} hint={t("release.override.field.enStartHint")} error={err("enStart")}>
                        {(describedBy) => <Input id={`${baseId}-start`} type="datetime-local" className="tabular-nums" required aria-invalid={err("enStart") ? true : undefined} aria-describedby={describedBy} value={form.enStart} onChange={(e) => set("enStart", e.target.value)} onBlur={() => touch("enStart")} />}
                    </FormRow>
                    <FormRow id={`${baseId}-end`} label={t("release.override.field.enEnd")} hint={t("release.override.field.enEndHint")} error={err("enEnd")}>
                        {(describedBy) => <Input id={`${baseId}-end`} type="datetime-local" className="tabular-nums" aria-invalid={err("enEnd") ? true : undefined} aria-describedby={describedBy} value={form.enEnd} onChange={(e) => set("enEnd", e.target.value)} onBlur={() => touch("enEnd")} />}
                    </FormRow>
                    <FormRow id={`${baseId}-enid`} label={t("release.override.field.enId")} hint={t("release.override.field.enIdHint")}>
                        {(describedBy) => <Input id={`${baseId}-enid`} className="font-mono" autoComplete="off" aria-describedby={describedBy} value={form.enId} onChange={(e) => set("enId", e.target.value)} />}
                    </FormRow>
                    <FormRow id={`${baseId}-enname`} label={t("release.override.field.enName")} hint={t("release.override.field.enNameHint")}>
                        {(describedBy) => <Input id={`${baseId}-enname`} autoComplete="off" aria-describedby={describedBy} value={form.enName} onChange={(e) => set("enName", e.target.value)} />}
                    </FormRow>
                    <FormRow id={`${baseId}-source`} label={t("release.override.field.source")} hint={t("release.override.field.sourceHint")}>
                        {/* The placeholder is the backend's stored default, not copy. */}
                        {(describedBy) => <Input id={`${baseId}-source`} autoComplete="off" placeholder="manual" aria-describedby={describedBy} value={form.source} onChange={(e) => set("source", e.target.value)} />}
                    </FormRow>
                    <FormRow id={`${baseId}-note`} label={t("release.override.field.note")}>
                        <Textarea id={`${baseId}-note`} value={form.note} onChange={(e) => set("note", e.target.value)} />
                    </FormRow>
                </SheetPanel>
                <SheetFooter>
                    {existing && !isNew && (
                        <Button type="button" variant="destructive-outline" className="sm:mr-auto" onClick={() => setConfirming(true)}>
                            {t("release.override.remove")}
                        </Button>
                    )}
                    <Button type="button" variant="ghost" onClick={onClose}>
                        {t("release.override.cancel")}
                    </Button>
                    <Button type="submit" loading={save.isPending} disabled={save.isPending || (touched.size > 0 && invalid)}>
                        {t("release.override.save")}
                    </Button>
                </SheetFooter>
            </form>
            <RemoveOverrideDialog open={confirming} pending={remove.isPending} onConfirm={removeExisting} onClose={() => setConfirming(false)} />
        </>
    );
}

/** What the row shows today, so the editor sees what they are about to replace. */
function CurrentResolution({ resolution, t }: { resolution: Resolution; t: OverrideT }): React.ReactElement {
    const locale = useLocale();
    const timeZone = useReleaseTimeZone();
    const line = "m-0 font-sans text-[12.5px] text-muted-foreground leading-normal";
    switch (resolution.status) {
        case "estimated":
            return <p className={line}>{t("release.override.current.estimated", { date: formatDate(resolution.enStart, locale, timeZone), lo: formatDate(resolution.lo, locale, timeZone), hi: formatDate(resolution.hi, locale, timeZone) })}</p>;
        case "override":
            return <p className={line}>{t("release.override.current.override", { date: formatDateRange(resolution.enStart, resolution.enEnd, locale, t, timeZone), source: resolution.source })}</p>;
        case "confirmed":
            // The resolver ranks a gamedata sighting above an override, so a save here is stored
            // and shown nowhere until EN drops the row. Say so before the editor saves.
            return (
                <div className="flex flex-col gap-2">
                    <p className={line}>{t("release.override.current.confirmed", { date: formatDateRange(resolution.enStart, resolution.enEnd, locale, t, timeZone) })}</p>
                    <Alert variant="warning">
                        <TriangleAlert />
                        <AlertDescription>{t("release.override.current.confirmedNote")}</AlertDescription>
                    </Alert>
                </div>
            );
        default:
            return <p className={line}>{t("release.override.current.none")}</p>;
    }
}

interface IRemoveOverrideDialogProps {
    open: boolean;
    pending: boolean;
    onConfirm: () => void;
    onClose: () => void;
}

/** The confirmation before an override is dropped; the manage list reuses it for its row action. */
export function RemoveOverrideDialog({ open, pending, onConfirm, onClose }: IRemoveOverrideDialogProps): React.ReactElement {
    const t: OverrideT = useT("tools");
    return (
        <AlertDialog open={open} onOpenChange={(next) => !next && onClose()}>
            <AlertDialogPopup className="max-w-md">
                <AlertDialogHeader>
                    <AlertDialogTitle>{t("release.override.remove.title")}</AlertDialogTitle>
                    <AlertDialogDescription>{t("release.override.remove.body")}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button variant="ghost" />}>{t("release.override.cancel")}</AlertDialogClose>
                    <Button variant="destructive" loading={pending} onClick={onConfirm}>
                        {t("release.override.remove")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}

interface IFormRowProps {
    id: string;
    label: string;
    hint?: string;
    error?: string;
    /** A render function receives the id of the hint/error line, for the control's `aria-describedby`. */
    children: React.ReactNode | ((describedBy: string | undefined) => React.ReactNode);
}

function FormRow({ id, label, hint, error, children }: IFormRowProps): React.ReactElement {
    const descId = `${id}-desc`;
    const text = error ?? hint;
    return (
        <div className="flex flex-col gap-1.5">
            <Label htmlFor={id} className={cn(error && "text-destructive-foreground")}>
                {label}
            </Label>
            {typeof children === "function" ? children(text ? descId : undefined) : children}
            {text && (
                <p id={descId} className={cn("m-0 text-[12px] leading-normal", error ? "text-destructive-foreground" : "text-muted-foreground")}>
                    {text}
                </p>
            )}
        </div>
    );
}
