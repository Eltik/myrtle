import { useCnChangeSummary } from "#/components/operators/list/impl/components/CnUpdateBadge";
import type { messages as cnBadgeMessages } from "#/components/operators/list/impl/components/CnUpdateBadge.messages";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { messages } from "./CnMarkers.messages";

type MarkersT = TypedT<typeof messages>;

/** Marks a module or skill that comes from CN's data, beside its name wherever the page lists it.
 *  The text takes its colour from the name beside it and the ring carries the red: primary text
 *  measured 4.03 on the muted surfaces the tag sits on, and white art captions need it white. */
export function CnTag({ className, title }: { className?: string; title?: string }) {
    const t: MarkersT = useT("operators");
    const label = title ?? t("cnUpdates.tag.title");
    return (
        <span role="img" aria-label={label} title={label} className={cn("inline-flex shrink-0 items-center rounded-sm px-1 py-px font-mono font-semibold text-[9px] uppercase leading-none tracking-wider ring-1 ring-primary", className)}>
            {t("cnUpdates.tag")}
        </span>
    );
}

/** CN's value, beside a number the page shows, where the two servers differ. Always visible, so it reads on touch too.
 *  Foreground on the tint, red on the ring: primary text on its own 10% tint is under 4.5:1. */
export function CnValue({ value, className }: { value: string; className?: string }) {
    const t: MarkersT = useT("operators");
    const label = t("cnUpdates.value.title", { value });
    return (
        <span title={label} className={cn("ms-2 inline-flex items-baseline gap-1 rounded-sm bg-primary/10 px-1 py-px font-mono text-[11px] text-foreground tabular-nums ring-1 ring-primary", className)}>
            <span aria-hidden="true" className="font-semibold">
                {t("cnUpdates.tag")}
            </span>
            <span aria-hidden="true">{value}</span>
            <span className="sr-only">{label}</span>
        </span>
    );
}

/**
 * A description repeated under the page's own as CN has it: this server's text with CN's numbers,
 * the changed ones marked, or with `lang` set, CN's own text where this server's no longer fits
 * CN's numbers. `html` is already sanitized.
 */
export function CnDescription({ html, lang, className }: { html: string; lang?: string; className?: string }) {
    const t: MarkersT = useT("operators");
    return (
        <div className={cn("mt-1.5 flex items-baseline gap-2", className)}>
            <CnTag title={lang ? t("cnUpdates.text.own.title") : t("cnUpdates.text.title")} />
            <span
                lang={lang}
                className="text-muted-foreground text-xs"
                // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                dangerouslySetInnerHTML={{ __html: html }}
            />
        </div>
    );
}

/** What CN changed for this operator, as one line of text: on the list it is only the Updated pill's title, which touch and keyboard cannot reach. */
export function CnChangeSummary({ changes, className }: { changes: readonly OperatorChange[]; className?: string }) {
    const t: TypedT<typeof cnBadgeMessages> = useT("operators");
    const summary = useCnChangeSummary(changes);
    return <p className={cn("text-muted-foreground text-sm", className)}>{t("cnUpdates.badge.title", { changes: summary })}</p>;
}
