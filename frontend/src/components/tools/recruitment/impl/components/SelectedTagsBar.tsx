import { RotateCcw, X } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import type { IRecruitmentTag } from "../types";

interface ISelectedTagsBarProps {
    selectedTags: IRecruitmentTag[];
    onRemove: (id: number) => void;
    onReset: () => void;
}

export function SelectedTagsBar({ selectedTags, onRemove, onReset }: ISelectedTagsBarProps): React.ReactElement | null {
    if (selectedTags.length === 0) return null;

    return (
        <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-card px-3 py-2">
            <span className="font-medium font-sans text-[11px] text-muted-foreground uppercase tracking-wide">Selected</span>
            <div className="flex flex-1 flex-wrap items-center gap-1.5">
                {selectedTags.map((tag) => (
                    <Button key={tag.id} size="xs" variant="secondary" onClick={() => onRemove(tag.id)} aria-label={`Remove ${tag.name}`}>
                        {tag.name}
                        <X className="size-3" />
                    </Button>
                ))}
            </div>
            <Button size="xs" variant="ghost" onClick={onReset}>
                <RotateCcw />
                Reset
            </Button>
        </div>
    );
}
