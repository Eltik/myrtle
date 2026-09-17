import { Info } from "lucide-react";
import * as React from "react";
import { itemIcon } from "#/components/operators/detail/impl/assets";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { useLocale, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IBannerModel } from "../pulls/rates";
import type { messages } from "./PullsPlannerTab.messages";

export type PullsT = TypedT<typeof messages>;

/**
 * The tab's two type roles, declared once.
 *
 * Reported in #ui-ux: the same kind of text ran at three sizes depending on which
 * card it was in, and the small end was too small to read on a phone. FIELD is every
 * label attached to a control; HINT is the sentence under a group explaining it.
 * Nothing on this tab should be smaller than HINT.
 */
export const FIELD_LABEL = "font-medium font-sans text-[12.5px] text-muted-foreground";
export const HINT_TEXT = "font-sans text-[12px] text-muted-foreground leading-normal";

/**
 * The currencies this tab counts, as the game's own icons.
 *
 * Reported in #ui-ux: "OP/orundum/hh ticket/etc. icons desperately needed". The pair
 * is `(itemId, iconId)` straight out of `item_table.json`; `/api/item-icon` resolves
 * the icon id, so a display-name change cannot break these.
 */
const CURRENCY = {
    orundum: { id: "4003", icon: "DIAMOND_SHD" },
    originite: { id: "4002", icon: "DIAMOND" },
    permit: { id: "7003", icon: "TKT_GACHA" },
    tenPermit: { id: "7004", icon: "TKT_GACHA_10" },
    goldCert: { id: "4004", icon: "HGG_SHD" },
    greenCert: { id: "4005", icon: "LGG_SHD" },
    monthlyCard: { id: "mcardVoucher", icon: "mcardVoucher" },
} as const;

export type CurrencyName = keyof typeof CURRENCY;

/** Decorative: the word beside it is what carries the meaning, so this is `aria-hidden`. */
export function CurrencyIcon({ name, className }: { name: CurrencyName; className?: string }): React.ReactElement {
    const c = CURRENCY[name];
    return <img src={itemIcon(c.id, c.icon, null)} alt="" aria-hidden="true" decoding="async" loading="lazy" className={cn("size-4.5 shrink-0 object-contain", className)} />;
}

/** A label with its currency's icon in front of it. */
export function CurrencyLabel({ name, children, className }: { name: CurrencyName; children: React.ReactNode; className?: string }): React.ReactElement {
    return (
        <span className={cn("inline-flex items-center gap-1.5", className)}>
            <CurrencyIcon name={name} />
            {children}
        </span>
    );
}

/**
 * An explanation attached to the thing it explains.
 *
 * Reported in #ui-ux: a note at the BOTTOM of a card was read as a note about the
 * card rather than about the one field it qualified. `openOnHover` keeps it a hover
 * on a pointer, and it is still a button, so a tap opens it and a keyboard reaches it.
 */
export function InfoHint({ label, children }: { label: string; children: React.ReactNode }): React.ReactElement {
    return (
        <Popover>
            <PopoverTrigger
                openOnHover
                delay={120}
                render={(props) => (
                    <button {...props} type="button" aria-label={label} className="inline-flex size-4 cursor-help items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-2">
                        <Info className="size-3.5" aria-hidden="true" />
                    </button>
                )}
            />
            <PopoverPopup className="w-[min(320px,calc(100vw-2rem))]">
                <p className="m-0 font-sans text-[12.5px] text-foreground leading-normal">{children}</p>
            </PopoverPopup>
        </Popover>
    );
}

/**
 * Odds need more resolution than `useFormatters().percent` gives: the difference
 * between 96.4% and 96.42% is the difference between agreeing with an independent
 * implementation and appearing to disagree with it. The locale is still passed
 * explicitly, which is the part that matters.
 */
export function usePct(): (fraction: number, digits?: number) => string {
    const locale = useLocale();
    return React.useCallback(
        (fraction: number, digits = 2) => {
            const clamped = Math.min(1, Math.max(0, fraction));
            // Do not round a real chance up to a certainty it is not, or down to an
            // impossibility. The epsilon has to TRACK `digits`: at two fraction
            // digits the percent formatter rounds from 0.99995 upward, so a fixed
            // 0.9999999 guard sits orders of magnitude on the wrong side of the
            // threshold and lets a real 0.99998 render as a flat 100%.
            const eps = 5 * 10 ** -(digits + 3);
            const value = clamped >= 1 ? 1 : clamped <= 0 ? 0 : Math.min(Math.max(clamped, eps), 1 - eps);
            return new Intl.NumberFormat(locale, { style: "percent", minimumFractionDigits: 0, maximumFractionDigits: digits }).format(value);
        },
        [locale],
    );
}

interface INumberFieldProps {
    id: string;
    label: React.ReactNode;
    /**
     * Rings the field for a moment. Set on the boxes "Use my account's data" fills,
     * right after it is pressed: it was not obvious WHICH boxes it writes, and the
     * pity box, which it does not write, read as though it had been.
     */
    flash?: boolean;
    value: number;
    onChange: (value: number) => void;
    min?: number;
    max?: number;
    step?: number;
    className?: string;
}

