import { Card } from "#/components/ui/card";
import { ScrollArea } from "#/components/ui/scroll-area";
import { TooltipProvider } from "#/components/ui/tooltip";
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
    const activeIndex = TABS.findIndex((t) => t.type === activeTab);

    return (
        <TooltipProvider>
            <div className="flex min-w-0 flex-col lg:flex-row lg:gap-8">
                <nav aria-label="Operator sections" className="min-w-0 shrink-0 lg:w-56">
                    <div className="sticky top-14 z-30 -mx-3 mb-4 border-b border-border/50 bg-background/80 backdrop-blur-xl sm:top-16 sm:-mx-4 lg:hidden">
                        <ScrollArea className="w-full">
                            <div className="relative flex gap-1 px-3 py-2.5 sm:px-4" role="tablist">
                                {TABS.map((tab) => {
                                    const Icon = tab.icon;
                                    const isActive = activeTab === tab.type;
                                    return (
                                        <button
                                            aria-controls={`panel-${tab.type}`}
                                            aria-selected={isActive}
                                            className={cn(
                                                "group relative flex shrink-0 items-center gap-2 rounded-full px-4 py-1.5 font-medium text-sm transition-all duration-200",
                                                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background",
                                                isActive
                                                    ? "bg-primary text-primary-foreground shadow-sm"
                                                    : "text-muted-foreground hover:bg-accent hover:text-foreground",
                                            )}
                                            key={tab.type}
                                            onClick={() => onTabChange(tab.type)}
                                            role="tab"
                                            type="button"
                                        >
                                            <Icon className="h-4 w-4" />
                                            <span>{tab.label}</span>
                                        </button>
                                    );
                                })}
                            </div>
                        </ScrollArea>
                    </div>
                    <div className="sticky top-20 hidden lg:block" role="tablist">
                        <div className="relative flex flex-col gap-0.5 rounded-xl border border-border/60 bg-card/40 p-2 backdrop-blur-sm">
                            <div aria-hidden className="absolute left-2 right-2 h-11 rounded-lg bg-primary/10 ring-1 ring-primary/30 transition-all duration-300 ease-out"
                                style={{
                                    transform: `translateY(${activeIndex * (44 + 2)}px)`,
                                    opacity: activeIndex >= 0 ? 1 : 0,
                                }}
                            />
                            {TABS.map((tab) => {
                                const Icon = tab.icon;
                                const isActive = activeTab === tab.type;
                                return (
                                    <button
                                        aria-controls={`panel-${tab.type}`}
                                        aria-selected={isActive}
                                        className={cn(
                                            "group relative z-10 flex h-11 items-center gap-3 rounded-lg px-3 text-left font-medium text-sm transition-colors duration-200",
                                            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                            isActive
                                                ? "text-foreground"
                                                : "text-muted-foreground hover:text-foreground",
                                        )}
                                        key={tab.type}
                                        onClick={() => onTabChange(tab.type)}
                                        role="tab"
                                        type="button"
                                    >
                                        <Icon
                                            className={cn(
                                                "h-4 w-4 shrink-0 transition-colors",
                                                isActive ? "text-primary" : "text-muted-foreground/70 group-hover:text-foreground",
                                            )}
                                        />
                                        <span className="truncate">{tab.label}</span>
                                        {isActive && (
                                            <span
                                                aria-hidden
                                                className="ml-auto h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_8px_hsl(var(--primary))]"
                                            />
                                        )}
                                    </button>
                                );
                            })}
                        </div>

                        <div className="mt-3 px-3">
                            <p className="truncate font-mono text-xs uppercase tracking-wider text-muted-foreground/60">
                                {operator.name ?? "Operator"}
                            </p>
                        </div>
                    </div>
                </nav>
                <section
                    aria-labelledby={`panel-${activeTab}`}
                    className="min-w-0 flex-1"
                    id={`panel-${activeTab}`}
                    role="tabpanel"
                >
                    <Card className="overflow-hidden border-border/60 bg-card/60 backdrop-blur-md">
                        <div
                            className="animate-in fade-in-50 slide-in-from-bottom-1 duration-300"
                            key={activeTab}
                        >
                            <ActiveContent operator={operator} />
                        </div>
                    </Card>
                </section>
            </div>
        </TooltipProvider>
    )
}
