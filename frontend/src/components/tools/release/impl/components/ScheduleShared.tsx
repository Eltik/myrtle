import { X } from "lucide-react";
import * as React from "react";
import { itemIcon } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { FilterChip } from "#/components/ui/filter-chip";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { type TypedRichT, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { FarmStage } from "#/types/generated/FarmStage";
import { useAutoTranslate } from "../autoTranslate";
import { formatDateRange } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { useReleaseTagLabel } from "../labels";
import { overrideKey, scheduleOverrideTarget } from "../overrides";
import { type IScheduleItem, KIND_LABEL_KEYS, type ScheduleKind } from "../schedule";
import type { messages as scheduleMessages } from "../schedule.messages";
import { useReleaseOverrides } from "../useReleaseOverrides";
import { BadgeWithOverride, OverrideButton } from "./overrides/OverrideButton";
import { type IOverrideTarget, OverrideSheet } from "./overrides/OverrideSheet";
import { ResolutionBadge } from "./ResolutionBadge";
import type { messages } from "./ScheduleShared.messages";
import { SkinTileView } from "./SkinPopup";
import { CnName, type OperatorLookup, resolveName, Tag, ToggleField, useArt } from "./shared";

/** These parts render their own chrome plus the kind labels `schedule.ts` carries. */
type ScheduleSharedT = TypedT<typeof messages & typeof scheduleMessages & typeof helperMessages>;
type ScheduleSharedRichT = TypedRichT<typeof messages>;

const OCC_LABEL_KEYS: Record<string, keyof typeof messages & string> = {
    ALWAYS: "release.occ.always",
    ALMOST: "release.occ.almost",
    USUAL: "release.occ.usual",
    OFTEN: "release.occ.often",
    SOMETIMES: "release.occ.sometimes",
    RARELY: "release.occ.rarely",
};

function occLabel(occ: string, t: ScheduleSharedT): string {
    const key = OCC_LABEL_KEYS[occ];
    return key ? t(key) : occ.toLowerCase();
}

/**
 * Per-kind colour, in four roles.
 *
 * `pill` tints are 22-30% with a matching 1px INSET RING at 45-55%, so a pill carries its hue on
 * its own edge whether or not it drew a dot. At 12-15% a sky pill and a pink pill were both
 * essentially the page background, and the only full-strength hue was a 2px dot that is
 * suppressed on any segment continuing from the week before. Inset ring, not border: the
 * calendar lane is a fixed 22px (`my-0.5` plus `leading-[18px]`) and a border would add 2px to
 * every pill and overflow it.
 *
 * `swatch` is what the legend uses: the pill's own tint inside the pill's own border. A fully
 * opaque `dot` square beside the tinted pills is the same hue and visibly a different colour.
 *
 * `tone` is the hue as a CSS colour for the kind tag and filter chip, which tint with it and keep
 * foreground text.
 *
 * Pill text is `foreground`, not a per-hue 950 / 100 step: foreground measured 14.68 to 17.11
 * light and 9.08 to 11.94 dark on the resting tints over the card (5.57 lowest, yellow, on the
 * dark hover tint).
 */
/**
 * `ink` is the pressed filter-chip border where the plain tone misses 3:1 on the light page:
 * sky-500 2.60, yellow-500 1.83, teal-500 2.32. The inks are literal oklch (Tailwind sky-600,
 * yellow-700, teal-600) so they hold in both themes without a token: 3.85 / 5.01, 4.71 / 4.09,
 * 3.50 / 5.50 on the light / dark page. Pink-500 (3.43 / 5.62) and violet-500 (4.21 / 4.57)
 * clear it as they are.
 */
export const KIND_STYLE: Record<ScheduleKind, { pill: string; swatch: string; dot: string; tone: string; ink?: string }> = {
    event: {
        pill: "ring-1 ring-inset ring-sky-500/45 bg-sky-500/22 text-foreground hover:bg-sky-500/35 dark:ring-sky-400/45 dark:bg-sky-400/25 dark:hover:bg-sky-400/40",
        swatch: "border-sky-500/55 bg-sky-500/22 dark:border-sky-400/55 dark:bg-sky-400/25",
        dot: "bg-sky-500",
        tone: "var(--color-sky-500)",
        ink: "oklch(0.588 0.158 241.966)",
    },
    banner: {
        pill: "ring-1 ring-inset ring-yellow-500/50 bg-yellow-400/30 text-foreground hover:bg-yellow-400/45 dark:ring-yellow-300/45 dark:bg-yellow-300/25 dark:hover:bg-yellow-300/40",
        swatch: "border-yellow-500/60 bg-yellow-400/30 dark:border-yellow-300/55 dark:bg-yellow-300/25",
        dot: "bg-yellow-400",
        tone: "var(--color-yellow-500)",
        ink: "oklch(0.554 0.135 66.442)",
    },
    skin: {
        pill: "ring-1 ring-inset ring-pink-500/45 bg-pink-500/22 text-foreground hover:bg-pink-500/35 dark:ring-pink-400/45 dark:bg-pink-400/25 dark:hover:bg-pink-400/40",
        swatch: "border-pink-500/55 bg-pink-500/22 dark:border-pink-400/55 dark:bg-pink-400/25",
        dot: "bg-pink-500",
        tone: "var(--color-pink-500)",
    },
    rerun: {
        pill: "ring-1 ring-inset ring-violet-500/45 bg-violet-500/22 text-foreground hover:bg-violet-500/35 dark:ring-violet-400/45 dark:bg-violet-400/25 dark:hover:bg-violet-400/40",
        swatch: "border-violet-500/55 bg-violet-500/22 dark:border-violet-400/55 dark:bg-violet-400/25",
        dot: "bg-violet-500",
        tone: "var(--color-violet-500)",
    },
    review: {
        pill: "ring-1 ring-inset ring-teal-500/45 bg-teal-500/22 text-foreground hover:bg-teal-500/35 dark:ring-teal-400/45 dark:bg-teal-400/25 dark:hover:bg-teal-400/40",
        swatch: "border-teal-500/55 bg-teal-500/22 dark:border-teal-400/55 dark:bg-teal-400/25",
        dot: "bg-teal-500",
        tone: "var(--color-teal-500)",
        ink: "oklch(0.6 0.118 184.704)",
    },
};

export const ALL_KINDS: ScheduleKind[] = ["event", "banner", "skin", "rerun", "review"];

export function useItemName(item: IScheduleItem): string {
    const autoOn = useAutoTranslate();
    return resolveName(item.nameCn, item.nameEn, item.nameAuto, autoOn).text;
}

export function itemName(item: IScheduleItem, autoOn: boolean): string {
    return resolveName(item.nameCn, item.nameEn, item.nameAuto, autoOn).text;
}

interface IScheduleControlsProps {
    kinds: Set<ScheduleKind>;
    onKindsChange: (kinds: Set<ScheduleKind>) => void;
    stageOnly: boolean;
    onStageOnlyChange: (v: boolean) => void;
    counts: Record<ScheduleKind, number>;
}

export function ScheduleControls({ kinds, onKindsChange, stageOnly, onStageOnlyChange, counts }: IScheduleControlsProps): React.ReactElement {
    const t: ScheduleSharedT = useT("tools");
    const toggle = (k: ScheduleKind) => {
        const next = new Set(kinds);
        if (next.has(k)) next.delete(k);
        else next.add(k);
        onKindsChange(next);
    };
    return (
        <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
            {ALL_KINDS.map((k) => {
                const on = kinds.has(k);
                return (
                    <FilterChip
                        key={k}
                        active={on}
                        onSelect={() => toggle(k)}
                        tone={KIND_STYLE[k].tone}
                        toneInk={KIND_STYLE[k].ink}
                        count={counts[k]}
                        label={
                            <span className="inline-flex items-center gap-1.5">
                                <span aria-hidden="true" className={cn("size-3 rounded-sm border", KIND_STYLE[k].swatch)} />
                                {t(KIND_LABEL_KEYS[k])}
                            </span>
                        }
                    />
                );
            })}
            <ToggleField id="schedule-stage-only" label={t("release.controls.stageOnly")} checked={stageOnly} onChange={onStageOnlyChange} />
            {/* Not 11px `text-muted-foreground`: it explains the one mark on the calendar that
                separates a confirmed date from an estimated one, so it is not secondary text:
                14px, in the body foreground. */}
            <span className="font-sans text-[14px] text-foreground">{t("release.controls.dotLegend")}</span>
        </div>
    );
}

export function AnchorCaption({ anchor }: { anchor: EventAnchor }): React.ReactElement {
    const t: ScheduleSharedT = useT("tools");
    const rt: ScheduleSharedRichT = useRichT("tools");
    const autoOn = useAutoTranslate();
    const name = resolveName(anchor.nameCn, anchor.nameEn, anchor.nameEnAuto, autoOn);
    // biome-ignore lint/a11y/noNoninteractiveTabindex: focusable so a keyboard reader can bring up the explanation, as a hover does
    const trigger = <span tabIndex={0} />;
    return (
        <Tooltip>
            <TooltipTrigger render={trigger}>
                {rt("release.anchor.with", {
                    event: (
                        <span className="text-foreground" lang={name.untranslated ? "zh-CN" : undefined} translate={name.untranslated ? "yes" : undefined}>
                            {name.text}
                        </span>
                    ),
                })}
            </TooltipTrigger>
            <TooltipPopup className="max-w-xs">{t("release.anchor.title", { event: anchor.cnId })}</TooltipPopup>
        </Tooltip>
    );
}

interface IScheduleDetailProps {
    item: IScheduleItem;
    lookup: OperatorLookup;
    today: Date;
    onClose: () => void;
}

export function ScheduleDetail({ item, lookup, today, onClose }: IScheduleDetailProps): React.ReactElement {
    const t: ScheduleSharedT = useT("tools");
    const locale = useLocale();
    const tagLabel = useReleaseTagLabel();
    const art = useArt(item.imagePath);
    const name = useItemName(item);
    const style = KIND_STYLE[item.kind];
    const isSkin = item.kind === "skin" || item.kind === "rerun" || item.kind === "review";
    const cnLabel = item.kind === "rerun" ? t("release.detail.cnRelisted") : t("release.detail.cn");
    const showArt = art.src !== null && !isSkin;
    const { canEdit, overrides } = useReleaseOverrides();
    const [overrideTarget, setOverrideTarget] = React.useState<IOverrideTarget | null>(null);
    // Events, banners and Fashion Reviews map to one override row each; a skin group or rerun
    // stands for several skins, so it gets no pencil.
    const target = canEdit ? scheduleOverrideTarget(item.key) : null;
    const badge = <ResolutionBadge resolution={item.resolution} today={today} caption={item.anchor ? <AnchorCaption anchor={item.anchor} /> : undefined} />;
    return (
        <Card className="relative gap-0 p-3 sm:p-4">
            <Button variant="ghost" size="icon-sm" onClick={onClose} aria-label={t("release.detail.close")} className="absolute top-2 right-2 text-muted-foreground">
                <X aria-hidden="true" />
            </Button>
            <div className={cn("grid grid-cols-1 gap-x-4 gap-y-2 sm:items-start", showArt ? "sm:grid-cols-[200px_minmax(0,1fr)_minmax(240px,auto)]" : "sm:grid-cols-[minmax(0,1fr)_minmax(240px,auto)]")}>
                {showArt && <img src={art.src ?? undefined} alt={name} onError={art.onError} className="aspect-[5/2] w-full rounded-md bg-muted object-cover sm:w-50" />}
                <div className="flex min-w-0 flex-col gap-1.5 pr-8 sm:pr-0">
                    <CnName cn={item.nameCn} en={item.nameEn} auto={item.nameAuto} primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                        <Tag tone={style.tone}>{t(KIND_LABEL_KEYS[item.kind])}</Tag>
                        {!isSkin && <Tag>{tagLabel(item.tag)}</Tag>}
                    </CnName>
                    {item.cnStart > 0 && (
                        <div className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                            <span className="mr-1">{cnLabel}</span>
                            {formatDateRange(item.cnStart, item.cnEnd, locale, t)}
                        </div>
                    )}
                    {!isSkin && item.charIds.length > 0 && (
                        <div className="flex flex-wrap items-center gap-1">
                            {item.charIds.map((id) => {
                                const entry = lookup.get(id);
                                const label = entry ? entry.name : id;
                                return (
                                    <span key={id} className="inline-flex items-center gap-1 rounded-md border border-border bg-muted/40 py-0.5 pr-2 pl-0.5 font-sans text-[11.5px]" title={id}>
                                        <span className="size-5 overflow-hidden rounded-sm">
                                            <OperatorAvatar charId={id} name={label} server={entry ? undefined : "cn"} />
                                        </span>
                                        {label}
                                    </span>
                                );
                            })}
                        </div>
                    )}
                </div>
                {target ? <BadgeWithOverride badge={badge} button={<OverrideButton name={name} exists={overrides.has(overrideKey(target.kind, target.cnId))} onClick={() => setOverrideTarget({ mode: "row", ...target, name, resolution: item.resolution })} />} /> : badge}
            </div>
            {isSkin && item.skins && item.skins.length > 0 && (
                <div className="mt-3 flex gap-2 overflow-x-auto pb-1">
                    {item.skins.map((t) => (
                        <SkinTileView key={t.skinId} skinId={t.skinId} charId={t.charId} charName={t.charName} skinName={t.skinName} skinNameEn={t.skinNameEn} skinNameAuto={t.skinNameAuto} portraitPath={t.portraitPath} lookup={lookup} />
                    ))}
                </div>
            )}
            {canEdit && <OverrideSheet target={overrideTarget} onClose={() => setOverrideTarget(null)} />}
        </Card>
    );
}

export function FarmStages({ stages, compact = false }: { stages: FarmStage[]; compact?: boolean }): React.ReactElement | null {
    const t: ScheduleSharedT = useT("tools");
    if (stages.length === 0) return null;
    return (
        <div className={cn("flex flex-wrap", compact ? "gap-1.5" : "gap-2")}>
            {stages.map((st) => (
                <div key={st.stageId} className={cn("flex items-center gap-2 rounded-md border border-border/60 bg-muted/30", compact ? "px-1.5 py-1" : "px-2 py-1.5")}>
                    <span className="flex flex-col leading-tight">
                        <span className="font-mono font-semibold text-[11.5px] text-foreground">{st.code}</span>
                        <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{t("release.farm.ap", { ap: st.apCost })}</span>
                    </span>
                    {st.drops.map((d) => (
                        <Tooltip key={d.itemId}>
                            <TooltipTrigger
                                render={
                                    compact ? (
                                        // biome-ignore lint/a11y/noNoninteractiveTabindex: compact rows show only the icon, so the item and its rate are reachable by keyboard here
                                        <span role="img" tabIndex={0} aria-label={t("release.farm.dropTitle", { item: d.nameEn ?? d.name, rate: occLabel(d.occ, t) })} className="flex items-center gap-1" />
                                    ) : (
                                        <span className="flex items-center gap-1" />
                                    )
                                }
                            >
                                <img src={itemIcon(d.itemId, d.iconId, null, d.nameEn ? undefined : "cn")} alt="" loading="lazy" className={cn("rounded-full bg-black/40 object-contain", compact ? "size-7" : "size-9")} />
                                {!compact && (
                                    <span className="flex flex-col leading-tight">
                                        <span className="max-w-36 truncate font-sans text-[11.5px] text-foreground" lang={d.nameEn ? undefined : "zh-CN"} translate={d.nameEn ? undefined : "yes"}>
                                            {d.nameEn ?? d.name}
                                        </span>
                                        <span className="font-sans text-[10px] text-muted-foreground">{occLabel(d.occ, t)}</span>
                                    </span>
                                )}
                            </TooltipTrigger>
                            <TooltipPopup>{t("release.farm.dropTitle", { item: d.nameEn ?? d.name, rate: occLabel(d.occ, t) })}</TooltipPopup>
                        </Tooltip>
                    ))}
                </div>
            ))}
        </div>
    );
}
