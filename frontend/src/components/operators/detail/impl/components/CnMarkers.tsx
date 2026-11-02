import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./CnMarkers.messages";

type MarkersT = TypedT<typeof messages>;

/** Marks a module or skill that comes from CN's data, beside its name wherever the page lists it. */
export function CnTag({ className, title }: { className?: string; title?: string }) {
    const t: MarkersT = useT("operators");
    const label = title ?? t("cnUpdates.tag.title");
    return (
        <span role="img" aria-label={label} title={label} className={cn("inline-flex shrink-0 items-center rounded-sm px-1 py-px font-mono font-semibold text-[9px] text-primary uppercase leading-none tracking-wider ring-1 ring-primary/40", className)}>
            {t("cnUpdates.tag")}
        </span>
    );
}

/** CN's value, beside a number the page shows, where the two servers differ. Always visible, so it reads on touch too. */
export function CnValue({ value, className }: { value: string; className?: string }) {
    const t: MarkersT = useT("operators");
    const label = t("cnUpdates.value.title", { value });
    return (
        <span title={label} className={cn("ms-2 inline-flex items-baseline gap-1 rounded-sm bg-primary/10 px-1 py-px font-mono text-[10px] text-primary tabular-nums ring-1 ring-primary/30", className)}>
            <span aria-hidden="true" className="font-semibold">
                {t("cnUpdates.tag")}
            </span>
            <span aria-hidden="true">{value}</span>
            <span className="sr-only">{label}</span>
        </span>
    );
}

/** A description repeated with CN's numbers under the page's own, the changed numbers marked; `html` is already sanitized. */
export function CnDescription({ html, className }: { html: string; className?: string }) {
    const t: MarkersT = useT("operators");
    return (
        <div className={cn("mt-1.5 flex items-baseline gap-2", className)}>
            <CnTag title={t("cnUpdates.text.title")} />
            <span
                className="text-muted-foreground text-xs"
                // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                dangerouslySetInnerHTML={{ __html: html }}
            />
        </div>
    );
}
