import { XIcon } from "lucide-react";
import { useId, useMemo } from "react";
import { Button } from "#/components/ui/button";
import { Combobox, ComboboxChip, ComboboxChips, ComboboxChipsInput, ComboboxEmpty, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxValue } from "#/components/ui/combobox";
import { Field, FieldLabel } from "#/components/ui/field";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./BannerFilter.messages";
import type { IBannerOption } from "./banner-filter";

interface IBannerFilterProps {
    /** Every banner the history has a pull on (`bannerOptions`). */
    options: readonly IBannerOption[];
    /** The chosen pool ids; empty is every banner. */
    value: readonly string[];
    onChange: (next: string[]) => void;
    /** The menu opened or closed: lets a caller load the full banner list only when it is wanted. */
    onOpenChange?: (open: boolean) => void;
    disabled?: boolean;
}

/**
 * Limits the history to one or more banners: a searchable menu whose choices sit as chips
 * in its field. A chosen id the history has no pull on (a shared link to a banner since
 * emptied) keeps a chip under its bare id, so the reader sees why the page is empty.
 */
export function BannerFilter({ options, value, onChange, onOpenChange, disabled }: IBannerFilterProps) {
    const t: TypedT<typeof messages> = useT("gacha");
    const id = useId();
    const selected = useMemo(() => {
        const byId = new Map(options.map((o) => [o.poolId, o]));
        return value.map((poolId) => byId.get(poolId) ?? { poolId, label: poolId, pulls: 0, lastAt: 0 });
    }, [options, value]);

    return (
        <Field className="min-w-0 max-w-full gap-1.5 sm:flex-row sm:items-center sm:gap-2">
            <FieldLabel htmlFor={id} className="whitespace-nowrap font-bold font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.16em]">
                {t("history.filter.label")}
            </FieldLabel>
            <div className="flex min-w-0 items-center gap-1.5">
                <Combobox<IBannerOption, true>
                    multiple
                    items={options as IBannerOption[]}
                    value={selected}
                    onValueChange={(next) => onChange(next.map((o) => o.poolId))}
                    itemToStringLabel={(o) => o.label}
                    itemToStringValue={(o) => o.poolId}
                    isItemEqualToValue={(a, b) => a.poolId === b.poolId}
                    onOpenChange={onOpenChange}
                    disabled={disabled}
                >
                    <ComboboxChips className="min-w-0 sm:min-w-72 sm:max-w-xl">
                        <ComboboxValue>
                            {(chips: IBannerOption[]) => (
                                <>
                                    {chips.map((o) => (
                                        <ComboboxChip key={o.poolId} aria-label={o.label}>
                                            {o.label}
                                        </ComboboxChip>
                                    ))}
                                    <ComboboxChipsInput id={id} size="sm" aria-label={t("history.filter.label")} placeholder={chips.length === 0 ? t("history.filter.placeholder") : ""} />
                                </>
                            )}
                        </ComboboxValue>
                    </ComboboxChips>
                    <ComboboxPopup className="w-[min(360px,calc(100vw-2rem))]">
                        <ComboboxEmpty>{t("history.filter.noMatches")}</ComboboxEmpty>
                        <ComboboxList>
                            {(o: IBannerOption) => (
                                <ComboboxItem key={o.poolId} value={o}>
                                    <span className="flex min-w-0 flex-1 items-center justify-between gap-3">
                                        <span className="truncate">{o.label}</span>
                                        <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{t("history.filter.pulls", { count: o.pulls })}</span>
                                    </span>
                                </ComboboxItem>
                            )}
                        </ComboboxList>
                    </ComboboxPopup>
                </Combobox>
                {value.length > 0 ? (
                    <Button type="button" variant="ghost" size="icon-sm" aria-label={t("history.filter.clear")} title={t("history.filter.clear")} onClick={() => onChange([])} disabled={disabled}>
                        <XIcon />
                    </Button>
                ) : null}
            </div>
        </Field>
    );
}
