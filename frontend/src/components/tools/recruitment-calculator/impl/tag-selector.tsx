"use client";

import { Check, ChevronDown } from "lucide-react";
import { cn } from "~/lib/utils";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
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
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
            {TAG_GROUP_ORDER.map((groupKey) => {
                const groupTags = tags[groupKey];
                if (!groupTags || groupTags.length === 0) return null;

                return (
                    <Disclosure key={groupKey} open={true} className="rounded-md border border-border">
                        <DisclosureTrigger>
                            <div className="flex w-full items-center justify-between rounded-lg bg-muted/50 px-3 py-2.5">
                                <span className="font-semibold text-foreground/80 text-sm tracking-wide">{TAG_GROUP_LABELS[groupKey]}</span>
                                <ChevronDown className="group-data-[state=closed]:rotate-90 h-4 w-4 transition-transform duration-200" />
                            </div>
                        </DisclosureTrigger>
                        <DisclosureContent>
                            <div className="flex flex-wrap gap-1.5 p-3">
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
                                            {tag.name}
                                        </button>
                                    );
                                })}
                            </div>
                        </DisclosureContent>
                    </Disclosure>
                );
            })}
        </div>
    );
}
