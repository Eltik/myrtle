import { Link } from "@tanstack/react-router";
import { SearchX } from "lucide-react";
import * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Card } from "#/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Label } from "#/components/ui/label";
import { LoadFailed } from "#/components/ui/load-failed";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, parseOperatorName, rarityToNumber } from "#/lib/utils";
import type { AutoName } from "#/types/generated/AutoName";
import type { AutoNameSource } from "#/types/generated/AutoNameSource";
import type { IOperatorIndexEntry } from "#/types/operators";
import { useAutoTranslate } from "../autoTranslate";
import { assetUrl } from "../helpers";
import type { messages } from "./shared.messages";

type SharedT = TypedT<typeof messages>;

/**
 * A kind or type tag. With `tone` (a CSS colour) the hue tints the fill and border and the text
 * stays foreground: the kind colours as text on the muted tag measured 2.46 (yellow-600) to 3.81
 * (pink-600). Without a tone it is a plain outline tag.
 */
export function Tag({ children, className, tone }: { children: React.ReactNode; className?: string; tone?: string }): React.ReactElement {
    return (
        <Badge variant={tone ? "tint" : "outline"} size="sm" className={cn("font-mono", className)} style={tone ? ({ "--tone": tone } as React.CSSProperties) : undefined}>
            {children}
        </Badge>
    );
}

const AUTO_TAG_KEYS: Record<AutoNameSource, { label: keyof typeof messages & string; title: keyof typeof messages & string }> = {
    memory: { label: "release.auto.memory", title: "release.auto.memory.title" },
    appellation: { label: "release.auto.appellation", title: "release.auto.appellation.title" },
    override: { label: "release.auto.override", title: "release.auto.override.title" },
};

export function AutoTag({ source, className }: { source: AutoNameSource; className?: string }): React.ReactElement {
    const t: SharedT = useT("tools");
    const label = t(AUTO_TAG_KEYS[source].label);
    const title = t(AUTO_TAG_KEYS[source].title);
    // biome-ignore lint/a11y/noNoninteractiveTabindex: focusable so a keyboard reader can bring up the explanation, as a hover does
    const trigger = <span tabIndex={0} className={cn("inline-flex shrink-0 cursor-help items-center rounded-sm border border-border/60 px-1 py-px font-sans text-[9.5px] text-subtle-foreground leading-[1.3]", className)} translate="no" />;
    return (
        <Tooltip>
            <TooltipTrigger render={trigger}>{label}</TooltipTrigger>
            <TooltipPopup className="max-w-xs">{title}</TooltipPopup>
        </Tooltip>
    );
}

export interface IResolvedName {
    text: string;
    auto: AutoName | null;
    untranslated: boolean;
    original: string | null;
}

export function resolveName(cnText: string, en: string | null | undefined, auto: AutoName | null | undefined, autoOn: boolean): IResolvedName {
    if (en?.trim()) return { text: en, auto: null, untranslated: false, original: cnText && cnText !== en ? cnText : null };
    if (autoOn && auto?.text.trim()) return { text: auto.text, auto, untranslated: false, original: cnText && cnText !== auto.text ? cnText : null };
    return { text: cnText, auto: null, untranslated: autoOn, original: null };
}

interface ICnNameProps {
    cn: string;
    en?: string | null;
    auto?: AutoName | null;
    className?: string;
    primaryClassName?: string;
    compact?: boolean;
    children?: React.ReactNode;
}

