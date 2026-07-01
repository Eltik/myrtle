import { useSuspenseQuery } from "@tanstack/react-query";
import { Link, useParams } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { useCallback, useMemo, useRef, useState } from "react";
import { enemiesQueryOptions, type IEnemy } from "#/lib/api/enemies";
import type { ILevel } from "#/lib/api/level";
import { materialsQueryOptions } from "#/lib/api/materials";
import { stagesQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import type { IStage, IZone } from "#/types/stages";
import { DropsSection } from "./impl/DropsSection";
import { EnemiesSection } from "./impl/EnemiesSection";
import { buildStageEnemyStats, computeStageEnemyStats, groupDrops, tallyEnemies } from "./impl/helpers";
import { DEFAULT_MAP_SETTINGS, MapSettings } from "./impl/MapSettings";
import { OverviewSection } from "./impl/OverviewSection";
import { PropertiesSection } from "./impl/PropertiesSection";
import { TileLegend } from "./impl/primitives";
import { SpawnSchedule } from "./impl/SpawnSchedule";
import { StageHeader } from "./impl/StageHeader";
import type { IStageEnemyStats } from "./impl/types";
import { WavesSection } from "./impl/WavesSection";
import { type IMapViewHandle, MapView } from "./map";

export function StageDetail({ level, fallbackStage, fallbackZone }: { level: ILevel | null; fallbackStage?: IStage | null; fallbackZone?: IZone | null }) {
    const { stageId } = useParams({ from: "/stages_/$stageId" });
    const { data: stages } = useSuspenseQuery(stagesQueryOptions());
    const { data: zones } = useSuspenseQuery(zonesQueryOptions());
    const { data: handbook } = useSuspenseQuery(enemiesQueryOptions());
    const { data: materials } = useSuspenseQuery(materialsQueryOptions());

    const stage = useMemo(() => stages.find((s) => s.stageId === stageId) ?? fallbackStage ?? null, [stages, stageId, fallbackStage]);
    const zone = useMemo(() => zones.find((z) => z.zoneId === stage?.zoneId) ?? fallbackZone ?? undefined, [zones, stage, fallbackZone]);
    const tally = useMemo(() => tallyEnemies(level, handbook.enemyData), [level, handbook.enemyData]);
    const dropGroups = useMemo(() => (stage ? groupDrops(stage, materials.items) : []), [stage, materials.items]);
    const [mapSettings, setMapSettings] = useState(DEFAULT_MAP_SETTINGS);

    const mapRef = useRef<IMapViewHandle>(null);
    const moveMultiplier = level?.options?.moveMultiplier ?? 1;
    const stageStats = useMemo(() => buildStageEnemyStats(level, handbook.enemyData), [level, handbook.enemyData]);
    const statsFor = useCallback((id: string, enemy: IEnemy | null): IStageEnemyStats | null => stageStats[id] ?? computeStageEnemyStats(enemy, undefined, moveMultiplier), [stageStats, moveMultiplier]);
    const focusEnemy = useCallback((id: string, time?: number) => mapRef.current?.focusSpawn({ enemyId: id, time }), []);

    if (!stage) {
        return (
            <div className="mx-auto w-[min(1100px,calc(100%-2rem))] py-20 text-center">
                <h1 className="m-0 font-bold font-sans text-[22px] text-foreground">Stage not found</h1>
                <p className="mt-2 font-sans text-[13.5px] text-muted-foreground">
                    No stage with id <code className="font-mono text-foreground">{stageId}</code> exists.
                </p>
                <Link to="/stages" className="mt-4 inline-block font-medium font-sans text-[13.5px] text-primary hover:underline">
                    Back to Stages
                </Link>
            </div>
        );
    }

    return (
        <div className="relative z-1 mx-auto w-[min(1600px,calc(100%-2rem))] pb-16">
            <nav className="pt-7 pb-3.5 font-medium font-sans text-[12px] text-muted-foreground leading-none" aria-label="Breadcrumb">
                <ol className="flex items-center gap-1.5">
                    <li>
                        <Link to="/stages" className="transition-colors hover:text-foreground">
                            Stages
                        </Link>
                    </li>
                    <ChevronRight className="h-2.5 w-2.5" aria-hidden="true" />
                    <li className="truncate text-foreground">{stage.code}</li>
                </ol>
            </nav>

            <div className="flex flex-col gap-7">
                <StageHeader stage={stage} zone={zone} />
                <OverviewSection level={level} />
            </div>

            <div className="mt-7 flex flex-col gap-3">
                {level ? (
                    <>
                        <div className="relative overflow-hidden rounded-[14px] border border-border bg-[#181818]">
                            <MapView ref={mapRef} code={stage.code} enemyData={handbook.enemyData} level={level} onSettingsChange={setMapSettings} settings={mapSettings} statsFor={statsFor} />
                        </div>
                        <TileLegend />
                        <MapSettings settings={mapSettings} onChange={setMapSettings} />
                    </>
                ) : (
                    <div className="flex min-h-70 items-center justify-center rounded-[14px] border border-border border-dashed p-10 text-center">
                        <p className="m-0 font-sans text-[13px] text-muted-foreground">No map data available for this stage.</p>
                    </div>
                )}
            </div>

            <div className="mt-7 flex flex-col gap-7">
                <SpawnSchedule level={level} enemyData={handbook.enemyData} onFocusEnemy={focusEnemy} />

                <div className="grid grid-cols-1 gap-x-5 gap-y-6 lg:grid-cols-2 lg:items-start">
                    <div className="flex min-w-0 flex-col gap-6">
                        <DropsSection groups={dropGroups} />
                        <WavesSection level={level} />
                    </div>
                    <div className="flex min-w-0 flex-col gap-6">
                        <EnemiesSection tally={tally} onFocusEnemy={focusEnemy} />
                        <PropertiesSection stage={stage} level={level} />
                    </div>
                </div>
            </div>
        </div>
    );
}
