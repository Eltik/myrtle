"use client";

import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import type { Operator } from "~/types/api";
import { ElitePromotionTab } from "./impl/elite-promotion-tab";
import { ModulesTab } from "./impl/modules-tab";
import { SkillMasteryTab } from "./impl/skill-mastery-tab";

interface LevelUpContentProps {
    operator: Operator;
}

export function LevelUpContent({ operator }: LevelUpContentProps) {
    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Level-Up Costs</h2>
                <p className="text-muted-foreground text-sm">Materials required for promotions, skill upgrades, and modules</p>
            </div>

            <Tabs className="w-full" defaultValue="elite">
                <TabsList className="mb-6 grid w-full grid-cols-3">
                    <TabsTrigger value="elite">Elite Promotion</TabsTrigger>
                    <TabsTrigger value="skills">Skill Mastery</TabsTrigger>
                    <TabsTrigger value="modules">Modules</TabsTrigger>
                </TabsList>

                <TabsContent value="elite">
                    <ElitePromotionTab operator={operator} />
                </TabsContent>

                <TabsContent value="skills">
                    <SkillMasteryTab operator={operator} />
                </TabsContent>

                <TabsContent value="modules">
                    <ModulesTab operator={operator} />
                </TabsContent>
            </Tabs>
        </div>
    );
}
