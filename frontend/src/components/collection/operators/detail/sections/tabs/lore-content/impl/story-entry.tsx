"use client";

import { memo } from "react";
import { renderParagraph } from "./helpers";

interface StoryEntryProps {
    storyText: string;
}

/**
 * Individual story entry component
 */
export const StoryEntry = memo(function StoryEntry({ storyText }: StoryEntryProps) {
    const paragraphs = storyText.split("\n");

    return <div className="prose-sm max-w-none">{paragraphs.map((p, idx) => renderParagraph(p, idx))}</div>;
});
