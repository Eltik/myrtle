import { Badge, Tabs, TabsList, TabsPanel, TabsTab } from "frontend";
import { BookOpenIcon, BoxesIcon, SwordsIcon, UserIcon } from "lucide-react";

/** The enemy detail page's section switcher — default (segmented) list plus panels. */
export const Segmented = () => (
    <Tabs className="w-full max-w-xl gap-0" defaultValue="overview">
        <div className="border-border border-b">
            <TabsList className="mb-3">
                <TabsTab className="h-8 text-[13px]" value="overview">
                    Overview
                </TabsTab>
                <TabsTab className="h-8 text-[13px]" value="stats">
                    Stats
                </TabsTab>
                <TabsTab className="h-8 text-[13px]" value="skills">
                    Skills
                </TabsTab>
                <TabsTab className="h-8 text-[13px]" value="appears">
                    Appears In
                </TabsTab>
            </TabsList>
        </div>
        <div className="pt-5">
            <TabsPanel value="overview">
                <p className="text-muted-foreground text-sm">
                    <span className="font-medium text-foreground">Sarkaz Greatsword Wielder</span> — a heavy melee enemy introduced in Chapter 8. Blocks 2, ignores the first 400 physical damage, and enrages below 40% HP.
                </p>
            </TabsPanel>
            <TabsPanel value="stats">Stats</TabsPanel>
            <TabsPanel value="skills">Skills</TabsPanel>
            <TabsPanel value="appears">Appears In</TabsPanel>
        </div>
    </Tabs>
);

/** `variant="underline"` — the indicator becomes a brand-red rule under the active tab. */
export const Underline = () => (
    <Tabs className="w-full max-w-xl gap-0" defaultValue="skills">
        <div className="border-border border-b">
            <TabsList variant="underline">
                <TabsTab value="overview">Overview</TabsTab>
                <TabsTab value="skills">Skills</TabsTab>
                <TabsTab value="modules">Modules</TabsTab>
                <TabsTab value="riic">Base skills</TabsTab>
            </TabsList>
        </div>
        <div className="pt-5">
            <TabsPanel value="skills">
                <div className="flex flex-col gap-1">
                    <span className="font-medium text-sm">Sworn Enemy of Evil · M3</span>
                    <span className="text-muted-foreground text-sm">Attack interval −0.3s. Attack power +160%. Attacks deal Arts damage for 20 seconds.</span>
                </div>
            </TabsPanel>
            <TabsPanel value="overview">Overview</TabsPanel>
            <TabsPanel value="modules">Modules</TabsPanel>
            <TabsPanel value="riic">Base skills</TabsPanel>
        </div>
    </Tabs>
);

/** `orientation="vertical"` — the operator detail sidebar, as OperatorTabs composes it. */
export const Vertical = () => (
    <Tabs className="w-full max-w-xl" defaultValue="art" orientation="vertical">
        <TabsList className="w-56" variant="underline">
            <TabsTab value="overview">
                <UserIcon />
                Overview
            </TabsTab>
            <TabsTab value="skills">
                <SwordsIcon />
                Skills
            </TabsTab>
            <TabsTab value="modules">
                <BoxesIcon />
                Modules
            </TabsTab>
            <TabsTab value="art">
                <BookOpenIcon />
                Art &amp; files
            </TabsTab>
        </TabsList>
        <div className="flex-1 pl-6">
            <TabsPanel value="art">
                <div className="flex flex-col gap-2">
                    <span className="font-medium text-sm">Młynar</span>
                    <span className="text-muted-foreground text-sm">4 skins · 2 dynamic illustrations · 31 voice lines.</span>
                    <div className="flex gap-2">
                        <Badge variant="secondary">Elite II</Badge>
                        <Badge variant="outline">Guard</Badge>
                    </div>
                </div>
            </TabsPanel>
            <TabsPanel value="overview">Overview</TabsPanel>
            <TabsPanel value="skills">Skills</TabsPanel>
            <TabsPanel value="modules">Modules</TabsPanel>
        </div>
    </Tabs>
);
