"use client";

import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { cn } from "~/lib/utils";
import type { RecruitmentTag, TagType } from "~/types/frontend/impl/tools/recruitment";
import { SENIOR_OPERATOR_TAG_ID, TAG_GROUP_LABELS, TAG_GROUP_ORDER, TOP_OPERATOR_TAG_ID } from "./constants";

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
                    <Disclosure className="rounded-md border border-border" key={groupKey} open={true}>
                        <DisclosureTrigger>
                            <div className="flex w-full items-center justify-between rounded-lg bg-muted/50 px-3 py-2.5">
                                <span className="font-semibold text-foreground/80 text-sm tracking-wide">{TAG_GROUP_LABELS[groupKey]}</span>
                                <ChevronDown className="h-4 w-4 transition-transform duration-200 group-data-[state=closed]:rotate-90" />
                            </div>
                        </DisclosureTrigger>
                        <DisclosureContent>
                            <div className="flex flex-wrap gap-1.5 p-3">
                                {groupTags.map((tag) => {
                                    const isSelected = selectedTags.includes(tag.id);
                                    const isDisabled = !isSelected && isMaxSelected;
                                    const isHighPriority = tag.id === TOP_OPERATOR_TAG_ID || tag.id === SENIOR_OPERATOR_TAG_ID;

                                    return (
                                        <motion.button
                                            animate={{
                                                opacity: isDisabled ? 0.35 : 1,
                                                scale: isDisabled ? 0.95 : 1,
                                            }}
                                            className={cn(
                                                "group relative inline-flex items-center rounded-lg px-2.5 py-1.5 font-medium text-[0.8125rem] transition-colors",
                                                !isSelected && !isDisabled && !isHighPriority && "bg-muted/50 text-muted-foreground hover:bg-muted hover:text-foreground",
                                                isSelected && !isHighPriority && "bg-primary/15 text-primary ring-1 ring-primary/30",
                                                tag.id === TOP_OPERATOR_TAG_ID && !isSelected && !isDisabled && "bg-orange-500/15 text-orange-700 hover:bg-orange-500/25 hover:text-orange-600 dark:text-orange-400 dark:hover:text-orange-300",
                                                tag.id === TOP_OPERATOR_TAG_ID && isSelected && "bg-orange-500/25 text-orange-700 ring-1 ring-orange-500/50 dark:text-orange-400",
                                                tag.id === SENIOR_OPERATOR_TAG_ID && !isSelected && !isDisabled && "bg-yellow-500/15 text-yellow-700 hover:bg-yellow-500/25 hover:text-yellow-600 dark:text-yellow-400 dark:hover:text-yellow-300",
                                                tag.id === SENIOR_OPERATOR_TAG_ID && isSelected && "bg-yellow-500/25 text-yellow-700 ring-1 ring-yellow-500/40 dark:text-yellow-400",
                                                isDisabled && "cursor-not-allowed bg-muted/30 text-muted-foreground/50",
                                            )}
                                            disabled={isDisabled}
                                            key={tag.id}
                                            onClick={() => !isDisabled && onTagToggle(tag.id)}
                                            transition={{ duration: 0.15 }}
                                            type="button"
                                            whileHover={!isDisabled ? { scale: 1.05 } : undefined}
                                            whileTap={!isDisabled ? { scale: 0.95, opacity: 0.7, filter: "blur(0.5px)" } : undefined}
                                        >
                                            {tag.name}
                                        </motion.button>
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
