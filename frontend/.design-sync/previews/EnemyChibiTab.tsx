import { EnemyChibiTab, Tabs, TabsList, TabsPanel, TabsTab } from "frontend";

// The chibi tab resolves its spine data through a server function that the
// design bundle stubs out, so the honest render is the branch it settles into
// when the chibi index can't be reached.
export const ChibiUnavailable = () => (
    <div className="w-full max-w-2xl rounded-xl border border-border bg-card">
        <EnemyChibiTab enemyId="enemy_1500_skulsr" />
    </div>
);

export const WithinDetailTabs = () => (
    <div className="w-full max-w-2xl">
        <Tabs defaultValue="chibi" className="flex flex-col gap-0">
            <div className="border-border border-b">
                <TabsList className="mb-3">
                    <TabsTab value="overview" className="h-8 text-[13px]">
                        Overview
                    </TabsTab>
                    <TabsTab value="stats" className="h-8 text-[13px]">
                        Stats
                    </TabsTab>
                    <TabsTab value="skills" className="h-8 text-[13px]">
                        Skills
                    </TabsTab>
                    <TabsTab value="chibi" className="h-8 text-[13px]">
                        Chibi
                    </TabsTab>
                </TabsList>
            </div>
            <div className="pt-5">
                <TabsPanel value="chibi">
                    <div className="rounded-xl border border-border bg-card">
                        <EnemyChibiTab enemyId="enemy_1523_mandra" />
                    </div>
                </TabsPanel>
            </div>
        </Tabs>
    </div>
);
