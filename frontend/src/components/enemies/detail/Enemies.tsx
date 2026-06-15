import { useSuspenseQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { useMemo } from "react";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { enemiesQueryOptions } from "#/lib/api/enemies";
import { EnemyChibiTab } from "../list/impl/components/EnemyChibi";
import { enrichEnemy } from "../list/impl/enrich";
import { AppearsInTab } from "./impl/AppearsInTab";
import { EnemyHero, OverviewTab, SkillsTab, StatsTab } from "./impl/sections";

export function EnemyDetail() {
    const { id } = useParams({ from: "/enemies_/$id" });
    const { data: handbook } = useSuspenseQuery(enemiesQueryOptions());

    const enemy = useMemo(() => {
        const raw = handbook.enemyData[id];
        return raw ? enrichEnemy(raw, handbook.raceData) : null;
    }, [handbook, id]);

    if (!enemy) {
        return (
            <div className="mx-auto w-[min(1400px,calc(100%-2rem))] py-20 text-center">
                <h1 className="m-0 font-bold font-sans text-[22px] text-foreground">Enemy not found</h1>
                <p className="mt-2 font-sans text-[13.5px] text-muted-foreground">
                    No enemy with id <code className="font-mono text-foreground">{id}</code> exists in the handbook.
                </p>
                <Link to="/enemies" className="mt-4 inline-block font-medium font-sans text-[13.5px] text-primary hover:underline">
                    Back to Enemy Database
                </Link>
            </div>
        );
    }

    return (
        <div className="relative z-1 mx-auto w-[min(1100px,calc(100%-2rem))] pb-20">
            <nav className="pt-7 pb-3.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label="Breadcrumb">
                <ol className="flex items-center gap-1.5">
                    <li>Collection</li>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <li>
                        <Link to="/enemies" className="transition-colors hover:text-foreground">
                            Enemies
                        </Link>
                    </li>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <li className="truncate text-foreground">{enemy.name}</li>
                </ol>
            </nav>

            <EnemyHero enemy={enemy} />

            <Tabs defaultValue="overview" className="mt-5 flex flex-col gap-0">
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
                        <TabsTab value="appears" className="h-8 text-[13px]">
                            Appears In
                        </TabsTab>
                        <TabsTab value="chibi" className="h-8 text-[13px]">
                            Chibi
                        </TabsTab>
                    </TabsList>
                </div>
                <div className="pt-5">
                    <TabsPanel value="overview">
                        <OverviewTab enemy={enemy} />
                    </TabsPanel>
                    <TabsPanel value="stats">
                        <StatsTab enemy={enemy} />
                    </TabsPanel>
                    <TabsPanel value="skills">
                        <SkillsTab enemy={enemy} />
                    </TabsPanel>
                    <TabsPanel value="appears">
                        <AppearsInTab enemyId={enemy.enemyId} />
                    </TabsPanel>
                    <TabsPanel value="chibi">
                        <EnemyChibiTab key={enemy.enemyId} enemyId={enemy.enemyId} />
                    </TabsPanel>
                </div>
            </Tabs>
        </div>
    );
}
