"use client";

import { X } from "lucide-react";
import { cn } from "~/lib/utils";
import { SENIOR_OPERATOR_TAG_ID, TOP_OPERATOR_TAG_ID } from "./constants";
import type { RecruitmentTag } from "./types";

interface SelectedTagsDisplayProps {
    tags: RecruitmentTag[];
    selectedTagIds: number[];
    onRemoveTag: (tagId: number) => void;
    onClearAll: () => void;
}

export function SelectedTagsDisplay({ tags, selectedTagIds, onRemoveTag, onClearAll }: SelectedTagsDisplayProps) {
    const selectedTags = selectedTagIds.map((id) => tags.find((t) => t.id === id)).filter(Boolean) as RecruitmentTag[];

    if (selectedTagIds.length === 0) {
        return null;
    }

    return (
        <div className="flex flex-wrap items-center gap-2 pt-2">
            <span className="text-muted-foreground text-sm">Selected:</span>
            {selectedTags.map((tag) => {
                const isHighPriority = tag.id === TOP_OPERATOR_TAG_ID || tag.id === SENIOR_OPERATOR_TAG_ID;
                return (
                    <span className={cn("inline-flex items-center gap-1 rounded-md px-2 py-1 font-medium text-xs", isHighPriority ? "bg-amber-500/20 text-amber-400" : "bg-primary/20 text-foreground")} key={tag.id}>
                        {tag.name}
                        <button className="rounded-full p-0.5 hover:bg-foreground/10" onClick={() => onRemoveTag(tag.id)} type="button">
                            <X className="h-3 w-3" />
                        </button>
                    </span>
                );
            })}
            {selectedTagIds.length > 1 && (
                <button className="text-muted-foreground text-xs hover:text-foreground" onClick={onClearAll} type="button">
                    Clear all
                </button>
            )}
        </div>
    );
}
