import type * as React from "react";
import { Button } from "#/components/ui/button";
import { TAG_GROUP_LABELS, TAG_GROUP_ORDER } from "../constants";
import type { IRecruitmentTag, TagType } from "../types";

interface ITagSelectorProps {
    groups: Record<TagType, IRecruitmentTag[]>;
    selectedTagIds: ReadonlySet<number>;
    onToggle: (id: number) => void;
    maxReached: boolean;
}

export function TagSelector({ groups, selectedTagIds, onToggle, maxReached }: ITagSelectorProps): React.ReactElement {
    return (
        <div className="flex flex-col gap-4">
            {TAG_GROUP_ORDER.map((type) => {
                const tagsForGroup = groups[type];
                if (tagsForGroup.length === 0) return null;
                return (
                    <section key={type}>
                        <h3 className="mb-2 font-medium font-sans text-[11px] uppercase tracking-wide text-muted-foreground">{TAG_GROUP_LABELS[type]}</h3>
                        <div className="flex flex-wrap gap-1.5">
                            {tagsForGroup.map((tag) => {
                                const selected = selectedTagIds.has(tag.id);
                                const disabled = !selected && maxReached;
                                return (
                                    <Button key={tag.id} size="xs" variant={selected ? "default" : "outline"} disabled={disabled} aria-pressed={selected} onClick={() => onToggle(tag.id)}>
                                        {tag.name}
                                    </Button>
                                );
                            })}
                        </div>
                    </section>
                );
            })}
        </div>
    );
}
