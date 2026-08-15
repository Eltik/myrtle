import { Badge, Tabs, TabsList, TabsPanel, TabsTab } from "frontend";

/** Only the panel matching the root's value renders — here, the stats table. */
export const ActivePanel = () => (
    <Tabs className="w-full max-w-xl gap-0" defaultValue="stats">
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
            </TabsList>
        </div>
        <div className="pt-5">
            <TabsPanel value="overview">Overview</TabsPanel>
            <TabsPanel value="stats">
                <dl className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm sm:grid-cols-3">
                    {[
                        { k: "HP", v: "3,468" },
                        { k: "ATK", v: "1,036" },
                        { k: "DEF", v: "455" },
                        { k: "RES", v: "0" },
                        { k: "Redeploy", v: "70s" },
                        { k: "DP cost", v: "24" },
                    ].map((s) => (
                        <div className="flex items-baseline justify-between gap-3" key={s.k}>
                            <dt className="text-muted-foreground">{s.k}</dt>
                            <dd className="font-mono tabular-nums">{s.v}</dd>
                        </div>
                    ))}
                </dl>
            </TabsPanel>
            <TabsPanel value="skills">Skills</TabsPanel>
        </div>
    </Tabs>
);

/** Panel content under an underline list — an operator's module summary. */
export const UnderModuleTabs = () => (
    <Tabs className="w-full max-w-xl gap-0" defaultValue="modules">
        <div className="border-border border-b">
            <TabsList variant="underline">
                <TabsTab value="overview">Overview</TabsTab>
                <TabsTab value="modules">Modules</TabsTab>
                <TabsTab value="riic">Base skills</TabsTab>
            </TabsList>
        </div>
        <div className="pt-5">
            <TabsPanel value="modules">
                <div className="flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                        <span className="font-medium text-sm">GUA-Y</span>
                        <Badge size="sm" variant="outline">
                            Stage 3
                        </Badge>
                    </div>
                    <p className="text-muted-foreground text-sm">ATK +26, DEF +26. Sworn Enemy of Evil no longer consumes a charge when the target dies.</p>
                </div>
            </TabsPanel>
            <TabsPanel value="overview">Overview</TabsPanel>
            <TabsPanel value="riic">Base skills</TabsPanel>
        </div>
    </Tabs>
);

/** Vertical orientation — the panel sits beside the rail and takes the remaining width. */
export const BesideVerticalRail = () => (
    <Tabs className="w-full max-w-xl" defaultValue="riic" orientation="vertical">
        <TabsList className="w-48" variant="underline">
            <TabsTab value="overview">Overview</TabsTab>
            <TabsTab value="modules">Modules</TabsTab>
            <TabsTab value="riic">Base skills</TabsTab>
        </TabsList>
        <div className="flex-1 pl-6">
            <TabsPanel value="riic">
                <div className="flex flex-col gap-1">
                    <span className="font-medium text-sm">Standardization γ</span>
                    <span className="text-muted-foreground text-sm">When stationed in a Factory, productivity +30%.</span>
                </div>
            </TabsPanel>
            <TabsPanel value="overview">Overview</TabsPanel>
            <TabsPanel value="modules">Modules</TabsPanel>
        </div>
    </Tabs>
);
