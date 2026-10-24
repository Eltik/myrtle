import type { PutOverride } from "#/types/generated/PutOverride";
import type { ReleaseOverride } from "#/types/generated/ReleaseOverride";

/**
 * The override kinds the editor offers, in the backend's own spelling. `retro` is accepted
 * there too but nothing reads it, so it is left out rather than offered as a dead end.
 */
export const OVERRIDE_KINDS = ["activity", "pool", "skin", "review"] as const;

export type OverrideKind = (typeof OVERRIDE_KINDS)[number];

export function isOverrideKind(kind: string): kind is OverrideKind {
    return (OVERRIDE_KINDS as readonly string[]).includes(kind);
}

/** The map key for one override row, matching the backend's (kind, cnId) upsert key. */
export function overrideKey(kind: string, cnId: string): string {
    return `${kind}:${cnId}`;
}

export function overrideMap(rows: readonly ReleaseOverride[] | undefined): Map<string, ReleaseOverride> {
    const map = new Map<string, ReleaseOverride>();
    for (const o of rows ?? []) map.set(overrideKey(o.kind, o.cnId), o);
    return map;
}

/**
 * The override a schedule item can take, read off its key (`schedule.ts` builds `event:<cnId>`,
 * `banner:<cnPoolId>` and `skin:review:<cnStart>`). A skin or rerun item groups several skins
 * under one row, so no single skin id stands for it and it gets none.
 */
export function scheduleOverrideTarget(key: string): { kind: OverrideKind; cnId: string } | null {
    if (key.startsWith("event:")) return { kind: "activity", cnId: key.slice("event:".length) };
    if (key.startsWith("banner:")) return { kind: "pool", cnId: key.slice("banner:".length) };
    if (key.startsWith("skin:review:")) return { kind: "review", cnId: key.slice("skin:review:".length) };
    return null;
}

const INPUT_PATTERN = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?$/;

/**
 * Unix seconds as a `datetime-local` value, read in UTC. The input has no zone of its own, and
 * the form labels it UTC, so the browser's zone never enters the conversion.
 */
export function unixToUtcInput(seconds: number): string {
    return new Date(seconds * 1000).toISOString().slice(0, 16);
}

/**
 * A `datetime-local` value, read as UTC, to unix seconds. Null for an empty or malformed value,
 * and for a calendar date that does not exist (`Date.UTC` would roll 02-30 into March).
 */
export function utcInputToUnix(value: string): number | null {
    const m = INPUT_PATTERN.exec(value.trim());
    if (!m) return null;
    const [y, mo, d, h, mi, s] = [m[1], m[2], m[3], m[4], m[5], m[6] ?? "0"].map(Number);
    const ms = Date.UTC(y, mo - 1, d, h, mi, s);
    const back = new Date(ms);
    if (back.getUTCFullYear() !== y || back.getUTCMonth() !== mo - 1 || back.getUTCDate() !== d || back.getUTCHours() !== h || back.getUTCMinutes() !== mi) return null;
    return ms / 1000;
}

export interface IOverrideForm {
    kind: OverrideKind;
    cnId: string;
    enStart: string;
    enEnd: string;
    enId: string;
    enName: string;
    source: string;
    note: string;
}

export type OverrideFormField = "cnId" | "enStart" | "enEnd";

export type OverrideFormError = "cnIdRequired" | "startRequired" | "startInvalid" | "endInvalid" | "endBeforeStart";

/** The form for an existing row, or a blank one; a new activity override starts its EN id at the CN id, the usual case. */
export function overrideForm(kind: OverrideKind, cnId: string, existing: ReleaseOverride | null): IOverrideForm {
    if (existing) {
        return {
            kind,
            cnId,
            enStart: existing.enStart === null ? "" : unixToUtcInput(existing.enStart),
            enEnd: existing.enEnd === null ? "" : unixToUtcInput(existing.enEnd),
            enId: existing.enId ?? "",
            enName: existing.enName ?? "",
            source: existing.source,
            note: existing.note,
        };
    }
    return { kind, cnId, enStart: "", enEnd: "", enId: kind === "activity" ? cnId : "", enName: "", source: "", note: "" };
}

export function validateOverrideForm(form: IOverrideForm): Partial<Record<OverrideFormField, OverrideFormError>> {
    const errors: Partial<Record<OverrideFormField, OverrideFormError>> = {};
    if (!form.cnId.trim()) errors.cnId = "cnIdRequired";
    const start = utcInputToUnix(form.enStart);
    if (!form.enStart.trim()) errors.enStart = "startRequired";
    else if (start === null) errors.enStart = "startInvalid";
    if (form.enEnd.trim()) {
        const end = utcInputToUnix(form.enEnd);
        if (end === null) errors.enEnd = "endInvalid";
        else if (start !== null && end <= start) errors.enEnd = "endBeforeStart";
    }
    return errors;
}

/**
 * The PUT body. The backend replaces the whole row, so every field the form does not show
 * (`featuredChars`) is carried over from the row being edited, or an edit would wipe it.
 */
export function putOverrideInput(form: IOverrideForm, existing: ReleaseOverride | null): PutOverride {
    const text = (v: string): string | null => v.trim() || null;
    return {
        kind: form.kind,
        cnId: form.cnId.trim(),
        enId: text(form.enId),
        enName: text(form.enName),
        featuredChars: existing?.featuredChars ?? null,
        enStart: utcInputToUnix(form.enStart),
        enEnd: form.enEnd.trim() ? utcInputToUnix(form.enEnd) : null,
        source: text(form.source),
        note: text(form.note),
    };
}
