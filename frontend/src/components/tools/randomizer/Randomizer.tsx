import { useQuery } from "@tanstack/react-query";
import { ChevronRight } from "lucide-react";
import * as React from "react";
import { useAuth } from "#/hooks/use-auth";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { activitiesQueryOptions, retroActsQueryOptions, stagesQueryOptions, userStageClearsQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import { userRosterQueryOptions } from "#/lib/api/user";
import { buildActivityLookup } from "./impl/activity-lookup";
import { BriefingHero } from "./impl/components/BriefingHero";
import { EmptyState } from "./impl/components/EmptyState";
import { ModifierSlab } from "./impl/components/ModifierSlab";
import { SettingsSheet } from "./impl/components/SettingsSheet";
import { SquadSlab } from "./impl/components/SquadSlab";
import { StageSlab } from "./impl/components/StageSlab";
import { DEFAULT_SETTINGS, SETTINGS_VERSION, STORAGE_KEY_SETTINGS } from "./impl/constants";
import type { IChallenge, IRandomizerOperator, IRandomizerSettings } from "./impl/types";
import { buildRosterIndex, filterPlayableStages, pickRandomChallenge, pickRandomSquad, pickRandomStage, selectAvailableOperators, selectAvailableStages, toRandomizerOperator } from "./impl/utils";

const ROSTER_STORAGE_KEY = "randomizer-roster-v4";

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
    const { data: activities = [] } = useQuery(activitiesQueryOptions());
    const { data: retroActs = [] } = useQuery(retroActsQueryOptions());
    const { data: rosterEntries } = useQuery(userRosterQueryOptions(uid ?? ""));
    const { data: stageClears } = useQuery(userStageClearsQueryOptions(uid));

    const activityLookup = React.useMemo(() => buildActivityLookup(activities, retroActs), [activities, retroActs]);

    const randomizerOperators = React.useMemo<IRandomizerOperator[]>(
        () =>
            operatorsStatic
                .filter((op) => op.profession !== "TOKEN" && op.profession !== "TRAP")
                .map(toRandomizerOperator)
                .filter((op): op is IRandomizerOperator => op !== null),
        [operatorsStatic],
    );

    const playableStages = React.useMemo(() => filterPlayableStages(stages, activityLookup), [stages, activityLookup]);
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

    // Roster selection: `null` means "user hasn't pruned the roster yet" → treat as all
    // operators. Any array (even empty) is an explicit choice the user made.
    const [rosterStored, setRosterStored] = useLocalStorageState<string[] | null>(ROSTER_STORAGE_KEY, null, {
        parse: (raw) => {
            try {
                const parsed = JSON.parse(raw) as unknown;
                if (parsed === null) return null;
                return Array.isArray(parsed) ? (parsed as string[]) : undefined;
            } catch {
                return undefined;
            }
        },
    });
    const effectiveRosterSet = React.useMemo(() => {
        if (rosterStored === null) return new Set(randomizerOperators.map((op) => op.id));
        return new Set(rosterStored);
    }, [rosterStored, randomizerOperators]);
    const setRosterSelection = React.useCallback((next: Set<string>) => setRosterStored(Array.from(next)), [setRosterStored]);
    const resetRosterSelection = React.useCallback(() => setRosterStored(null), [setRosterStored]);

    const availableOperators = React.useMemo(() => {
        const filtered = selectAvailableOperators(randomizerOperators, settings, rosterIndex);
        return filtered.filter((op) => effectiveRosterSet.has(op.id));
    }, [randomizerOperators, settings, rosterIndex, effectiveRosterSet]);

    // The roster picker mirrors every operator-tab constraint, so flipping a setting
    // immediately prunes the roster view to match what the randomizer can actually draw.
    const rosterPickerOperators = React.useMemo(() => selectAvailableOperators(randomizerOperators, settings, rosterIndex), [randomizerOperators, settings, rosterIndex]);

    const availableStages = React.useMemo(() => selectAvailableStages(playableStages, zones, settings, stageClears ?? null, activityLookup), [playableStages, zones, settings, stageClears, activityLookup]);

    /** Full operator entities (skills, modules, etc) for the available pool — fed into challenge filters. */
    const availableOperatorsFull = React.useMemo(() => {
        const allowed = new Set(availableOperators.map((op) => op.id));
        return operatorsStatic.filter((op) => op.id !== null && allowed.has(op.id));
    }, [operatorsStatic, availableOperators]);

    const [rolledStage, setRolledStage] = React.useState<(typeof playableStages)[number] | null>(null);
    const [rolledSquad, setRolledSquad] = React.useState<IRandomizerOperator[]>([]);
    const [rolledChallenge, setRolledChallenge] = React.useState<IChallenge | null>(null);
    const [rollSeq, setRollSeq] = React.useState(0);

    const [settingsOpen, setSettingsOpen] = React.useState(false);

    const hasResult = rolledStage !== null || rolledSquad.length > 0 || rolledChallenge !== null;
    const canRoll = availableOperators.length > 0 && availableStages.length > 0;

    /**
     * Resolve the squad pool given the active challenge: if it's a SQUAD_FILTER,
     * apply the filter to the full operator data, then map back to the slim
     * representation used by the squad picker.
     */
    const resolveSquadPool = React.useCallback(
        (challenge: IChallenge | null): IRandomizerOperator[] => {
            if (!challenge || challenge.type !== "SQUAD_FILTER") return availableOperators;
            const restricted = availableOperatorsFull.filter(challenge.filter);
            const restrictedIds = new Set(restricted.map((op) => op.id));
            return availableOperators.filter((op) => restrictedIds.has(op.id));
        },
        [availableOperators, availableOperatorsFull],
    );

    const rollStage = React.useCallback(() => setRolledStage(pickRandomStage(availableStages)), [availableStages]);
    const rollSquad = React.useCallback(() => {
        const pool = resolveSquadPool(rolledChallenge);
        setRolledSquad(pickRandomSquad(pool, settings.squadSize, settings.allowDuplicates));
    }, [resolveSquadPool, rolledChallenge, settings.squadSize, settings.allowDuplicates]);
    const rollChallenge = React.useCallback(() => {
        if (!rolledStage) return;
        const picked = pickRandomChallenge({ stage: rolledStage, operators: availableOperatorsFull, squadSize: settings.squadSize });
        setRolledChallenge(picked?.challenge ?? null);
        // If the new challenge restricts the pool, reroll the squad to honour it.
        if (picked?.challenge.type === "SQUAD_FILTER") {
            setRolledSquad(pickRandomSquad(resolveSquadPool(picked.challenge), settings.squadSize, settings.allowDuplicates));
        }
    }, [rolledStage, availableOperatorsFull, settings.squadSize, settings.allowDuplicates, resolveSquadPool]);

    const rollAll = React.useCallback(() => {
        const stage = pickRandomStage(availableStages);
        const picked = stage ? pickRandomChallenge({ stage, operators: availableOperatorsFull, squadSize: settings.squadSize }) : null;
        const challenge = picked?.challenge ?? null;
        let squadPool = availableOperators;
        if (picked?.filteredOperators) {
            const allowed = new Set(picked.filteredOperators.map((op) => op.id));
            squadPool = availableOperators.filter((op) => allowed.has(op.id));
        }

        setRolledStage(stage);
        setRolledChallenge(challenge);
        setRolledSquad(pickRandomSquad(squadPool, settings.squadSize, settings.allowDuplicates));
        setRollSeq((n) => n + 1);
    }, [availableStages, availableOperatorsFull, availableOperators, settings.squadSize, settings.allowDuplicates]);

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
                {rolledStage && <StageSlab stage={rolledStage} zone={zoneById.get(rolledStage.zoneId)} lookup={activityLookup} onReroll={rollStage} />}
                {rolledSquad.length > 0 && <SquadSlab operators={rolledSquad} squadSize={settings.squadSize} onReroll={rollSquad} />}
                {rolledChallenge && <ModifierSlab challenge={rolledChallenge} onReroll={rollChallenge} />}
            </div>

            <SettingsSheet
                open={settingsOpen}
                onOpenChange={setSettingsOpen}
                settings={settings}
                onChange={updateSettings}
                allOperators={randomizerOperators}
                rosterPickerOperators={rosterPickerOperators}
                rosterSelection={effectiveRosterSet}
                rosterIsExplicit={rosterStored !== null}
                onRosterChange={setRosterSelection}
                onRosterReset={resetRosterSelection}
                rosterIndex={rosterIndex}
                hasProfile={hasProfile}
                stages={playableStages}
                zones={zones}
                activityLookup={activityLookup}
                stageClears={stageClears ?? null}
            />
        </div>
    );
}
