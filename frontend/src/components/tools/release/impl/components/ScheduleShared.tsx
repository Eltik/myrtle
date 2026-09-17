import { X } from "lucide-react";
import type * as React from "react";
import { itemIcon } from "#/components/operators/detail/impl/assets";
import { Card } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { type TypedRichT, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import type { FarmStage } from "#/types/generated/FarmStage";
import { useAutoTranslate } from "../autoTranslate";
import { formatDateRange } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import { useReleaseTagLabel } from "../labels";
import { type IScheduleItem, KIND_LABEL_KEYS, type ScheduleKind } from "../schedule";
import type { messages as scheduleMessages } from "../schedule.messages";
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
 * Per-kind colour, in three roles.
 *
 * `pill` was 12-15% alpha, which is where "the calendar colours are too dim"
 * came from: at that strength a sky pill and a pink pill are both essentially
 * the page background, and the only full-strength reference to the hue was a
 * 2px dot that is suppressed on any segment continuing from the week before.
 * The tints are now 22-30% with a matching 1px INSET RING at 45-55%, so a pill
 * carries its hue on its own edge whether or not it drew a dot. Inset ring, not
 * border: the calendar lane is a fixed 22px (`my-0.5` plus `leading-[18px]`)
 * and a border would add 2px to every pill and overflow it.
 *
 * `swatch` is NEW and is what the legend uses. The legend used to paint `dot`,
 * a fully opaque square, next to pills tinted at 12%: technically the same hue,
 * visibly a different colour, which is the mismatch that was reported. The
 * legend now paints the pill's own tint inside the pill's own border.
 */
export const KIND_STYLE: Record<ScheduleKind, { pill: string; swatch: string; dot: string; text: string }> = {
    event: {
        pill: "ring-1 ring-inset ring-sky-500/45 bg-sky-500/22 text-sky-950 hover:bg-sky-500/35 dark:ring-sky-400/45 dark:bg-sky-400/25 dark:text-sky-100 dark:hover:bg-sky-400/40",
        swatch: "border-sky-500/55 bg-sky-500/22 dark:border-sky-400/55 dark:bg-sky-400/25",
        dot: "bg-sky-500",
        text: "text-sky-600 dark:text-sky-400",
    },
    banner: {
        pill: "ring-1 ring-inset ring-yellow-500/50 bg-yellow-400/30 text-yellow-950 hover:bg-yellow-400/45 dark:ring-yellow-300/45 dark:bg-yellow-300/25 dark:text-yellow-100 dark:hover:bg-yellow-300/40",
        swatch: "border-yellow-500/60 bg-yellow-400/30 dark:border-yellow-300/55 dark:bg-yellow-300/25",
        dot: "bg-yellow-400",
        text: "text-yellow-600 dark:text-yellow-300",
    },
    skin: {
        pill: "ring-1 ring-inset ring-pink-500/45 bg-pink-500/22 text-pink-950 hover:bg-pink-500/35 dark:ring-pink-400/45 dark:bg-pink-400/25 dark:text-pink-100 dark:hover:bg-pink-400/40",
        swatch: "border-pink-500/55 bg-pink-500/22 dark:border-pink-400/55 dark:bg-pink-400/25",
        dot: "bg-pink-500",
        text: "text-pink-600 dark:text-pink-400",
    },
    rerun: {
        pill: "ring-1 ring-inset ring-violet-500/45 bg-violet-500/22 text-violet-950 hover:bg-violet-500/35 dark:ring-violet-400/45 dark:bg-violet-400/25 dark:text-violet-100 dark:hover:bg-violet-400/40",
        swatch: "border-violet-500/55 bg-violet-500/22 dark:border-violet-400/55 dark:bg-violet-400/25",
        dot: "bg-violet-500",
        text: "text-violet-600 dark:text-violet-400",
    },
    review: {
        pill: "ring-1 ring-inset ring-teal-500/45 bg-teal-500/22 text-teal-950 hover:bg-teal-500/35 dark:ring-teal-400/45 dark:bg-teal-400/25 dark:text-teal-100 dark:hover:bg-teal-400/40",
        swatch: "border-teal-500/55 bg-teal-500/22 dark:border-teal-400/55 dark:bg-teal-400/25",
        dot: "bg-teal-500",
        text: "text-teal-600 dark:text-teal-400",
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
                    <button key={k} type="button" aria-pressed={on} onClick={() => toggle(k)} className={cn("inline-flex cursor-pointer items-center gap-1.5 font-sans text-[12.5px] transition-colors", on ? "text-foreground" : "text-muted-foreground")}>
                        <span className={cn("size-3.5 rounded-sm border transition-colors", on ? KIND_STYLE[k].swatch : "border-muted-foreground/50")} />
                        {t(KIND_LABEL_KEYS[k])}
                        <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{counts[k]}</span>
                    </button>
                );
            })}
            <ToggleField id="schedule-stage-only" label={t("release.controls.stageOnly")} checked={stageOnly} onChange={onStageOnlyChange} />
            {/* Was 11px `text-muted-foreground`. It explains the one mark on the
                calendar that separates a confirmed date from an estimated one, so
                it is not secondary text: 14px, at the body foreground's 80%. */}
            <span className="font-sans text-[14px] text-foreground/80">{t("release.controls.dotLegend")}</span>
        </div>
    );
}