export function CnName({ cn: cnText, en, auto, className, primaryClassName, compact, children }: ICnNameProps): React.ReactElement {
    const t: SharedT = useT("tools");
    const autoOn = useAutoTranslate();
    const name = resolveName(cnText, en, auto, autoOn);
    const primary = (
        <span className={cn("min-w-0 truncate", primaryClassName)} lang={name.untranslated ? "zh-CN" : undefined} translate={name.untranslated ? "yes" : undefined}>
            {name.text}
        </span>
    );
    if (compact) {
        const tip = [name.original, name.auto ? t(AUTO_TAG_KEYS[name.auto.source].label) : null].filter(Boolean).join(" \u00b7 ");
        const row = (
            <span className={cn("flex min-w-0 items-center gap-1", className)}>
                {primary}
                {tip && <span className="sr-only">{` (${tip})`}</span>}
                {children}
            </span>
        );
        if (!tip) return row;
        return (
            <Tooltip>
                <TooltipTrigger render={row} />
                <TooltipPopup>{tip}</TooltipPopup>
            </Tooltip>
        );
    }
    return (
        <span className={cn("flex min-w-0 flex-col", className)}>
            <span className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
                {primary}
                {name.auto && <AutoTag source={name.auto.source} />}
                {children}
            </span>
            {name.original && (
                <span className="truncate font-sans text-[12px] text-muted-foreground" lang="zh-CN">
                    {name.original}
                </span>
            )}
        </span>
    );
}

export function useArt(path: string | null | undefined): { src: string | null; onError: () => void } {
    const [failed, setFailed] = React.useState(false);
    const onError = React.useCallback(() => setFailed(true), []);
    return { src: failed ? null : assetUrl(path ?? null), onError };
}

export function ToggleField({ id, label, ariaLabel, checked, onChange }: { id: string; label: React.ReactNode; ariaLabel?: string; checked: boolean; onChange: (v: boolean) => void }): React.ReactElement {
    return (
        <Label htmlFor={id} className="cursor-pointer gap-2 font-medium font-sans text-[12.5px] text-muted-foreground">
            <Switch id={id} checked={checked} onCheckedChange={onChange} aria-label={ariaLabel} />
            {label}
        </Label>
    );
}

export function SectionTitle({ children, count }: { children: React.ReactNode; count?: number }): React.ReactElement {
    const t: SharedT = useT("tools");
    return (
        <h3 className="m-0 mb-2 flex items-baseline gap-2 font-sans font-semibold text-[15px] text-foreground">
            {children}
            {count !== undefined && <span className="font-medium font-mono text-[11px] text-muted-foreground">{t("release.sectionCount", { count })}</span>}
        </h3>
    );
}

export function ReleaseEmpty({ title, description }: { title: string; description: string }): React.ReactElement {
    return (
        <Card>
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <SearchX />
                    </EmptyMedia>
                    <EmptyTitle>{title}</EmptyTitle>
                    <EmptyDescription>{description}</EmptyDescription>
                </EmptyHeader>
            </Empty>
        </Card>
    );
}

export function ReleaseLoading(): React.ReactElement {
    return (
        <Card className="gap-3 p-4">
            {Array.from({ length: 6 }, (_, i) => i).map((i) => (
                <div key={i} className="grid grid-cols-[1fr_auto] items-center gap-4">
                    <div className="flex flex-col gap-1.5">
                        <Skeleton className="h-3.5 w-[min(320px,60%)]" />
                        <Skeleton className="h-3 w-[min(200px,40%)]" />
                    </div>
                    <Skeleton className="h-4 w-40" />
                </div>
            ))}
        </Card>
    );
}

/** A release tab's failed fetch: the kit shape with the release copy, never the server's text. */
export function ReleaseError({ onRetry }: { onRetry: () => void }): React.ReactElement {
    const t: SharedT = useT("tools");
    return <LoadFailed message={t("release.loadFailed")} onRetry={onRetry} />;
}

export type OperatorLookup = Map<string, IOperatorIndexEntry>;

export function buildOperatorLookup(entries: IOperatorIndexEntry[] | undefined): OperatorLookup {
    const map: OperatorLookup = new Map();
    for (const e of entries ?? []) map.set(e.id, e);
    return map;
}

export function operatorLabel(id: string, entry: IOperatorIndexEntry | undefined, name: AutoName | null | undefined, autoOn: boolean): { text: string; auto: AutoName | null } {
    if (entry) return { text: parseOperatorName(entry.name).displayName, auto: null };
    if (autoOn && name?.text.trim()) return { text: name.text, auto: name };
    return { text: id, auto: null };
}

