import { RotateCcw, X } from "lucide-react";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IRecruitmentTag } from "../types";
import type { messages } from "./SelectedTagsBar.messages";

interface ISelectedTagsBarProps {
    selectedTags: IRecruitmentTag[];
    resultCount: number;
    onRemove: (id: number) => void;
    onReset: () => void;
}

export function SelectedTagsBar({ selectedTags, resultCount, onRemove, onReset }: ISelectedTagsBarProps): React.ReactElement | null {
    const t: TypedT<typeof messages> = useT("tools");
    if (selectedTags.length === 0) return null;

    return (
        <div className="flex flex-wrap items-center gap-x-2 gap-y-1.5 rounded-lg border bg-card px-3 py-2">
            <span className="font-medium font-sans text-[11px] text-muted-foreground uppercase tracking-wide">{t("recruit.selected")}</span>
            <div className="flex flex-1 basis-full flex-wrap items-center gap-1.5 sm:basis-auto">
                {selectedTags.map((tag) => (
                    <Button key={tag.id} size="xs" variant="secondary" onClick={() => onRemove(tag.id)} aria-label={t("recruit.selected.remove", { tag: tag.name })}>
                        {tag.name}
                        <X className="size-3" />
                    </Button>
                ))}
            </div>
            <span className="font-medium font-mono text-[11px] text-muted-foreground tabular-nums">{t("recruit.selected.comboCount", { count: resultCount })}</span>
            <Button size="xs" variant="ghost" onClick={onReset}>
                <RotateCcw />
                {t("recruit.selected.reset")}
            </Button>
        </div>
    );
}
