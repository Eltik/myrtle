"use client";

import { Check } from "lucide-react";
import { cn } from "~/lib/utils";
import { SENIOR_OPERATOR_TAG_ID, TAG_GROUP_LABELS, TAG_GROUP_ORDER, TOP_OPERATOR_TAG_ID } from "./constants";
import type { RecruitmentTag, TagType } from "./types";

interface TagSelectorProps {
    tags: Record<TagType, RecruitmentTag[]>;
    selectedTags: number[];
    onTagToggle: (tagId: number) => void;
    maxTags: number;
}

export function TagSelector({ tags, selectedTags, onTagToggle, maxTags }: TagSelectorProps) {
    const isMaxSelected = selectedTags.length >= maxTags;

    return (
        <div className="space-y-5">
            {TAG_GROUP_ORDER.map((groupKey) => {
                const groupTags = tags[groupKey];
                if (!groupTags || groupTags.length === 0) return null;

                return (
                    <div className="space-y-2.5" key={groupKey}>
                        <div className="flex items-center gap-2">
                            <span className="font-semibold text-foreground/80 text-xs tracking-wide">{TAG_GROUP_LABELS[groupKey]}</span>
                            <div className="h-px flex-1 bg-border/50" />
                        </div>
                        <div className="flex flex-wrap gap-1.5">
                            {groupTags.map((tag) => {
                                const isSelected = selectedTags.includes(tag.id);
                                const isDisabled = !isSelected && isMaxSelected;
                                const isHighPriority = tag.id === TOP_OPERATOR_TAG_ID || tag.id === SENIOR_OPERATOR_TAG_ID;

                                return (
                                    <button
                                        className={cn(
                                            "group relative inline-flex items-center rounded-lg px-2.5 py-1.5 font-medium text-[13px]",
                                            // Base unselected state
                                            !isSelected && "bg-muted/50 text-muted-foreground hover:bg-muted hover:text-foreground",
                                            // Selected state
                                            isSelected && !isHighPriority && "bg-primary/15 text-primary ring-1 ring-primary/30",
                                            // High priority unselected
                                            isHighPriority && !isSelected && "bg-amber-500/10 text-amber-500/70 hover:bg-amber-500/20 hover:text-amber-400",
                                            // High priority selected
                                            isHighPriority && isSelected && "bg-amber-500/20 text-amber-400 ring-1 ring-amber-500/40",
                                            // Disabled state
                                            isDisabled && "cursor-not-allowed opacity-30 hover:bg-muted/50 hover:text-muted-foreground",
                                        )}
                                        disabled={isDisabled}
                                        key={tag.id}
                                        onClick={() => !isDisabled && onTagToggle(tag.id)}
                                        type="button"
                                    >
                                        {isSelected && <Check className={cn("mr-1 h-3 w-3", isHighPriority ? "text-amber-400" : "text-primary")} strokeWidth={3} />}
                                        {tag.name}
                                    </button>
                                );
                            })}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}
