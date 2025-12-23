"use client";

import type React from "react";
import { cn } from "~/lib/utils";

/**
 * Parses a paragraph of lore text and returns the appropriate styled React element.
 * Handles different content types: headers, technical content, lists, quotes, emphasis, and statistics.
 */
export function renderParagraph(paragraph: string, index: number): React.ReactNode {
    // Skip empty paragraphs
    const trimmed = paragraph.trim();
    if (!trimmed) return null;

    // Handle section headers with [HeaderName] format (e.g., [Race] Vouivre)
    if (trimmed.startsWith("[") && trimmed.includes("]")) {
        const closingBracket = trimmed.indexOf("]");
        const headerText = trimmed.substring(1, closingBracket).trim();
        const contentText = trimmed.substring(closingBracket + 1).trim();

        // Headers related to stats or measurements use monospace styling
        const statHeaders = [
            "Race",
            "Place of Birth",
            "Date of Birth",
            "Physical Strength",
            "Mobility",
            "Physical Resilience",
            "Tactical Acumen",
            "Combat Skill",
            "Originium Arts Assimilation",
            "Height",
            "Weight",
            "Code Name",
            "Gender",
            "Combat Experience",
            "Cell-Originium Assimilation",
            "Blood Originium-Crystal Density",
            "Infection Status",
        ];

        const isStatHeader = statHeaders.includes(headerText);

        return (
            <div className="flex flex-col gap-1 py-1.5 sm:flex-row sm:items-baseline sm:gap-3" key={index}>
                <span className="min-w-40 shrink-0 font-semibold text-primary text-xs uppercase tracking-wide sm:text-right">{headerText}</span>
                <span className={cn("flex-1 text-foreground text-sm leading-relaxed", isStatHeader && "rounded-md bg-secondary/40 px-2 py-1 font-mono text-xs")}>{contentText || "—"}</span>
            </div>
        );
    }

    // Handle technical/scientific content (contains specific keywords and is relatively short)
    const technicalKeywords = /\b(Originium|Arts|Cell|Oripathy|Protocol|System|Algorithm|Version|Module|Device|Catalyst|Infection|Crystal|Density|Assimilation)\b/i;
    if (technicalKeywords.test(trimmed) && trimmed.length < 150) {
        return (
            <div className="my-3 rounded-lg border border-primary/20 bg-primary/5 px-4 py-3" key={index}>
                <code className="block font-mono text-foreground/90 text-xs leading-relaxed sm:text-sm">{trimmed}</code>
            </div>
        );
    }

    // Handle lists (lines starting with "-", "•", or "·")
    if (/^[-•·]/.test(trimmed)) {
        return (
            <div className="flex gap-3 py-1 pl-2" key={index}>
                <span className="mt-1 text-primary">•</span>
                <span className="flex-1 text-muted-foreground text-sm leading-relaxed">{trimmed.substring(1).trim()}</span>
            </div>
        );
    }

    // Handle numbered lists (e.g., "1.", "2.", etc.)
    if (/^\d+[.)]\s/.test(trimmed)) {
        const match = trimmed.match(/^(\d+[.)])\s*(.*)$/);
        if (match) {
            return (
                <div className="flex gap-3 py-1 pl-2" key={index}>
                    <span className="min-w-6 font-mono text-primary text-sm">{match[1]}</span>
                    <span className="flex-1 text-muted-foreground text-sm leading-relaxed">{match[2]}</span>
                </div>
            );
        }
    }

    // Handle quotes (wrapped in quotes or starting with >)
    if ((trimmed.startsWith('"') && trimmed.endsWith('"')) || (trimmed.startsWith("'") && trimmed.endsWith("'")) || trimmed.startsWith(">") || trimmed.startsWith("「") || trimmed.startsWith("『")) {
        let quoteContent = trimmed;
        if (trimmed.startsWith(">")) {
            quoteContent = trimmed.substring(1).trim();
        } else if ((trimmed.startsWith('"') && trimmed.endsWith('"')) || (trimmed.startsWith("'") && trimmed.endsWith("'"))) {
            quoteContent = trimmed.substring(1, trimmed.length - 1).trim();
        }

        return (
            <blockquote className="my-3 rounded-r-lg border-primary/40 border-l-4 bg-secondary/30 py-3 pr-4 pl-4" key={index}>
                <p className="text-muted-foreground text-sm italic leading-relaxed">{quoteContent}</p>
            </blockquote>
        );
    }

    // Handle emphasized paragraphs (ALL CAPS sections longer than 10 chars)
    if (trimmed.toUpperCase() === trimmed && trimmed.length > 10 && /[A-Z]/.test(trimmed)) {
        return (
            <p className="my-3 font-semibold text-amber-600 text-sm tracking-wide dark:text-amber-400" key={index}>
                {trimmed}
            </p>
        );
    }

    // Handle paragraphs with numeric data or statistics (percentages, decimals)
    const hasNumericData = /\b([0-9]+\.?[0-9]*%|[0-9]+[.,][0-9]+)\b/.test(trimmed);
    if (hasNumericData && trimmed.length < 200) {
        const parts = trimmed.split(/(\b[0-9]+\.?[0-9]*%|\b[0-9]+[.,][0-9]+\b)/g);

        // Create unique keys by tracking occurrences of each part
        const keyCount: Record<string, number> = {};
        const getUniqueKey = (part: string) => {
            keyCount[part] = (keyCount[part] || 0) + 1;
            return `${part}-${keyCount[part]}`;
        };

        return (
            <p className="my-2 text-muted-foreground text-sm leading-relaxed" key={index}>
                {parts.map((part) => {
                    const uniqueKey = getUniqueKey(part);
                    return /\b([0-9]+\.?[0-9]*%|[0-9]+[.,][0-9]+)\b/.test(part) ? (
                        <span className="font-mono font-semibold text-emerald-600 dark:text-emerald-400" key={uniqueKey}>
                            {part}
                        </span>
                    ) : (
                        <span key={uniqueKey}>{part}</span>
                    );
                })}
            </p>
        );
    }

    // Regular paragraph with good typography
    return (
        <p className="my-2 text-muted-foreground text-sm leading-relaxed" key={index}>
            {trimmed}
        </p>
    );
}
