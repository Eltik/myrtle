import { AppearsInTab, Tabs, TabsList, TabsPanel, TabsTab } from "frontend";

// AppearsInTab fans out two queries (the enemy's stage refs and the zone table)
// through server functions the design bundle stubs, so it settles into its
// "nothing indexed for this enemy" state — the same card a Doctor sees for an
// enemy whose level files haven't been extracted yet.
export const NoAppearancesRecorded = () => (
    <div className="w-full max-w-2xl">
        <AppearsInTab enemyId="enemy_1500_skulsr" />
    </div>
);

export const WithinDetailTabs = () => (
    <div className="w-full max-w-2xl">
        <Tabs defaultValue="appears" className="flex flex-col gap-0">
            <div className="border-border border-b">
                <TabsList className="mb-3">
                    <TabsTab value="overview" className="h-8 text-[13px]">
                        Overview
                    </TabsTab>
                    <TabsTab value="stats" className="h-8 text-[13px]">
                        Stats
                    </TabsTab>
                    <TabsTab value="appears" className="h-8 text-[13px]">
                        Appears In
                    </TabsTab>
                    <TabsTab value="chibi" className="h-8 text-[13px]">
                        Chibi
                    </TabsTab>
                </TabsList>
            </div>
            <div className="pt-5">
                <TabsPanel value="appears">
                    <AppearsInTab enemyId="enemy_1007_slime" />
                </TabsPanel>
            </div>
        </Tabs>
    </div>
);
