import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { PageHeader } from "#/components/ui/page-header";
import { useAuth } from "#/hooks/use-auth";
import { useLocalStorageState } from "#/hooks/use-local-storage-state";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { activitiesQueryOptions, retroActsQueryOptions, stagesQueryOptions, userStageClearsQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import { userRosterQueryOptions } from "#/lib/api/user";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IStage } from "#/types/stages";
import { buildActivityLookup } from "./impl/activity-lookup";
import { BriefingHero } from "./impl/components/BriefingHero";
import { EmptyState } from "./impl/components/EmptyState";
import { ModifierSlab } from "./impl/components/ModifierSlab";
import { SettingsSheet } from "./impl/components/SettingsSheet";
import { SquadSlab } from "./impl/components/SquadSlab";
import { StageSlab } from "./impl/components/StageSlab";
import { DEFAULT_SETTINGS, SETTINGS_VERSION, STORAGE_KEY_SETTINGS } from "./impl/constants";
import type { IChallenge, IRandomizerOperator, IRandomizerSettings } from "./impl/types";
import { applyProfileGate, buildRosterIndex, filterPlayableStages, pickRandomChallenge, pickRandomSquad, pickRandomStage, selectAvailableOperators, selectAvailableStages, toRandomizerOperator } from "./impl/utils";
import type { messages } from "./Randomizer.messages";

const ROSTER_STORAGE_KEY = "randomizer-roster-v4";

interface IPersistedSettings extends IRandomizerSettings {
    _version?: number;
}

/** Fills any setting added since the save with its default, and drops the version stamp. */
function migrateSettings(saved: IPersistedSettings): IRandomizerSettings {
    const { _version: _, ...rest } = saved;
    return { ...DEFAULT_SETTINGS, ...rest } satisfies IRandomizerSettings;
}

/**
 * `null` is a stored value of its own ("never pruned"), so it round-trips;
 * anything else that is not an array is malformed and keeps the initial value.
 * The hook already keeps the initial value when `JSON.parse` throws.
 */
function parseRosterSelection(raw: string): string[] | null | undefined {
    const parsed = JSON.parse(raw) as unknown;
    if (parsed === null) return null;
    return Array.isArray(parsed) ? (parsed as string[]) : undefined;
}

