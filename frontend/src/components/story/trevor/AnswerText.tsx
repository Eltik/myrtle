/**
 * An answer's text as React nodes: paragraphs, bullet lists, bold, and the
 * citation markers as small links to the source list below the answer. Never
 * HTML: the text is model output.
 */
import type React from "react";
import { Fragment } from "react";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type AnswerInline, answerBlocks } from "#/lib/trevor/answer";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

export interface IAnswerTextProps {
    text: string;
    /** Prefix of the source rows' element ids; null draws the markers without links (while streaming). */
    sourceId: string | null;
    /** The citation numbers that have a source row to link to. */
    cited: ReadonlySet<number>;
}

export function AnswerText({ text, sourceId, cited }: IAnswerTextProps): React.ReactElement {
    const t: TrevorT = useT("story");
    const inline = (runs: AnswerInline[]) =>
        runs.map((r, i) => {
            const key = `${r.kind}-${i}`;
            if (r.kind === "text") return <Fragment key={key}>{r.text}</Fragment>;
            if (r.kind === "bold") return <strong key={key}>{r.text}</strong>;
            return (
                <sup key={key} className="ms-0.5 font-mono text-[0.7em] text-muted-foreground">
                    [
                    {r.ns.map((n, j) => (
                        <Fragment key={n}>
                            {j > 0 ? ", " : null}
                            {sourceId && cited.has(n) ? (
                                <a href={`#${sourceId}-${n}`} aria-label={t("trevor.answer.cite", { n })} className="rounded-sm text-primary underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                                    {n}
                                </a>
                            ) : (
                                n
                            )}
                        </Fragment>
                    ))}
                    ]
                </sup>
            );
        });
    return (
        <div className="flex flex-col gap-2.5 text-sm leading-relaxed">
            {answerBlocks(text).map((b, i) =>
                b.kind === "p" ? (
                    // biome-ignore lint/suspicious/noArrayIndexKey: blocks have no identity beyond their order in one immutable text.
                    <p key={i}>{inline(b.inlines)}</p>
                ) : (
                    // biome-ignore lint/suspicious/noArrayIndexKey: as above.
                    <ul key={i} className="flex list-disc flex-col gap-1.5 ps-5">
                        {b.items.map((item, j) => (
                            // biome-ignore lint/suspicious/noArrayIndexKey: as above.
                            <li key={j}>{inline(item)}</li>
                        ))}
                    </ul>
                ),
            )}
        </div>
    );
}
