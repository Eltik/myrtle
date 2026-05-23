import { useMemo, useState } from "react";
import { Markdown, stripMarkdown } from "#/lib/markdown";
import { cn } from "#/lib/utils";
import { DESCRIPTION_CLAMP_THRESHOLD } from "./shared";

interface IExpandableDescriptionProps {
    text: string;
    className?: string;
    clampLines?: number;
    threshold?: number;
    markdown?: boolean;
}

export function ExpandableDescription({ text, className, clampLines = 3, threshold = DESCRIPTION_CLAMP_THRESHOLD, markdown = false }: IExpandableDescriptionProps) {
    const [expanded, setExpanded] = useState(false);
    const measured = useMemo(() => (markdown ? stripMarkdown(text) : text), [markdown, text]);
    const isLong = measured.length > threshold;
    const showToggle = isLong;
    const clamped = isLong && !expanded;

    const clampStyle: React.CSSProperties | undefined = clamped
        ? {
              display: "-webkit-box",
              WebkitBoxOrient: "vertical",
              WebkitLineClamp: clampLines,
              overflow: "hidden",
          }
        : undefined;

    return (
        <div className={cn("flex flex-col gap-1.5", className)}>
            {markdown ? (
                <div className="wrap-break-word" style={clampStyle}>
                    <Markdown text={text} flush />
                </div>
            ) : (
                <p className={cn("wrap-break-word m-0 whitespace-pre-line")} style={clampStyle}>
                    {text}
                </p>
            )}
            {showToggle && (
                <button type="button" onClick={() => setExpanded((v) => !v)} aria-expanded={expanded} className="self-start font-mono text-[11px] text-primary uppercase tracking-[0.12em] hover:underline">
                    {expanded ? "Show less" : "Show more"}
                </button>
            )}
        </div>
    );
}