export function Randomizer(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("tools");
    const { user, isAuthenticated } = useAuth();
    const hasProfile = isAuthenticated;
    const uid = user?.uid ?? null;

    const server = useGamedataServer();
    const { data: operatorsIndex = [] } = useQuery(operatorsIndexQueryOptions(server));
    const { data: stages = [] } = useQuery(stagesQueryOptions(server));
    const { data: zones = [] } = useQuery(zonesQueryOptions(server));
    const { data: activities = [] } = useQuery(activitiesQueryOptions(server));
    const { data: retroActs = [] } = useQuery(retroActsQueryOptions(server));
    const { data: rosterEntries } = useQuery({ ...userRosterQueryOptions(uid ?? ""), enabled: !!uid });
    const { data: stageClears } = useQuery(userStageClearsQueryOptions(uid));

    const activityLookup = React.useMemo(() => buildActivityLookup(activities, retroActs), [activities, retroActs]);

    const randomizerOperators = React.useMemo<IRandomizerOperator[]>(
        () =>
            operatorsIndex
                .filter((op) => op.profession !== "TOKEN" && op.profession !== "TRAP")
                .map(toRandomizerOperator)
                .filter((op): op is IRandomizerOperator => op !== null),
        [operatorsIndex],
    );

    const playableStages = React.useMemo(() => filterPlayableStages(stages, activityLookup), [stages, activityLookup]);
    const rosterIndex = React.useMemo(() => buildRosterIndex(hasProfile ? rosterEntries : null), [hasProfile, rosterEntries]);
    const zoneById = React.useMemo(() => new Map(zones.map((z) => [z.zoneId, z])), [zones]);

    const [persisted, setPersisted] = useLocalStorageState<IPersistedSettings>(STORAGE_KEY_SETTINGS, { ...DEFAULT_SETTINGS, _version: SETTINGS_VERSION });
    const hasStageClears = stageClears != null;
    const settings = React.useMemo(() => applyProfileGate(migrateSettings(persisted), { hasProfile, hasStageClears }), [persisted, hasProfile, hasStageClears]);
    const updateSettings = React.useCallback((next: Partial<IRandomizerSettings>) => setPersisted((prev) => ({ ...migrateSettings(prev), ...next, _version: SETTINGS_VERSION })), [setPersisted]);

    // Roster selection: `null` means "user hasn't pruned the roster yet" -> treat as all
    // operators. Any array (even empty) is an explicit choice the user made.
    const [rosterStored, setRosterStored] = useLocalStorageState<string[] | null>(ROSTER_STORAGE_KEY, null, { parse: parseRosterSelection });
    const effectiveRosterSet = React.useMemo(() => {
        if (rosterStored === null) return new Set(randomizerOperators.map((op) => op.id));
        return new Set(rosterStored);
    }, [rosterStored, randomizerOperators]);
    const setRosterSelection = React.useCallback((next: Set<string>) => setRosterStored(Array.from(next)), [setRosterStored]);
    const resetRosterSelection = React.useCallback(() => setRosterStored(null), [setRosterStored]);

    // The roster picker mirrors every operator-tab constraint, so flipping a setting
    // immediately prunes the roster view to match what the randomizer can actually draw.
    const rosterPickerOperators = React.useMemo(() => selectAvailableOperators(randomizerOperators, settings, rosterIndex), [randomizerOperators, settings, rosterIndex]);
    const availableOperators = React.useMemo(() => rosterPickerOperators.filter((op) => effectiveRosterSet.has(op.id)), [rosterPickerOperators, effectiveRosterSet]);

    const availableStages = React.useMemo(() => selectAvailableStages(playableStages, zones, settings, stageClears ?? null, activityLookup), [playableStages, zones, settings, stageClears, activityLookup]);

    const [rolledStage, setRolledStage] = React.useState<IStage | null>(null);
    const [rolledSquad, setRolledSquad] = React.useState<IRandomizerOperator[]>([]);
    const [rolledChallenge, setRolledChallenge] = React.useState<IChallenge | null>(null);
    const [rollSeq, setRollSeq] = React.useState(0);

    const [settingsOpen, setSettingsOpen] = React.useState(false);

    const hasResult = rolledStage !== null || rolledSquad.length > 0 || rolledChallenge !== null;
    const canRoll = availableOperators.length > 0 && availableStages.length > 0;

    /** The squad pool under `challenge`: a SQUAD_FILTER narrows the available operators. */
    const resolveSquadPool = React.useCallback(
        (challenge: IChallenge | null): IRandomizerOperator[] => {
            if (!challenge || challenge.type !== "SQUAD_FILTER") return availableOperators;
            return availableOperators.filter(challenge.filter);
        },
        [availableOperators],
    );

    const rollStage = React.useCallback(() => setRolledStage(pickRandomStage(availableStages)), [availableStages]);
    const rollSquad = React.useCallback(() => {
        const pool = resolveSquadPool(rolledChallenge);
        setRolledSquad(pickRandomSquad(pool, settings.squadSize, settings.allowDuplicates));
    }, [resolveSquadPool, rolledChallenge, settings.squadSize, settings.allowDuplicates]);
    const rollChallenge = React.useCallback(() => {
        if (!rolledStage) return;
        const picked = pickRandomChallenge({ stage: rolledStage, operators: availableOperators, squadSize: settings.squadSize });
        setRolledChallenge(picked?.challenge ?? null);
        // If the new challenge restricts the pool, reroll the squad to honour it.
        if (picked?.challenge.type === "SQUAD_FILTER") {
            setRolledSquad(pickRandomSquad(resolveSquadPool(picked.challenge), settings.squadSize, settings.allowDuplicates));
        }
    }, [rolledStage, availableOperators, settings.squadSize, settings.allowDuplicates, resolveSquadPool]);

    const rollAll = React.useCallback(() => {
        const stage = pickRandomStage(availableStages);
        const picked = stage ? pickRandomChallenge({ stage, operators: availableOperators, squadSize: settings.squadSize }) : null;
        const challenge = picked?.challenge ?? null;
        const squadPool = picked?.filteredOperators ?? availableOperators;

        setRolledStage(stage);
        setRolledChallenge(challenge);
        setRolledSquad(pickRandomSquad(squadPool, settings.squadSize, settings.allowDuplicates));
        setRollSeq((n) => n + 1);
    }, [availableStages, availableOperators, settings.squadSize, settings.allowDuplicates]);

    const reset = React.useCallback(() => {
        setRolledStage(null);
        setRolledSquad([]);
        setRolledChallenge(null);
    }, []);

    return (
        <div className="page-shell [--page-max:1320px]">
            <PageHeader breadcrumbLabel="breadcrumb" breadcrumb={[t("randomizer.breadcrumb.tools"), t("randomizer.breadcrumb.title")]} title={t("randomizer.breadcrumb.title")} className="mb-5" />

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
