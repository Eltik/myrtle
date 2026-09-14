import { X } from "lucide-react";
import type * as React from "react";
import { Card } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { cn } from "#/lib/utils";
import type { EventAnchor } from "#/types/generated/EventAnchor";
import { useAutoTranslate } from "../autoTranslate";
import { formatDateRange, humanizeTag } from "../helpers";
import { type IScheduleItem, KIND_LABEL, type ScheduleKind } from "../schedule";
import { ResolutionBadge } from "./ResolutionBadge";
import { SkinTileView } from "./SkinPopup";
import { CnName, type OperatorLookup, resolveName, Tag, ToggleField, useArt } from "./shared";

export const KIND_STYLE: Record<ScheduleKind, { solid: string; estimated: string; dot: string; text: string }> = {
    event: { solid: "bg-sky-600 text-white", estimated: "border border-sky-500/70 border-dashed bg-sky-500/25 text-sky-100", dot: "bg-sky-500", text: "text-sky-400" },
    banner: { solid: "bg-amber-600 text-white", estimated: "border border-amber-500/70 border-dashed bg-amber-500/25 text-amber-100", dot: "bg-amber-500", text: "text-amber-400" },
    skin: { solid: "bg-fuchsia-600 text-white", estimated: "border border-fuchsia-500/70 border-dashed bg-fuchsia-500/25 text-fuchsia-100", dot: "bg-fuchsia-500", text: "text-fuchsia-400" },
    rerun: { solid: "bg-violet-600 text-white", estimated: "border border-violet-500/70 border-dashed bg-violet-500/25 text-violet-100", dot: "bg-violet-500", text: "text-violet-400" },
};

export const ALL_KINDS: ScheduleKind[] = ["event", "banner", "skin", "rerun"];

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
    children?: React.ReactNode;
}

export function ScheduleControls({ kinds, onKindsChange, stageOnly, onStageOnlyChange, counts, children }: IScheduleControlsProps): React.ReactElement {
    const toggle = (k: ScheduleKind) => {
        const next = new Set(kinds);
        if (next.has(k)) next.delete(k);
        else next.add(k);
        onKindsChange(next);
    };
    return (
        <div className="flex flex-wrap items-center gap-x-5 gap-y-2">
            <div className="flex flex-wrap items-center gap-1.5">
                {ALL_KINDS.map((k) => {
                    const on = kinds.has(k);
                    return (
                        <button
                            key={k}
                            type="button"
                            onClick={() => toggle(k)}
                            aria-pressed={on}
                            className={cn("inline-flex cursor-pointer items-center gap-1.5 rounded-md border px-2 py-1 font-medium font-sans text-[12px] transition-colors", on ? "border-border bg-muted text-foreground" : "border-transparent text-muted-foreground hover:text-foreground")}
                        >
                            <span className={cn("size-2 rounded-full", KIND_STYLE[k].dot, !on && "opacity-40")} />
                            {KIND_LABEL[k]}
                            <span className="font-mono text-[10.5px] text-muted-foreground tabular-nums">{counts[k]}</span>
                        </button>
                    );
                })}
            </div>
            <ToggleField id="schedule-stage-only" label="Stage events only" checked={stageOnly} onChange={onStageOnlyChange} />
            {children}
            <span className="font-sans text-[11px] text-muted-foreground">Solid: confirmed or announced. Dashed: estimated; select a bar for its band and details.</span>
        </div>
    );
}

export function AnchorCaption({ anchor }: { anchor: EventAnchor }): React.ReactElement {
    const autoOn = useAutoTranslate();
    const name = resolveName(anchor.nameCn, anchor.nameEn, anchor.nameEnAuto, autoOn);
    return (
        <span title={`Released on CN the day ${anchor.cnId} started; 96% of such skins reach EN with their event`}>
            with{" "}
            <span className="text-foreground/80" lang={name.untranslated ? "zh-CN" : undefined} translate={name.untranslated ? "yes" : undefined}>
                {name.text}
            </span>
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
    const art = useArt(item.imagePath);
    const name = useItemName(item);
    const style = KIND_STYLE[item.kind];
    const isSkin = item.kind === "skin" || item.kind === "rerun";
    const cnLabel = item.kind === "rerun" ? "CN re-listed" : "CN";
    const showArt = art.src !== null && !isSkin;
    return (
        <Card className="relative gap-0 overflow-hidden p-3 pl-4 sm:p-4 sm:pl-5">
            <span aria-hidden="true" className={cn("absolute inset-y-0 left-0 w-1", style.dot)} />
            <button type="button" onClick={onClose} aria-label="Close" className="absolute top-2 right-2 flex size-7 cursor-pointer items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-foreground">
                <X className="size-4" />
            </button>
            <div className={cn("grid grid-cols-1 gap-x-4 gap-y-2 sm:items-start", showArt ? "sm:grid-cols-[200px_minmax(0,1fr)_minmax(240px,auto)]" : "sm:grid-cols-[minmax(0,1fr)_minmax(240px,auto)]")}>
                {showArt && <img src={art.src ?? undefined} alt={name} onError={art.onError} className="aspect-[5/2] w-full rounded-md bg-muted object-cover sm:w-50" />}
                <div className="flex min-w-0 flex-col gap-1.5 pr-8 sm:pr-0">
                    <CnName cn={item.nameCn} en={item.nameEn} auto={item.nameAuto} primaryClassName="font-sans font-semibold text-[14px] text-foreground">
                        <Tag className={style.text}>{KIND_LABEL[item.kind]}</Tag>
                        {!isSkin && <Tag>{humanizeTag(item.tag)}</Tag>}
                    </CnName>
                    {item.cnStart > 0 && (
                        <div className="font-mono text-[11.5px] text-muted-foreground tabular-nums">
                            <span className="mr-1 uppercase tracking-[0.06em]">{cnLabel}</span>
                            {formatDateRange(item.cnStart, item.cnEnd)}
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
