import type React from "react";
import { useState } from "react";
import { nextPage, pageOf, RECORD_PAGE } from "#/components/story/library/impl/paging";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "../MusicGallery.messages";

type MusicT = TypedT<typeof messages>;

/** The dashed box a list view shows in place of an empty list or a missing source. */
export const NOTICE_CLASS = "rounded-xl border border-border border-dashed p-10 text-center font-sans text-[13px] text-muted-foreground";

export interface IPagedList<T> {
    shown: readonly T[];
    /** How many items are behind the button. */
    hidden: number;
    /** How many the next press reveals. */
    more: number;
    showMore: () => void;
}

/**
 * `items` a page at a time. The limit is remembered against the `pageKey` it
 * was raised under, so new controls start from one page without an effect.
 */
export function usePagedList<T>(items: readonly T[], pageKey: string): IPagedList<T> {
    const [raised, setRaised] = useState<{ key: string; limit: number }>({ key: pageKey, limit: RECORD_PAGE });
    const limit = raised.key === pageKey ? raised.limit : RECORD_PAGE;
    const { shown, hidden } = pageOf(items, limit);
    const more = nextPage(hidden);
    return { shown, hidden, more, showMore: () => setRaised({ key: pageKey, limit: limit + more }) };
}

/** The "Show more" button under a paged list; nothing once every item is shown. */
export function ShowMore({ page }: { page: IPagedList<unknown> }): React.ReactElement | null {
    const t: MusicT = useT("gallery");
    if (page.hidden <= 0) return null;
    return (
        <div className="mt-4 flex justify-center">
            <Button variant="outline" className="max-sm:min-h-11" onClick={page.showMore}>
                {t("music.showMore", { count: page.more, remaining: page.hidden })}
            </Button>
        </div>
    );
}
