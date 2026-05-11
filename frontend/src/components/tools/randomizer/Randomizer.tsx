import { useQuery } from "@tanstack/react-query";
import { ChevronRight } from "lucide-react";
import * as React from "react";
import { useAuth } from "#/hooks/use-auth";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { stagesQueryOptions, userStageClearsQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import { userRosterQueryOptions } from "#/lib/api/user";
import { BriefingHero } from "./impl/components/BriefingHero";
import { EmptyState } from "./impl/components/EmptyState";
import { ModifierSlab } from "./impl/components/ModifierSlab";
import { SettingsSheet } from "./impl/components/SettingsSheet";
import { SquadSlab } from "./impl/components/SquadSlab";
import { StageSlab } from "./impl/components/StageSlab";
import { DEFAULT_SETTINGS, SETTINGS_VERSION, STORAGE_KEY_SETTINGS } from "./impl/constants";
import type { IChallenge, IRandomizerOperator, IRandomizerSettings } from "./impl/types";
import { buildRosterIndex, filterPlayableStages, pickRandomChallenge, pickRandomSquad, pickRandomStage, selectAvailableOperators, selectAvailableStages, toRandomizerOperator } from "./impl/utils";

const ROSTER_STORAGE_KEY = "randomizer-roster-v3";

interface IPersistedSettings extends IRandomizerSettings {
    _version?: number;
}

function migrateSettings(saved: IPersistedSettings): IRandomizerSettings {
    const { _version: _, ...rest } = saved;
    return { ...DEFAULT_SETTINGS, ...rest } satisfies IRandomizerSettings;
}

export function Randomizer(): React.ReactElement {
    const { user, isAuthenticated } = useAuth();
    const hasProfile = isAuthenticated;
    const uid = user?.uid ?? null;

    const { data: operatorsStatic = [] } = useQuery(operatorsListQueryOptions());
    const { data: stages = [] } = useQuery(stagesQueryOptions());
    const { data: zones = [] } = useQuery(zonesQueryOptions());
    const { data: rosterEntries } = useQuery(userRosterQueryOptions(uid ?? ""));
    const { data: stageClears } = useQuery(userStageClearsQueryOptions(uid));

    const randomizerOperators = React.useMemo<IRandomizerOperator[]>(
        () =>
            operatorsStatic
                .filter((op) => op.profession !== "TOKEN" && op.profession !== "TRAP")
                .map(toRandomizerOperator)
                .filter((op): op is IRandomizerOperator => op !== null),
        [operatorsStatic],
    );

    const playableStages = React.useMemo(() => filterPlayableStages(stages), [stages]);
    const rosterIndex = React.useMemo(() => buildRosterIndex(hasProfile ? rosterEntries : null), [hasProfile, rosterEntries]);
    const zoneById = React.useMemo(() => new Map(zones.map((z) => [z.zoneId, z])), [zones]);

    const [persisted, setPersisted] = useLocalStorageState<IPersistedSettings>(
        STORAGE_KEY_SETTINGS,
        { ...DEFAULT_SETTINGS, _version: SETTINGS_VERSION },
        {
            parse: (raw) => {
                try {
                    return JSON.parse(raw) as IPersistedSettings;
                } catch {
                    return undefined;
                }
            },
        },
    );
    const settings = React.useMemo(() => migrateSettings(persisted), [persisted]);
    const updateSettings = React.useCallback((next: Partial<IRandomizerSettings>) => setPersisted((prev) => ({ ...migrateSettings(prev), ...next, _version: SETTINGS_VERSION })), [setPersisted]);

    // Roster selection — defaults to "all known operators" until the user prunes it.
    const [rosterSelection, setRosterSelection] = useLocalStorageState<string[]>(ROSTER_STORAGE_KEY, []);
    const rosterSet = React.useMemo(() => new Set(rosterSelection), [rosterSelection]);
    const effectiveRosterSet = React.useMemo(() => (rosterSelection.length === 0 ? new Set(randomizerOperators.map((op) => op.id)) : rosterSet), [rosterSelection, rosterSet, randomizerOperators]);

    const availableOperators = React.useMemo(() => {
        const filtered = selectAvailableOperators(randomizerOperators, settings, rosterIndex);
        return filtered.filter((op) => effectiveRosterSet.has(op.id));
    }, [randomizerOperators, settings, rosterIndex, effectiveRosterSet]);

    const availableStages = React.useMemo(() => selectAvailableStages(playableStages, zones, settings, stageClears ?? null), [playableStages, zones, settings, stageClears]);

    const [rolledStage, setRolledStage] = React.useState<(typeof playableStages)[number] | null>(null);
    const [rolledSquad, setRolledSquad] = React.useState<IRandomizerOperator[]>([]);
    const [rolledChallenge, setRolledChallenge] = React.useState<IChallenge | null>(null);
    const [rollSeq, setRollSeq] = React.useState(0);

    const [settingsOpen, setSettingsOpen] = React.useState(false);

    const hasResult = rolledStage !== null || rolledSquad.length > 0 || rolledChallenge !== null;
    const canRoll = availableOperators.length > 0 && availableStages.length > 0;

    const rollStage = React.useCallback(() => setRolledStage(pickRandomStage(availableStages)), [availableStages]);
    const rollSquad = React.useCallback(() => setRolledSquad(pickRandomSquad(availableOperators, settings.squadSize, settings.allowDuplicates)), [availableOperators, settings.squadSize, settings.allowDuplicates]);
    const rollChallenge = React.useCallback(() => setRolledChallenge(pickRandomChallenge()), []);

    const rollAll = React.useCallback(() => {
        setRolledStage(pickRandomStage(availableStages));
        setRolledSquad(pickRandomSquad(availableOperators, settings.squadSize, settings.allowDuplicates));
        setRolledChallenge(pickRandomChallenge());
        setRollSeq((n) => n + 1);
    }, [availableStages, availableOperators, settings.squadSize, settings.allowDuplicates]);

    const reset = React.useCallback(() => {
        setRolledStage(null);
        setRolledSquad([]);
        setRolledChallenge(null);
    }, []);

    return (
        <div className="relative z-1 mx-auto w-[min(1320px,calc(100%-2rem))] py-5 pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] leading-none text-muted-foreground">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">Randomizer</span>
            </nav>

            <BriefingHero operatorsAvailable={availableOperators.length} operatorsRoster={effectiveRosterSet.size} stagesAvailable={availableStages.length} hasResult={hasResult} canRoll={canRoll} onRollAll={rollAll} onReset={reset} onOpenSettings={() => setSettingsOpen(true)} />

            <div className="mt-6 flex flex-col gap-3 sm:gap-4" key={rollSeq}>
                {!hasResult && <EmptyState />}
                {rolledStage && <StageSlab stage={rolledStage} zone={zoneById.get(rolledStage.zoneId)} onReroll={rollStage} />}
                {rolledSquad.length > 0 && <SquadSlab operators={rolledSquad} squadSize={settings.squadSize} onReroll={rollSquad} />}
                {rolledChallenge && <ModifierSlab challenge={rolledChallenge} onReroll={rollChallenge} />}
            </div>

            <SettingsSheet open={settingsOpen} onOpenChange={setSettingsOpen} settings={settings} onChange={updateSettings} operators={randomizerOperators} rosterSelection={effectiveRosterSet} onRosterChange={(next) => setRosterSelection(Array.from(next))} rosterIndex={rosterIndex} hasProfile={hasProfile} />
        </div>
    );
}
