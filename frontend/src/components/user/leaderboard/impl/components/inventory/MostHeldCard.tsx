import { ItemIcon } from "#/components/user/profile/impl/components/tabs/Items/ItemIcon";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { toIconEntry } from "../../inventory.helpers";
import type { ICatalogItem } from "../../inventory.types";
import type { messages } from "./MostHeldCard.messages";

interface IMostHeldCardProps {
    catalog: ICatalogItem[];
    current: string;
    onItem: (next: string) => void;
    isLoading?: boolean;
}

const ROWS = 10;

export function MostHeldCard({ catalog, current, onItem, isLoading }: IMostHeldCardProps) {
    const t: TypedT<typeof messages> = useT("user");
    const f = useFormatters();
    const rows = catalog.slice(0, ROWS);

    return (
        <aside className="rounded-xl border border-border bg-card p-4 shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
            <div className="mb-3 flex flex-col gap-1">
                <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase leading-none tracking-[0.16em]">{t("leaderboard.mostHeld.title")}</span>
                <span className="font-sans text-[12px] text-muted-foreground leading-snug">{t("leaderboard.mostHeld.subtitle")}</span>
            </div>
            {isLoading && rows.length === 0 ? (
                <div className="flex flex-col gap-1.5">
                    {Array.from({ length: 6 }, (_, i) => `most-held-skeleton-${i}`).map((key) => (
                        <div key={key} className="flex items-center gap-2.5 px-1 py-1.5">
                            <span className="size-7 rounded-md bg-muted" />
                            <span className="h-3 flex-1 rounded bg-muted" />
                            <span className="h-3 w-14 rounded bg-muted/70" />
                        </div>
                    ))}
                </div>
            ) : (
                <ul className="m-0 flex list-none flex-col gap-0.5 p-0">
                    {rows.map((c) => {
                        const active = c.item_id === current;
                        return (
                            <li key={c.item_id}>
                                <button type="button" onClick={() => onItem(c.item_id)} aria-pressed={active} className={cn("flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-1.5 py-1.5 text-left transition-colors hover:bg-accent", active && "bg-[color-mix(in_srgb,var(--primary)_8%,transparent)]")}>
                                    <ItemIcon item={toIconEntry(c)} size={28} className="rounded-md" />
                                    <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                                        <span className={cn("truncate font-sans text-[13px] text-foreground leading-tight", active && "font-semibold text-primary")}>{c.name}</span>
                                        <span className="truncate font-mono text-[10.5px] text-muted-foreground leading-none">{t("leaderboard.mostHeld.top", { top: f.compact(c.top) })}</span>
                                    </span>
                                    <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums leading-none">{t("leaderboard.mostHeld.holders", { count: c.holders })}</span>
                                </button>
                            </li>
                        );
                    })}
                </ul>
            )}
        </aside>
    );
}
