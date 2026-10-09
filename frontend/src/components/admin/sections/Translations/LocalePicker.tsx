import { useEffect, useRef } from "react";
import { Badge } from "#/components/ui/badge";
import { Card } from "#/components/ui/card";
import { Progress } from "#/components/ui/progress";
import { Skeleton } from "#/components/ui/skeleton";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { Locale } from "#/types/generated/Locale";
import type { LocaleProgress } from "#/types/generated/LocaleProgress";
import { localeStats } from "./model";
import type { messages } from "./Translations.messages";

type T = TypedT<typeof messages>;

/**
 * The locale cards: a sideways snap scroller below lg (ten cards in a stack, or even two
 * columns on a tablet, push the work area a screen down), a grid from lg up. The scroller
 * bleeds into the page gutter, which is px-4 on a phone and px-8 from md.
 */
export const LOCALE_PICKER = "items-start gap-3 max-lg:-mx-4 max-lg:flex max-lg:snap-x max-lg:snap-mandatory max-lg:scroll-px-4 max-lg:overflow-x-auto max-lg:px-4 max-lg:py-0.5 md:max-lg:-mx-8 md:max-lg:scroll-px-8 md:max-lg:px-8 lg:grid lg:grid-cols-3 xl:grid-cols-4";
export const LOCALE_PICKER_ITEM = "max-lg:w-[min(80%,16rem)] max-lg:shrink-0 max-lg:snap-start";

/** Below lg the selected locale card can sit past the scroller's edge: bring it in once `ready`, sideways only. Attach the ref to the picker. */
export function useSelectedCardInView(ready: boolean): React.RefObject<HTMLDivElement | null> {
    const pickerRef = useRef<HTMLDivElement | null>(null);
    useEffect(() => {
        const box = pickerRef.current;
        if (!ready || !box || box.scrollWidth <= box.clientWidth) return;
        const card = box.querySelector<HTMLElement>("[aria-pressed=true]");
        if (!card) return;
        const left = card.getBoundingClientRect().left - box.getBoundingClientRect().left + box.scrollLeft;
        // Land it on the snap line, which is the scroller's own padding (it grows from md).
        const inset = Number.parseFloat(getComputedStyle(box).scrollPaddingLeft) || 0;
        if (left < box.scrollLeft || left + card.offsetWidth > box.scrollLeft + box.clientWidth) box.scrollLeft = left - inset;
    }, [ready]);
    return pickerRef;
}

export function LocaleCard({ locale, progress, progressPending, viewOnly, selected, onSelect }: { locale: Locale; progress: LocaleProgress | undefined; progressPending: boolean; viewOnly: boolean; selected: boolean; onSelect: () => void }): React.ReactElement {
    const t: T = useT("admin");
    const fmt = useFormatters();
    const stats = localeStats(progress);
    return (
        <button
            type="button"
            aria-pressed={selected}
            onClick={onSelect}
            className={cn(LOCALE_PICKER_ITEM, "cursor-pointer rounded-2xl text-left outline-none transition-shadow focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background", selected ? "shadow-[0_0_0_2px_var(--primary)]" : "shadow-[0_0_0_2px_transparent]")}
        >
            <Card className="gap-2.5 px-[18px] py-4">
                <div className="flex min-w-0 items-center gap-2">
                    <span className="truncate font-semibold text-[14.5px]" title={locale.english_name}>
                        {locale.english_name}
                    </span>
                    <span className="truncate text-[13px] text-muted-foreground" lang={locale.code} title={locale.native_name}>
                        {locale.native_name}
                    </span>
                    {!locale.enabled ? (
                        <Badge variant="outline" size="sm">
                            {t("translations.locale.hidden")}
                        </Badge>
                    ) : null}
                    {viewOnly ? (
                        <Badge variant="secondary" size="sm">
                            {t("translations.locale.viewOnly")}
                        </Badge>
                    ) : null}
                    {progress ? <span className="ml-auto shrink-0 font-semibold text-[13px] tabular-nums">{t("translations.locale.pct", { pct: fmt.number(stats.pct) })}</span> : null}
                </div>
                <Progress value={progress ? stats.pct : null} aria-label={locale.english_name} />
                {progress ? <span className="text-[12.5px] text-muted-foreground">{stats.todo > 0 ? t("translations.locale.line", { stale: fmt.number(stats.stale), missing: fmt.number(stats.missing) }) : t("translations.locale.complete")}</span> : progressPending ? <Skeleton className="h-4 w-40" /> : null}
            </Card>
        </button>
    );
}
