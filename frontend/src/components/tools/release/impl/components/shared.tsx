import { Link } from "@tanstack/react-router";
import { SearchX } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { Label } from "#/components/ui/label";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { Switch } from "#/components/ui/switch";
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

export function Tag({ children, className }: { children: React.ReactNode; className?: string }): React.ReactElement {
    return <span className={cn("inline-flex shrink-0 items-center rounded-sm bg-muted px-1.5 py-px font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.06em]", className)}>{children}</span>;
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
    return (
        <span className={cn("inline-flex shrink-0 cursor-help items-center rounded-sm border border-border/60 px-1 py-px font-sans text-[9.5px] text-muted-foreground/80 leading-[1.3]", className)} title={title} translate="no">
            {label}
        </span>
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
        return (
            <span className={cn("flex min-w-0 items-center gap-1", className)} title={tip || undefined}>
                {primary}
                {children}
            </span>
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

export function ToggleField({ id, label, checked, onChange }: { id: string; label: string; checked: boolean; onChange: (v: boolean) => void }): React.ReactElement {
    return (
        <Label htmlFor={id} className="cursor-pointer gap-2 font-sans text-[12.5px] text-muted-foreground">
            <Switch id={id} checked={checked} onCheckedChange={onChange} />
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

export function ReleaseError({ error, onRetry }: { error: unknown; onRetry: () => void }): React.ReactElement {
    const t: SharedT = useT("tools");
    const message = error instanceof Error ? error.message : String(error);
    return (
        <Card className="items-start gap-2 p-4">
            <p className="m-0 font-sans text-[13px] text-destructive-foreground">{message || t("release.loadFailed")}</p>
            <Button size="sm" variant="outline" onClick={onRetry}>
                {t("release.retry")}
            </Button>
        </Card>
    );
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
        <Link to="/operators/$id" params={{ id }} className="inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-secondary/40 py-0.5 pr-2 pl-0.5 font-medium font-sans text-[12px] text-foreground hover:bg-accent/50" title={entry.name}>
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
    return <img src={src} alt={alt} title={title} loading="lazy" onError={onError} className={cn("w-full rounded-md bg-muted object-cover", wide ? "aspect-[5/2] sm:w-60" : "aspect-video sm:w-40")} />;
}
