import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import type React from "react";
import { useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { storyIndexQueryOptions } from "#/lib/api/story";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { messages as sharedMessages } from "../shared.messages";
import type { messages } from "./reader.messages";

const CATEGORIES: StoryCategory[] = ["main", "side", "vignette", "record"];

export interface IChapterDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    currentStoryId: string;
    currentCategory: StoryCategory;
    currentGroupId: string;
}

/** Category -> group (or operator) -> story, then navigate. */
export function ChapterDialog({ open, onOpenChange, currentStoryId, currentCategory, currentGroupId }: IChapterDialogProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const tc: TypedT<typeof sharedMessages> = useT("story");
    const navigate = useNavigate();
    const { data: index } = useQuery({ ...storyIndexQueryOptions(useGamedataServer()), enabled: open });
    const [category, setCategory] = useState<StoryCategory>(currentCategory);
    const [groupId, setGroupId] = useState<string>(currentGroupId);
    const [storyId, setStoryId] = useState<string>(currentStoryId);

    const groups = useMemo(() => {
        if (!index) return [];
        if (category === "record") return index.records.map((r) => ({ id: r.charId, name: r.name, stories: r.stories }));
        return index.groups.filter((g) => g.category === category).map((g) => ({ id: g.id, name: g.name, stories: g.stories }));
    }, [index, category]);
    const stories: StoryEntry[] = useMemo(() => groups.find((g) => g.id === groupId)?.stories ?? [], [groups, groupId]);

    const pickCategory = (c: StoryCategory) => {
        setCategory(c);
        setGroupId("");
        setStoryId("");
    };
    const pickGroup = (g: string) => {
        setGroupId(g);
        setStoryId("");
    };

    const go = () => {
        if (!storyId) return;
        onOpenChange(false);
        void navigate({ to: "/stories/$storyId", params: { storyId } });
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup closeProps={{ className: "absolute end-2 top-2 max-sm:size-11" }}>
                <DialogHeader>
                    <DialogTitle>{t("chapter.title")}</DialogTitle>
                </DialogHeader>
                <DialogPanel className="flex max-h-[70dvh] flex-col gap-3">
                    <div className="flex flex-col gap-1 text-sm">
                        <span className="text-muted-foreground">{t("chapter.category")}</span>
                        <Select value={category} onValueChange={(v: string | null) => v && pickCategory(v as StoryCategory)}>
                            <SelectTrigger className="max-sm:h-11">
                                <SelectValue>{() => tc(`category.${category}`)}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {CATEGORIES.map((c) => (
                                    <SelectItem key={c} value={c}>
                                        {tc(`category.${c}`)}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="flex flex-col gap-1 text-sm">
                        <span className="text-muted-foreground">{category === "record" ? t("chapter.operator") : t("chapter.group")}</span>
                        <Select value={groupId} onValueChange={(v: string | null) => v && pickGroup(v)}>
                            <SelectTrigger className="max-sm:h-11">
                                <SelectValue>{() => groups.find((g) => g.id === groupId)?.name ?? ""}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {groups.map((g) => (
                                    <SelectItem key={g.id} value={g.id}>
                                        {g.name}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="flex flex-col gap-1 text-sm">
                        <span className="text-muted-foreground">{t("chapter.story")}</span>
                        <Select value={storyId} onValueChange={(v: string | null) => v && setStoryId(v)}>
                            <SelectTrigger className="max-sm:h-11">
                                <SelectValue>{() => stories.find((s) => s.id === storyId)?.name ?? ""}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {stories.map((s) => (
                                    <SelectItem key={s.id} value={s.id} disabled={!s.hasScript}>
                                        {s.code ? `${s.code} ` : ""}
                                        {s.name}
                                        {s.avgTag ? ` · ${s.avgTag}` : ""}
                                        {s.hasScript ? "" : ` ${t("chapter.noScript")}`}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                </DialogPanel>
                <DialogFooter>
                    <Button className="max-sm:h-11" onClick={go} disabled={!storyId || storyId === currentStoryId}>
                        {t("chapter.open")}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
