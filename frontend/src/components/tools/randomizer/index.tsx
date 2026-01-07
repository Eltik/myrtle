"use client";

import { Dices, RotateCcw, Shuffle } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import { Button } from "~/components/ui/shadcn/button";
import { useAuth } from "~/hooks/use-auth";
import type { OperatorPosition, OperatorProfession, OperatorRarity } from "~/types/api/impl/operator";
import type { Stage } from "~/types/api/impl/stage";
import type { User } from "~/types/api/impl/user";
import type { Zone } from "~/types/api/impl/zone";
import { ChallengeDisplay } from "./impl/challenge-display";
import { FilterPanel } from "./impl/filter-panel";
import { RosterPanel } from "./impl/roster-panel";
import { SquadDisplay } from "./impl/squad-display";
import { StageDisplay } from "./impl/stage-display";
import type { Challenge, RandomizerSettings } from "./impl/types";
import { generateChallenge, generateRandomSquad, selectRandomStage } from "./impl/utils";
import { ZoneFilterPanel } from "./impl/zone-filter-panel";

export interface RandomizerOperator {
    id: string | null;
    name: string;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
    portrait: string;
}

interface RandomizerProps {
    zones: Zone[];
    stages: Stage[];
    operators: RandomizerOperator[];
}

const STORAGE_KEY_ROSTER = "randomizer-roster";
const STORAGE_KEY_SETTINGS = "randomizer-settings";
const SETTINGS_VERSION = 4;

const DEFAULT_SETTINGS: RandomizerSettings = {
    allowedClasses: ["WARRIOR", "SNIPER", "TANK", "MEDIC", "SUPPORT", "CASTER", "SPECIAL", "PIONEER"],
    allowedRarities: [6, 5, 4, 3, 2, 1],
    allowedZoneTypes: ["MAINLINE", "ACTIVITY"],
    squadSize: 12,
    allowDuplicates: false,
    onlyCompletedStages: true,
    onlyAvailableStages: true,
    onlyE2Operators: false,
    selectedStages: [],
};

function migrateSettings(saved: RandomizerSettings & { _version?: number }): RandomizerSettings {
    const version = saved._version ?? 1;

    if (version < 2) {
        saved.onlyE2Operators = false;
        saved.onlyCompletedStages = true;
    }

    if (version < 3) {
        saved.allowedZoneTypes = ["MAINLINE", "ACTIVITY"];
        saved.selectedStages = [];
    }

    if (version < 4) {
        saved.onlyAvailableStages = true;
    }

    const { _version: _, ...settings } = saved;
    return settings as RandomizerSettings;
}

function isFullUser(user: unknown): user is User {
    return user !== null && typeof user === "object" && "troop" in user && user.troop !== null && typeof user.troop === "object" && "chars" in user.troop;
}

