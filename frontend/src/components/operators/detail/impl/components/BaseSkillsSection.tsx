import { Building2, ChevronDown } from "lucide-react";
import { useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import type { IOperatorBaseSkill } from "#/types/operators";
import { baseSkillIcon, eliteIcon } from "../assets";
import { descriptionToHtml } from "../description";

interface IBaseSkillsSectionProps {
    skills: IOperatorBaseSkill[];
    server?: "en" | "cn";
}

const ROOM_LABEL: Record<string, string> = {
    CONTROL: "Control Center",
    MANUFACTURE: "Factory",
    TRADING: "Trading Post",
    POWER: "Power Plant",
    DORMITORY: "Dormitory",
    HIRE: "Office",
    MEETING: "Reception Room",
    TRAINING: "Training Room",
    WORKSHOP: "Workshop",
};

function roomLabel(roomType: string): string {
    return ROOM_LABEL[roomType] ?? "Base";
}

function formatTarget(target: string): string {
    // Targets look like "F_GOLD" / "F_EXP" / "F_ASC" - strip the F_ prefix and title-case.
    const trimmed = target.replace(/^F_/, "").toLowerCase();
    return trimmed.charAt(0).toUpperCase() + trimmed.slice(1);
}

export function BaseSkillsSection({ skills, server }: IBaseSkillsSectionProps) {
    const [open, setOpen] = useState(true);

    if (!skills || skills.length === 0) return null;

    // Sort by unlock order: elite ASC, then level ASC. The source data is already
    // grouped by slot, but explicit sort makes the ladder obvious.
    const sorted = [...skills].sort((a, b) => a.unlockElite - b.unlockElite || a.unlockLevel - b.unlockLevel);

    return (
        <Collapsible open={open} onOpenChange={setOpen}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-2">
                    <Building2 className="h-4 w-4 text-primary" />
                    <span className="font-medium text-sm">Base Skills</span>
                    <Badge variant="outline" className="text-[10px]">
                        {sorted.length}
                    </Badge>
                </span>
                <ChevronDown className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>
                <div className="mt-3 grid gap-3 md:grid-cols-2">
                    {sorted.map((skill) => {
                        const iconUrl = skill.skillIcon ? baseSkillIcon(skill.skillIcon, server) : "";
                        const html = descriptionToHtml(skill.description, []);
                        const targets = skill.targets.map(formatTarget);
                        return (
                            <div className="rounded-lg border border-border/50 bg-card/30 p-3" key={skill.buffId}>
                                <div className="flex gap-3">
                                    <div className="relative flex h-12 w-12 shrink-0 items-center justify-center self-start rounded-md border border-border/40 bg-secondary/30">
                                        {iconUrl ? <img alt={skill.buffName} className="h-10 w-10 object-contain" decoding="async" loading="lazy" src={iconUrl} /> : <Building2 className="h-5 w-5 text-muted-foreground" />}
                                        <Tooltip>
                                            <TooltipTrigger
                                                render={(props) => (
                                                    <div {...props} className="absolute -right-1 -bottom-1 flex h-5 w-5 items-center justify-center rounded-full border border-border bg-background shadow-sm">
                                                        <img alt={`Elite ${skill.unlockElite}`} className="icon-theme-aware h-3.5 w-3.5 object-contain" decoding="async" loading="lazy" src={eliteIcon(skill.unlockElite, server)} />
                                                    </div>
                                                )}
                                            />
                                            <TooltipPopup>
                                                Unlocks at Elite {skill.unlockElite}, Lv {skill.unlockLevel}
                                            </TooltipPopup>
                                        </Tooltip>
                                    </div>
                                    <div className="min-w-0 flex-1">
                                        <div className="mb-1 flex flex-wrap items-center gap-2">
                                            <h4 className="font-medium text-foreground text-sm">{skill.buffName}</h4>
                                            <Badge variant="outline" className="text-[10px]">
                                                {roomLabel(skill.roomType)}
                                            </Badge>
                                        </div>
                                        <p className="text-[11px] text-muted-foreground">
                                            Elite {skill.unlockElite} · Lv {skill.unlockLevel}
                                            {targets.length > 0 && ` · ${targets.join(", ")}`}
                                        </p>
                                        <span
                                            className="mt-1.5 block text-muted-foreground text-xs"
                                            // biome-ignore lint/security/noDangerouslySetInnerHtml: sanitized via descriptionToHtml
                                            dangerouslySetInnerHTML={{ __html: html }}
                                        />
                                    </div>
                                </div>
                            </div>
                        );
                    })}
                </div>
            </CollapsibleContent>
        </Collapsible>
    );
}
