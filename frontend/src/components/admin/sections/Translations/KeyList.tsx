import { useEffect, useRef } from "react";
import { skeletons } from "#/components/admin/Primitives";
import { Button } from "#/components/ui/button";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { TranslationEntry } from "#/types/generated/TranslationEntry";
import { entryStatus } from "./model";
import { StatusBadge } from "./StatusBadge";
import type { messages } from "./Translations.messages";

type T = TypedT<typeof messages>;

interface IKeyListProps {
    entries: TranslationEntry[];
    selectedKey: string | undefined;
    pending: boolean;
    error: boolean;
    onRetry: () => void;
    emptyText: string;
    hasMore: boolean;
    loadingMore: boolean;
    onLoadMore: () => void;
    total: number;
    onSelect: (key: string) => void;
}

export function KeyList({ entries, selectedKey, pending, error, onRetry, emptyText, hasMore, loadingMore, onLoadMore, total, onSelect }: IKeyListProps): React.ReactElement {
    const t: T = useT("admin");
    const fmt = useFormatters();
    const scrollRef = useRef<HTMLDivElement | null>(null);
    const sentinelRef = useRef<HTMLDivElement | null>(null);

    // Save & next moves the selection without a click: keep the row in view inside the list, never scrolling the page.
    useEffect(() => {
        const box = scrollRef.current;
        if (!box || !selectedKey) return;
        const row = box.querySelector<HTMLElement>(`[data-key="${CSS.escape(selectedKey)}"]`);
        if (!row) return;
        const top = row.offsetTop - box.offsetTop;
        if (top < box.scrollTop) box.scrollTop = top - 6;
        else if (top + row.offsetHeight > box.scrollTop + box.clientHeight) box.scrollTop = top + row.offsetHeight - box.clientHeight + 6;
    }, [selectedKey]);

    // Scrolling to the bottom loads the next page; the button below is the fallback.
    useEffect(() => {
        const box = scrollRef.current;
        const sentinel = sentinelRef.current;
        if (!box || !sentinel || !hasMore) return;
        const observer = new IntersectionObserver(
            (items) => {
                if (items.some((i) => i.isIntersecting) && !loadingMore) onLoadMore();
            },
            { root: box, rootMargin: "120px" },
        );
        observer.observe(sentinel);
        return () => observer.disconnect();
    }, [hasMore, loadingMore, onLoadMore]);

    return (
        <div ref={scrollRef} className="relative h-[min(440px,60dvh)] overflow-auto p-1.5 lg:h-auto lg:min-h-0 lg:flex-1">
            {pending ? (
                <div className="flex flex-col gap-1.5 p-1">{skeletons(8, "h-11")}</div>
            ) : error ? (
                <div className="flex flex-col items-start gap-2 px-2.5 py-6 text-[13.5px] text-muted-foreground">
                    {t("translations.list.error")}
                    <Button size="xs" variant="outline" onClick={onRetry}>
                        {t("translations.list.retry")}
                    </Button>
                </div>
            ) : entries.length === 0 ? (
                <div className="px-2.5 py-6 text-[13.5px] text-muted-foreground">{emptyText}</div>
            ) : (
                <>
                    <ul className="flex flex-col">
                        {entries.map((entry) => (
                            <li key={entry.key}>
                                <button
                                    type="button"
                                    data-key={entry.key}
                                    aria-current={entry.key === selectedKey || undefined}
                                    onClick={() => onSelect(entry.key)}
                                    className={cn("flex w-full cursor-pointer items-start gap-2.5 rounded-lg px-2.5 py-[9px] text-left hover:bg-accent", entry.key === selectedKey && "bg-accent")}
                                >
                                    <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                                        <span className="truncate text-[13.5px]" title={entry.source_text}>
                                            {entry.source_text}
                                        </span>
                                        <span className="truncate font-mono text-[11.5px] text-muted-foreground" title={entry.key}>
                                            {entry.key}
                                        </span>
                                    </span>
                                    <StatusBadge status={entryStatus(entry)} size="sm" />
                                </button>
                            </li>
                        ))}
                    </ul>
                    {hasMore ? (
                        <div ref={sentinelRef} className="flex items-center justify-between gap-2 px-2.5 pt-2 pb-1">
                            <span className="text-[12px] text-muted-foreground">{t("translations.list.shown", { shown: fmt.number(entries.length), total: fmt.number(total) })}</span>
                            <Button size="xs" variant="ghost" loading={loadingMore} onClick={onLoadMore}>
                                {t("translations.list.more")}
                            </Button>
                        </div>
                    ) : null}
                </>
            )}
        </div>
    );
}
