import type React from "react";

/**
 * Parses text containing <color name=#hex>text</color> tags
 * and returns React elements with inline color styling.
 *
 * Example input: "<color name=#ffffff>EPOQUE Collection</color> - Description"
 * Returns: [<span style={{color: "#ffffff"}}>EPOQUE Collection</span>, " - Description"]
 */
export function parseColorTags(text: string): React.ReactNode[] {
    const result: React.ReactNode[] = [];
    // Use [\s\S]*? to match any character including newlines (non-greedy)
    // Support both name=#hex and name=#hex formats
    const regex = /<color\s+name=(#[0-9a-fA-F]{3,8})>([\s\S]*?)<\/color>/gi;

    let lastIndex = 0;
    let keyIndex = 0;
    let match = regex.exec(text);

    while (match !== null) {
        // Add text before the match
        if (match.index > lastIndex) {
            result.push(text.slice(lastIndex, match.index));
        }

        // Add the colored span
        const [, color, content] = match;
        result.push(
            <span key={`color-${keyIndex++}`} style={{ color }}>
                {content}
            </span>,
        );

        lastIndex = regex.lastIndex;
        match = regex.exec(text);
    }

    // Add remaining text after last match
    if (lastIndex < text.length) {
        result.push(text.slice(lastIndex));
    }

    // If no matches found, return original text
    if (result.length === 0) {
        return [text];
    }

    return result;
}

/**
 * Component wrapper for parsing and rendering color-tagged text
 */
interface ColorTagTextProps {
    text: string;
    className?: string;
}

export function ColorTagText({ text, className }: ColorTagTextProps) {
    return <span className={className}>{parseColorTags(text)}</span>;
}
