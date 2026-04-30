import { Card, CardPanel } from "#/components/ui/card";
import { ScrollArea } from "#/components/ui/scroll-area";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { Tooltip, TooltipPopup, TooltipProvider, TooltipTrigger } from "#/components/ui/tooltip";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { TABS, type TabType } from "../constants";
import { AudioContent } from "./tabs/AudioContent";
import { InfoContent } from "./tabs/InfoContent";
import { LevelUpContent } from "./tabs/LevelUpContent";
import { LoreContent } from "./tabs/LoreContent";
import { SkillsContent } from "./tabs/SkillsContent";
import { SkinsContent } from "./tabs/SkinsContent";

interface IOperatorTabsProps {
    operator: IOperatorListItem;
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

const CONTENT_MAP = {
    info: InfoContent,
    skills: SkillsContent,
    levelup: LevelUpContent,
    skins: SkinsContent,
    audio: AudioContent,
    lore: LoreContent,
} as const;

export function OperatorTabs({ operator, activeTab, onTabChange }: IOperatorTabsProps) {
    const ActiveContent = CONTENT_MAP[activeTab];
    const handleValueChange = (value: unknown) => {
        if (typeof value === "string") onTabChange(value as TabType);
    };

    return (
        <TooltipProvider>
            <div className="flex min-w-0 flex-col lg:flex-row lg:gap-8">
                <nav aria-label="Operator sections" className="min-w-0 shrink-0 lg:w-56">
                    <Tabs className="sticky top-14 z-30 -mx-3 mb-4 border-b border-border/50 bg-background/80 backdrop-blur-xl sm:top-16 sm:-mx-4 lg:hidden" onValueChange={handleValueChange} value={activeTab}>
                        <ScrollArea className="w-full" scrollFade>
                            <TabsList className={cn("w-max gap-1 rounded-none bg-transparent px-3 py-2.5 sm:px-4", "**:data-[slot=tab-indicator]:rounded-full **:data-[slot=tab-indicator]:bg-primary! **:data-[slot=tab-indicator]:shadow-sm")}>
                                {TABS.map((tab) => {
                                    const Icon = tab.icon;
                                    return (
                                        <TabsTab className="h-auto grow-0 rounded-full px-4 py-1.5 font-medium text-muted-foreground text-sm hover:bg-accent hover:text-foreground data-active:text-primary-foreground! data-active:hover:bg-transparent" key={tab.type} value={tab.type}>
                                            <Icon className="h-4 w-4" />
                                            <span>{tab.label}</span>
                                        </TabsTab>
                                    );
                                })}
                            </TabsList>
                        </ScrollArea>
                    </Tabs>

                    <Tabs className="sticky top-20 hidden lg:block" onValueChange={handleValueChange} orientation="vertical" value={activeTab}>
                        <TabsList
                            className={cn(
                                "w-full gap-0.5 rounded-xl border border-border/60 bg-card/40 p-2 backdrop-blur-sm",
                                "**:data-[slot=tab-indicator]:rounded-lg **:data-[slot=tab-indicator]:bg-primary/10 **:data-[slot=tab-indicator]:ring-1 **:data-[slot=tab-indicator]:ring-primary/30 `**:data-[slot=tab-indicator]:shadow-none",
                            )}
                        >
                            {TABS.map((tab) => {
                                const Icon = tab.icon;
                                return (
                                    <Tooltip key={tab.type}>
                                        <TooltipTrigger render={<TabsTab className="group h-11 justify-start gap-3 rounded-lg px-3 text-left font-medium text-muted-foreground text-sm hover:text-foreground data-active:text-foreground!" value={tab.type} />}>
                                            <Icon className="h-4 w-4 shrink-0 text-muted-foreground/70 transition-colors group-hover:text-foreground group-data-active:text-primary" />
                                            <span className="truncate">{tab.label}</span>
                                        </TooltipTrigger>
                                        <TooltipPopup side="right">{tab.label}</TooltipPopup>
                                    </Tooltip>
                                );
                            })}
                        </TabsList>

                        <div className="mt-3 px-3">
                            <p className="truncate font-mono text-muted-foreground/60 text-xs uppercase tracking-wider">{operator.name ?? "Operator"}</p>
                        </div>
                    </Tabs>
                </nav>

                <Card aria-labelledby={`panel-${activeTab}`} className="min-w-0 flex-1 overflow-hidden border-border/60 bg-card/60 backdrop-blur-md" id={`panel-${activeTab}`} render={<section />} role="tabpanel">
                    <CardPanel className="animate-in fade-in-50 slide-in-from-bottom-1 p-0 duration-300" key={activeTab}>
                        <ActiveContent operator={operator} />
                    </CardPanel>
                </Card>
            </div>
        </TooltipProvider>
    );
}
