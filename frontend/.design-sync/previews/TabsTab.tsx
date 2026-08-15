import { Badge, Tabs, TabsList, TabsPanel, TabsTab } from "frontend";
import { BoxesIcon, ImageIcon, SwordsIcon, UserIcon } from "lucide-react";

/** Tabs take an icon plus a label; the icon is auto-sized by the tab's own rules. */
export const WithIcons = () => (
    <Tabs className="w-full max-w-xl" defaultValue="skills">
        <TabsList>
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
                <ImageIcon />
                Art
            </TabsTab>
        </TabsList>
        <TabsPanel className="pt-4 text-muted-foreground text-sm" value="skills">
            Sworn Enemy of Evil · M3 — attack interval −0.3s, attack power +160% for 20 seconds.
        </TabsPanel>
    </Tabs>
);

/** A disabled tab: CN-only content the Global server has not received. */
export const DisabledTab = () => (
    <Tabs className="w-full max-w-xl" defaultValue="modules">
        <TabsList>
            <TabsTab value="overview">Overview</TabsTab>
            <TabsTab value="modules">Modules</TabsTab>
            <TabsTab disabled value="paradox">
                Paradox Simulation
            </TabsTab>
            <TabsTab disabled value="voice">
                Voice lines
            </TabsTab>
        </TabsList>
        <TabsPanel className="pt-4 text-muted-foreground text-sm" value="modules">
            Two modules unlocked at E2 40. Paradox Simulation has not reached the EN server yet.
        </TabsPanel>
    </Tabs>
);

/** Trailing counts inside the tab, the shape the tier-list browser uses. */
export const WithCounts = () => (
    <Tabs className="w-full max-w-xl" defaultValue="mine">
        <div className="border-border border-b">
            <TabsList variant="underline">
                <TabsTab value="all">
                    All lists
                    <Badge size="sm" variant="secondary">
                        312
                    </Badge>
                </TabsTab>
                <TabsTab value="mine">
                    My lists
                    <Badge size="sm" variant="secondary">
                        14
                    </Badge>
                </TabsTab>
                <TabsTab value="official">
                    Official
                    <Badge size="sm" variant="secondary">
                        6
                    </Badge>
                </TabsTab>
            </TabsList>
        </div>
        <TabsPanel className="pt-4 text-muted-foreground text-sm" value="mine">
            14 tier lists, 3 of them published. Last edited "6★ Guards by Chapter 8 clear speed" 2 days ago.
        </TabsPanel>
    </Tabs>
);