export function Randomizer({ zones, stages, operators }: RandomizerProps) {
    const { user, loading: authLoading } = useAuth();

    const [roster, setRoster] = useState<Set<string>>(() => new Set(operators.map((op) => op.id).filter((id): id is string => id !== null)));

    const [settings, setSettings] = useState<RandomizerSettings>(DEFAULT_SETTINGS);

    const [isHydrated, setIsHydrated] = useState(false);

    useEffect(() => {
        if (typeof window !== "undefined") {
            const savedRoster = localStorage.getItem(STORAGE_KEY_ROSTER);
            if (savedRoster) {
                try {
                    const parsed = JSON.parse(savedRoster) as string[];
                    setRoster(new Set(parsed));
                } catch {
                    // Keep default roster if parsing fails
                }
            }

            const savedSettings = localStorage.getItem(STORAGE_KEY_SETTINGS);
            if (savedSettings) {
                try {
                    const parsed = JSON.parse(savedSettings) as RandomizerSettings & { _version?: number };
                    const migrated = migrateSettings(parsed);
                    setSettings(migrated);
                } catch {
                    // Keep default settings if parsing fails
                }
            }

            setIsHydrated(true);
        }
    }, []);

    const [randomizedStage, setRandomizedStage] = useState<Stage | null>(null);
    const [randomizedSquad, setRandomizedSquad] = useState<RandomizerOperator[]>([]);
    const [randomizedChallenge, setRandomizedChallenge] = useState<Challenge | null>(null);

    useEffect(() => {
        if (isHydrated) {
            localStorage.setItem(STORAGE_KEY_ROSTER, JSON.stringify(Array.from(roster)));
        }
    }, [roster, isHydrated]);

    useEffect(() => {
        if (isHydrated) {
            localStorage.setItem(STORAGE_KEY_SETTINGS, JSON.stringify({ ...settings, _version: SETTINGS_VERSION }));
        }
    }, [settings, isHydrated]);

    const fullUser = useMemo(() => {
        return isFullUser(user) ? user : null;
    }, [user]);

    const hasProfile = !authLoading && fullUser !== null;

    const availableOperators = useMemo(() => {
        return operators.filter((op) => {
            if (!op.id || !roster.has(op.id)) return false;

            const rarityNum = Number.parseInt(op.rarity.replace("TIER_", ""), 10);
            if (!settings.allowedRarities.includes(rarityNum)) return false;

            if (!settings.allowedClasses.includes(op.profession)) return false;

            if (settings.onlyE2Operators && fullUser) {
                const charData = Object.values(fullUser.troop.chars).find((char) => char.charId === op.id);
                if (!charData || charData.evolvePhase < 2) return false;
            }

            return true;
        });
    }, [operators, roster, settings, fullUser]);

    useEffect(() => {
        if (!settings.selectedStages || settings.selectedStages.length === 0) return;

        const availableStageIds = new Set(
            stages
                .filter((stage) => {
                    const zone = zones.find((z) => z.zoneId === stage.zoneId);
                    if (!zone || !settings.allowedZoneTypes.includes(zone.type)) return false;

                    if (settings.onlyCompletedStages && fullUser) {
                        const stageData = fullUser.dungeon.stages[stage.stageId];
                        return stageData && stageData.completeTimes > 0;
                    }

                    return true;
                })
                .map((s) => s.stageId),
        );

        const validSelectedStages = settings.selectedStages.filter((id) => availableStageIds.has(id));

        if (validSelectedStages.length !== settings.selectedStages.length) {
            setSettings((prev) => ({ ...prev, selectedStages: validSelectedStages }));
        }
    }, [zones, settings.selectedStages, settings.allowedZoneTypes, settings.onlyCompletedStages, fullUser, stages]);

    const handleFiltersChanged = useCallback(() => {
        const validOperatorIds = new Set(availableOperators.map((op) => op.id).filter((id): id is string => id !== null));
        const newRoster = new Set(Array.from(roster).filter((id) => validOperatorIds.has(id)));

        if (newRoster.size !== roster.size) {
            setRoster(newRoster);
        }
    }, [availableOperators, roster]);

    const handleRandomizeAll = useCallback(() => {
        const stage = selectRandomStage(stages, zones, settings.allowedZoneTypes, fullUser, settings.onlyCompletedStages, settings.selectedStages, settings.onlyAvailableStages);
        setRandomizedStage(stage);

        const squad = generateRandomSquad(availableOperators, settings.squadSize, settings.allowDuplicates);
        setRandomizedSquad(squad);

        const challenge = generateChallenge();
        setRandomizedChallenge(challenge);
    }, [stages, zones, availableOperators, settings, fullUser]);

    const handleRandomizeStage = useCallback(() => {
        const stage = selectRandomStage(stages, zones, settings.allowedZoneTypes, fullUser, settings.onlyCompletedStages, settings.selectedStages, settings.onlyAvailableStages);
        setRandomizedStage(stage);
    }, [stages, zones, settings.allowedZoneTypes, settings.onlyCompletedStages, settings.selectedStages, settings.onlyAvailableStages, fullUser]);

    const handleRandomizeSquad = useCallback(() => {
        const squad = generateRandomSquad(availableOperators, settings.squadSize, settings.allowDuplicates);
        setRandomizedSquad(squad);
    }, [availableOperators, settings.squadSize, settings.allowDuplicates]);

    const handleRandomizeChallenge = useCallback(() => {
        const challenge = generateChallenge();
        setRandomizedChallenge(challenge);
    }, []);

    const handleReset = useCallback(() => {
        setRandomizedStage(null);
        setRandomizedSquad([]);
        setRandomizedChallenge(null);
    }, []);

    const getZoneName = useCallback(
        (zoneId: string) => {
            const zone = zones.find((z) => z.zoneId === zoneId);
            return zone?.zoneNameFirst ?? zone?.zoneNameSecond ?? zoneId;
        },
        [zones],
    );

    const handleImportProfile = useCallback(() => {
        if (!fullUser) {
            toast.error("No profile data available. Please log in.");
            return;
        }

        const userOperatorIds = new Set(
            Object.values(fullUser.troop.chars)
                .map((char) => char.charId)
                .filter((id): id is string => id !== null && id !== undefined),
        );

        const matchedOperators = operators
            .filter((op) => op.id && userOperatorIds.has(op.id))
            .map((op) => op.id)
            .filter((id): id is string => id !== null);

        if (matchedOperators.length === 0) {
            toast.error("No matching operators found in your profile");
            return;
        }

        setRoster(new Set(matchedOperators));
        setSettings((prev) => ({ ...prev, onlyE2Operators: false }));
        toast.success(`Imported ${matchedOperators.length} operators from your profile`);
    }, [fullUser, operators]);

    const handleE2Enabled = useCallback(() => {
        if (!fullUser) return;

        const e2OperatorIds = new Set(
            Object.values(fullUser.troop.chars)
                .filter((char) => char.evolvePhase >= 2)
                .map((char) => char.charId)
                .filter((id): id is string => id !== null && id !== undefined),
        );

        const filteredRoster = new Set(Array.from(roster).filter((id) => e2OperatorIds.has(id)));

        if (filteredRoster.size !== roster.size) {
            setRoster(filteredRoster);
            toast.success(`Filtered to ${filteredRoster.size} E2 operators`);
        }
    }, [fullUser, roster]);

    return (
        <div className="space-y-8">
            <div className="space-y-3">
                <div className="flex items-center gap-3">
                    <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-linear-to-br from-primary/20 to-primary/10 shadow-lg shadow-primary/10">
                        <Dices className="h-6 w-6 text-primary" />
                    </div>
                    <div>
                        <h1 className="font-bold text-3xl text-foreground tracking-tight md:text-4xl">Randomizer</h1>
                        <p className="mt-1 text-muted-foreground text-sm">Create unique challenge runs with randomized stages, operators, and modifiers</p>
                    </div>
                </div>
            </div>

            <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex flex-wrap items-center gap-3">
                    <Button className="gap-2 shadow-lg shadow-primary/20 transition-all hover:scale-105 hover:shadow-primary/30 hover:shadow-xl" disabled={availableOperators.length === 0 || stages.length === 0} onClick={handleRandomizeAll} size="lg">
                        <Shuffle className="h-5 w-5" />
                        Randomize All
                    </Button>
                    {(randomizedStage || randomizedSquad.length > 0 || randomizedChallenge) && (
                        <Button className="gap-2 bg-transparent" onClick={handleReset} size="lg" variant="outline">
                            <RotateCcw className="h-4 w-4" />
                            Reset
                        </Button>
                    )}
                </div>
                <div className="flex flex-wrap gap-3 text-muted-foreground text-sm">
                    <div className="rounded-lg bg-secondary/50 px-3 py-1.5">
                        <span className="font-medium text-foreground">{roster.size}</span> / {operators.length} operators
                    </div>
                    {settings.selectedStages?.length > 0 && (
                        <div className="rounded-lg bg-secondary/50 px-3 py-1.5">
                            <span className="font-medium text-foreground">{settings.selectedStages.length}</span> stages selected
                        </div>
                    )}
                </div>
            </div>

            {(randomizedStage || randomizedSquad.length > 0 || randomizedChallenge) && (
                <div className="space-y-4">
                    {randomizedStage && <StageDisplay getZoneName={getZoneName} onRandomize={handleRandomizeStage} stage={randomizedStage} />}

                    {randomizedSquad.length > 0 && <SquadDisplay onRandomize={handleRandomizeSquad} operators={randomizedSquad} squadSize={settings.squadSize} />}

                    {randomizedChallenge && <ChallengeDisplay challenge={randomizedChallenge} onRandomize={handleRandomizeChallenge} />}
                </div>
            )}

            <div className="grid min-w-0 gap-6 lg:grid-cols-2 xl:gap-8">
                <RosterPanel hasProfile={hasProfile} onImportProfile={handleImportProfile} operators={operators} roster={roster} setRoster={setRoster} />

                <div className="space-y-6">
                    <FilterPanel hasProfile={hasProfile} onE2Disabled={handleImportProfile} onE2Enabled={handleE2Enabled} onFiltersChanged={handleFiltersChanged} setSettings={setSettings} settings={settings} />

                    <ZoneFilterPanel
                        allowedZoneTypes={settings.allowedZoneTypes}
                        hasProfile={hasProfile}
                        onlyAvailableStages={settings.onlyAvailableStages}
                        onlyCompletedStages={settings.onlyCompletedStages}
                        selectedStages={settings.selectedStages}
                        setAllowedZoneTypes={(types) => setSettings({ ...settings, allowedZoneTypes: types })}
                        setOnlyAvailableStages={(value) => setSettings({ ...settings, onlyAvailableStages: value })}
                        setOnlyCompletedStages={(value) => setSettings({ ...settings, onlyCompletedStages: value })}
                        setSelectedStages={(stages) => setSettings({ ...settings, selectedStages: stages })}
                        stages={stages}
                        user={fullUser}
                        zones={zones}
                    />
                </div>
            </div>
        </div>
    );
}
