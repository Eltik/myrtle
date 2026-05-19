import { useState } from "react";
import { cn } from "#/lib/utils";
import { DESCRIPTION_CLAMP_THRESHOLD } from "./shared";

interface IExpandableDescriptionProps {
    text: string;
    className?: string;
    clampLines?: number;
    threshold?: number;
}

export function ExpandableDescription({ text, className, clampLines = 3, threshold = DESCRIPTION_CLAMP_THRESHOLD }: IExpandableDescriptionProps) {
    const [expanded, setExpanded] = useState(false);
    const isLong = text.length > threshold;
    const showToggle = isLong;
    const clamped = isLong && !expanded;

    return (
        <div className={cn("flex flex-col gap-1.5", className)}>
            <p
                className={cn("wrap-break-word m-0 whitespace-pre-line")}
                style={
                    clamped
                        ? {
                              display: "-webkit-box",
                              WebkitBoxOrient: "vertical",
                              WebkitLineClamp: clampLines,
                              overflow: "hidden",
                          }
                        : undefined
                }
            >
                {text}
            </p>
            {showToggle && (
                <button type="button" onClick={() => setExpanded((v) => !v)} aria-expanded={expanded} className="self-start font-mono text-[11px] text-primary uppercase tracking-[0.12em] hover:underline">
                    {expanded ? "Show less" : "Show more"}
                </button>
            )}
        </div>
    );
}