export function AnchorCaption({ anchor }: { anchor: EventAnchor }): React.ReactElement {
    const t: ScheduleSharedT = useT("tools");
    const rt: ScheduleSharedRichT = useRichT("tools");
    const autoOn = useAutoTranslate();
    const name = resolveName(anchor.nameCn, anchor.nameEn, anchor.nameEnAuto, autoOn);
    return (
        <span title={t("release.anchor.title", { event: anchor.cnId })}>
            {rt("release.anchor.with", {
                event: (
                    <span className="text-foreground/80" lang={name.untranslated ? "zh-CN" : undefined} translate={name.untranslated ? "yes" : undefined}>
                        {name.text}
                    </span>
                ),
            })}
        </span>
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
    return (
        <Card className="relative gap-0 overflow-hidden p-3 pl-4 sm:p-4 sm:pl-5">
            <span aria-hidden="true" className={cn("absolute inset-y-0 left-0 w-1", style.dot)} />
            <button type="button" onClick={onClose} aria-label={t("release.detail.close")} className="absolute top-2 right-2 flex size-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground">
                <X className="size-4" />
            </button>
            <div className={cn("grid grid-cols-1 gap-x-4 gap-y-2 sm:items-start", showArt ? "sm:grid-cols-[200px_minmax(0,1fr)_minmax(240px,auto)]" : "sm:grid-cols-[minmax(0,1fr)_minmax(240px,auto)]")}>
                {showArt && <img src={art.src ?? undefined} alt={name} onError={art.onError} className="aspect-[5/2] w-full rounded-md bg-muted object-cover sm:w-50" />}
                <div className="flex min-w-0 flex-col gap-1.5 pr-8 sm:pr-0">
                    <CnName cn={item.nameCn} en={item.nameEn} auto={item.nameAuto} primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                        <Tag className={style.text}>{t(KIND_LABEL_KEYS[item.kind])}</Tag>
                        {!isSkin && <Tag>{tagLabel(item.tag)}</Tag>}
                    </CnName>
                    {item.cnStart > 0 && (
                        <div className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                            <span className="mr-1 uppercase tracking-[0.06em]">{cnLabel}</span>
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
                <ResolutionBadge resolution={item.resolution} today={today} caption={item.anchor ? <AnchorCaption anchor={item.anchor} /> : undefined} />
            </div>
            {isSkin && item.skins && item.skins.length > 0 && (
                <div className="mt-3 flex gap-2 overflow-x-auto pb-1">
                    {item.skins.map((t) => (
                        <SkinTileView key={t.skinId} skinId={t.skinId} charId={t.charId} charName={t.charName} skinName={t.skinName} skinNameEn={t.skinNameEn} skinNameAuto={t.skinNameAuto} portraitPath={t.portraitPath} lookup={lookup} />
                    ))}
                </div>
            )}
        </Card>
    );
}

export function FarmStages({ stages, compact = false }: { stages: FarmStage[]; compact?: boolean }): React.ReactElement | null {
    const t: ScheduleSharedT = useT("tools");
    if (stages.length === 0) return null;
    return (
        <div className={cn("flex flex-wrap", compact ? "gap-1.5" : "gap-2")}>
            {stages.map((st) => (
                <div key={st.stageId} className={cn("flex items-center gap-2 rounded-md border border-border/60 bg-muted/30", compact ? "px-1.5 py-1" : "px-2 py-1.5")} title={t("release.farm.stageTitle", { code: st.code, ap: st.apCost })}>
                    <span className="flex flex-col leading-tight">
                        <span className="font-mono font-semibold text-[11.5px] text-foreground">{st.code}</span>
                        <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{t("release.farm.ap", { ap: st.apCost })}</span>
                    </span>
                    {st.drops.map((d) => (
                        <span key={d.itemId} className="flex items-center gap-1" title={t("release.farm.dropTitle", { item: d.nameEn ?? d.name, rate: occLabel(d.occ, t) })}>
                            <img src={itemIcon(d.itemId, d.iconId, null, d.nameEn ? undefined : "cn")} alt="" loading="lazy" className={cn("rounded-full bg-black/40 object-contain", compact ? "size-7" : "size-9")} />
                            {!compact && (
                                <span className="flex flex-col leading-tight">
                                    <span className="max-w-36 truncate font-sans text-[11.5px] text-foreground" lang={d.nameEn ? undefined : "zh-CN"} translate={d.nameEn ? undefined : "yes"}>
                                        {d.nameEn ?? d.name}
                                    </span>
                                    <span className="font-sans text-[10px] text-muted-foreground">{occLabel(d.occ, t)}</span>
                                </span>
                            )}
                        </span>
                    ))}
                </div>
            ))}
        </div>
    );
}