export function OpRef({ id, lookup, name }: { id: string; lookup: OperatorLookup; name?: AutoName | null }): React.ReactElement {
    const autoOn = useAutoTranslate();
    const entry = lookup.get(id);
    if (!entry) {
        const label = operatorLabel(id, undefined, name, autoOn);
        return (
            <span className={cn("inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-secondary/40 py-0.5 pr-2 pl-0.5 text-muted-foreground", label.auto ? "font-medium font-sans text-[12px]" : "font-mono text-[11.5px]")} title={id}>
                <span className="inline-flex size-5 shrink-0 items-center justify-center overflow-hidden rounded bg-muted font-bold font-sans text-[10px] text-muted-foreground">
                    <OperatorAvatar charId={id} name={label.text} server="cn" />
                </span>
                {label.text}
                {label.auto && <AutoTag source={label.auto.source} />}
            </span>
        );
    }
    const { displayName } = parseOperatorName(entry.name);
    const rarity = rarityToNumber(entry.rarity);
    return (
        <Link
            to="/operators/$id"
            params={{ id }}
            className="relative inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-secondary/40 py-0.5 pr-2 pl-0.5 font-medium font-sans text-[12px] text-foreground pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:bg-accent/50"
            title={entry.name}
        >
            <span className="inline-flex size-5 shrink-0 items-center justify-center overflow-hidden rounded font-bold font-sans text-[10px] text-white" style={{ backgroundColor: `var(--rarity-${rarity})` }}>
                <OperatorAvatar charId={id} name={entry.name} />
            </span>
            {displayName}
            <span className="font-mono text-[10px] text-muted-foreground">{rarity}★</span>
        </Link>
    );
}

export function OpRefList({ ids, lookup, names }: { ids: string[]; lookup: OperatorLookup; names?: { [key in string]?: AutoName } }): React.ReactElement | null {
    if (ids.length === 0) return null;
    return (
        <span className="flex flex-wrap gap-1">
            {ids.map((id) => (
                <OpRef key={id} id={id} lookup={lookup} name={names?.[id] ?? null} />
            ))}
        </span>
    );
}

interface IListRowProps {
    visual: React.ReactNode;
    wide?: boolean;
    badge: React.ReactNode;
    children: React.ReactNode;
}

export function ListRow({ visual, wide = false, badge, children }: IListRowProps): React.ReactElement {
    return (
        <div
            className={cn(
                "grid gap-x-3 gap-y-1.5 border-border border-t py-2.5 first:border-t-0 sm:items-start sm:gap-x-4",
                visual ? (wide ? "grid-cols-[128px_minmax(0,1fr)] sm:grid-cols-[240px_minmax(0,1fr)_minmax(240px,auto)]" : "grid-cols-[112px_minmax(0,1fr)] sm:grid-cols-[160px_minmax(0,1fr)_minmax(240px,auto)]") : "grid-cols-1 sm:grid-cols-[minmax(0,1fr)_minmax(240px,auto)]",
            )}
        >
            {visual}
            <div className="flex min-w-0 flex-col gap-1.5">{children}</div>
            <div className="col-span-full sm:col-span-1">{badge}</div>
        </div>
    );
}

export function RowImage({ src, alt, title, onError, wide = false }: { src: string; alt: string; title?: string; onError: () => void; wide?: boolean }): React.ReactElement {
    // The tooltip names where borrowed art came from; the alt says it too, since a hover is the only way to the tooltip.
    const img = <img src={src} alt={title ?? alt} loading="lazy" onError={onError} className={cn("w-full rounded-md bg-muted object-cover", wide ? "aspect-[5/2] sm:w-60" : "aspect-video sm:w-40")} />;
    if (!title) return img;
    return (
        <Tooltip>
            <TooltipTrigger render={img} />
            <TooltipPopup>{title}</TooltipPopup>
        </Tooltip>
    );
}
