/**
 * ONE STORY AS A PRINTABLE DOCUMENT (`/stories/$storyId?view=book`): the
 * book's own HTML (`toHtml`, the EPUB's typography) in an isolated frame, so
 * none of the site's CSS reaches it, with one bar above it: back to the
 * reader, and Print, which prints the frame alone through the book's
 * `@media print` rules ("Save as PDF" is the browser's own destination there).
 */
import { Link } from "@tanstack/react-router";
import type React from "react";
import { useMemo, useRef } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type BookGroup, bookOf } from "#/lib/story/book/book";
import { toHtml } from "#/lib/story/book/html";
import { resolveNickname, useStorySettings } from "#/lib/story/settings";
import type { StoryScript } from "#/types/generated/StoryScript";
import type { messages } from "./export.messages";

export function BookView({ script, group, server }: { script: StoryScript; group: BookGroup; server: string }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const [settings] = useStorySettings();
    const frame = useRef<HTMLIFrameElement>(null);
    const html = useMemo(() => {
        const book = bookOf({ index: { groups: [group] }, scripts: new Map([[script.id, script]]), server }, { kind: "story", storyId: script.id }, { nickname: resolveNickname(settings.nickname), images: "cg+bg", branches: "all" });
        return toHtml(book, { typeface: "inter", fonts: { regular: "/fonts/inter-latin-400-normal.woff", bold: "/fonts/inter-latin-700-normal.woff", format: "woff" }, imageSrc: (path) => asset(path) });
    }, [script, group, server, settings.nickname]);
    return (
        <div className="flex h-dvh flex-col">
            <div className="flex items-center justify-between gap-2 border-border border-b px-4 py-2">
                <Button variant="ghost" size="sm" className="max-sm:h-11" render={<Link to="/stories/$storyId" params={{ storyId: script.id }} />}>
                    {t("export.book.back")}
                </Button>
                <Button size="sm" className="max-sm:h-11" onClick={() => frame.current?.contentWindow?.print()} data-book-print>
                    {t("export.book.print")}
                </Button>
            </div>
            <iframe ref={frame} title={script.name} srcDoc={html} className="min-h-0 w-full flex-1 bg-white" data-book-frame />
        </div>
    );
}