export function PullsNumber({ id, label, flash, value, onChange, min = 0, max, step = 1, className }: INumberFieldProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-1">
            <Label htmlFor={id} className={FIELD_LABEL}>
                {label}
            </Label>
            <Input
                id={id}
                type="number"
                inputMode="numeric"
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(e) => {
                    const next = Number(e.target.value);
                    // `Number("")` is 0 and finite, so an empty box reads as zero
                    // rather than NaN. That is the wanted behaviour here, but it
                    // has to be deliberate: never trust a falsy check.
                    if (!Number.isFinite(next)) return;
                    onChange(Math.min(max ?? Number.MAX_SAFE_INTEGER, Math.max(min, next)));
                }}
                className={cn("h-8 w-28 text-right font-mono tabular-nums transition-shadow duration-300", flash && "ring-2 ring-primary ring-offset-1 ring-offset-background", className)}
            />
        </div>
    );
}

/**
 * The player-facing name of a banner archetype. `humanizeTag` title-cases the raw
 * gamedata enum, which is fine for a debug tag and wrong for a picker: the game calls
 * LINKAGE "Collaboration" and ATTAIN "Special Headhunting", and neither spelling is
 * a string a translator can reach.
 */
export type BannerArchetype = "LIMITED" | "SINGLE" | "DOUBLE" | "NORMAL" | "LINKAGE" | "CLASSIC" | "ATTAIN";

export function useBannerLabel(): (ruleType: BannerArchetype) => string {
    const t: PullsT = useT("tools");
    // Seven literal call sites rather than a table of key strings. The table reads
    // better and extracts as seven dead keys, because the extractor matches literal
    // t() arguments and cannot follow an index into a constant.
    const labels: Record<BannerArchetype, string> = {
        LIMITED: t("release.pulls.banner.LIMITED"),
        SINGLE: t("release.pulls.banner.SINGLE"),
        DOUBLE: t("release.pulls.banner.DOUBLE"),
        NORMAL: t("release.pulls.banner.NORMAL"),
        LINKAGE: t("release.pulls.banner.LINKAGE"),
        CLASSIC: t("release.pulls.banner.CLASSIC"),
        ATTAIN: t("release.pulls.banner.ATTAIN"),
    };
    return (ruleType) => labels[ruleType];
}

export function Stat({ label, value, sub, icon, hint, className }: { label: React.ReactNode; value: React.ReactNode; sub?: React.ReactNode; icon?: CurrencyName; hint?: React.ReactNode; className?: string }): React.ReactElement {
    return (
        <div className={cn("flex min-w-0 flex-col gap-0.5", className)}>
            <span className="inline-flex items-center gap-1.5 font-medium font-sans text-[11.5px] text-muted-foreground">
                {label}
                {hint}
            </span>
            <span className="inline-flex items-center gap-1.5 font-bold font-mono text-[18px] text-foreground tabular-nums leading-none">
                {icon && <CurrencyIcon name={icon} className="size-4" />}
                {value}
            </span>
            {sub && <span className="font-mono text-[12px] text-muted-foreground tabular-nums">{sub}</span>}
        </div>
    );
}

/** The plain-language reading of a banner's model, so a number is never unexplained. */
export function BannerModelNote({ model, className }: { model: IBannerModel; className?: string }): React.ReactElement {
    const t: PullsT = useT("tools");
    const lines: string[] = [t("release.pulls.rule.share", { percent: Math.round(model.shareTotal * 100), count: model.featuredCount })];

    // Both scope names are resolved unconditionally and then chosen between. A
    // ternary INSIDE the t() call would read fine and extract as a dead key, because
    // the extractor matches literal call sites and cannot see through a conditional.
    const standard = t("release.pulls.rule.scope.standard");
    const kernel = t("release.pulls.rule.scope.kernel");
    if (model.carryOver) lines.push(t("release.pulls.rule.carry", { scope: model.scope === "kernel" ? kernel : standard }));
    else lines.push(t("release.pulls.rule.isolated"));

    const g = model.guarantee;
    if (g.kind === "linkage" && g.at !== undefined) lines.push(t("release.pulls.rule.guarantee.linkage", { at: g.at }));
    if (g.kind === "selection" && g.first !== undefined && g.second !== undefined) lines.push(t("release.pulls.rule.guarantee.selection", { first: g.first, second: g.second }));
    if (g.kind === "attain") lines.push(t("release.pulls.rule.guarantee.attain"));
    if (model.spark !== null) lines.push(t("release.pulls.rule.spark", { count: model.spark }));

    return (
        <ul className={cn("m-0 flex list-none flex-col gap-0.5 p-0", HINT_TEXT, className)}>
            {lines.map((line) => (
                <li key={line}>{line}</li>
            ))}
            {model.inferred && <li className="text-amber-500/90">{t("release.pulls.banners.inferred")}</li>}
        </ul>
    );
}
