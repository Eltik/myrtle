import { Tabs, TabsList, TabsPanel, TabsTab } from "frontend";
import { BoxesIcon, SwordsIcon, UserIcon } from "lucide-react";

/** Default variant — a muted pill track with a sliding white indicator. */
export const Segmented = () => (
    <Tabs defaultValue="stats">
        <TabsList>
            <TabsTab className="h-8 text-[13px]" value="overview">
                Overview
            </TabsTab>
            <TabsTab className="h-8 text-[13px]" value="stats">
                Stats
            </TabsTab>
            <TabsTab className="h-8 text-[13px]" value="skills">
                Skills
            </TabsTab>
            <TabsTab className="h-8 text-[13px]" value="chibi">
                Chibi
            </TabsTab>
        </TabsList>
    </Tabs>
);

/** Underline variant — no track, a brand-red rule marks the active tab. */
export const Underline = () => (
    <Tabs className="w-full max-w-xl" defaultValue="modules">
        <div className="border-border border-b">
            <TabsList variant="underline">
                <TabsTab value="overview">Overview</TabsTab>
                <TabsTab value="skills">Skills</TabsTab>
                <TabsTab value="modules">Modules</TabsTab>
                <TabsTab value="riic">Base skills</TabsTab>
            </TabsList>
        </div>
        <TabsPanel className="pt-4 text-muted-foreground text-sm" value="modules">
            GUA-Y · Stage 3 — ATK +26, and Sworn Enemy of Evil no longer consumes charges on kill.
        </TabsPanel>
    </Tabs>
);

/** Vertical, with icons — the operator page's left rail. */
export const Vertical = () => (
    <Tabs defaultValue="skills" orientation="vertical">
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
        </TabsList>
    </Tabs>
);
