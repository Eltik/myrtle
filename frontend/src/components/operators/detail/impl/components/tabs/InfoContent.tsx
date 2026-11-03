import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Dna, Heart, Info, LibraryBig, type LucideIcon, MapPin, Package, Palette, User } from "lucide-react";
import { memo, type ReactNode, useEffect, useId, useMemo, useRef, useState } from "react";
import { useOperatorFactLabel } from "#/components/operators/OperatorFacts";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { LoadFailed } from "#/components/ui/load-failed";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Separator } from "#/components/ui/separator";
import { Skeleton } from "#/components/ui/skeleton";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { operatorGamedataServer } from "#/lib/api/gamedata";
import { rangesQueryOptions } from "#/lib/api/ranges";
import { GameText } from "#/lib/gamedata/GameText";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { blackboardDiff } from "#/lib/operators/cn-changes";
import { cn, rarityToNumber } from "#/lib/utils";
import type { IAttributeData, IModulePart, IModulePhase, IOperatorListItem, IOperatorModule, IPotentialRank } from "#/types/operators";
import { asset, eliteIcon, potentialIcon } from "../../assets";
import { type ICnModuleTexts, type ICnText, templateWithCnValues } from "../../cnText";
import { type ICnUpdates, useCnUpdatesContext } from "../../cnUpdates";
import type { messages as detailConstantsMessages } from "../../constants.messages";
import { DIFF_INSERT_CLASS, descriptionToHtml, renderDescriptionDiffHtml } from "../../description";
import { combinedDescriptionBlackboard, formatAttributeKey, formatStatValue, getActiveTalentCandidate, getOperatorAttributeStats } from "../../helpers";
import type { messages as helperMessages } from "../../helpers.messages";
import { useCommunityDefaults } from "../../useCommunityDefaults";
import { BaseSkillsSection } from "../BaseSkillsSection";
import { CnDescription, CnTag, CnValue } from "../CnMarkers";
import { CommunitySharePill } from "../CommunitySharePill";
import { LevelBreakdown } from "../LevelBreakdown";
import { OperatorNotes } from "../OperatorNotes";
import { OperatorRange } from "../OperatorRange";
import { SummonsSection } from "../SummonsSection";
import type { messages } from "./InfoContent.messages";

interface IInfoContentProps {
    operator: IOperatorListItem;
}

/** `cn` is CN's value at the same state, set only where it differs. */
type Stat = { iconURL: string; label: string; value: string; cn?: string };

/** This tab renders its own chrome, the shared stat labels, and the labels `helpers.ts` derives. */
type InfoT = TypedT<typeof messages & typeof detailConstantsMessages & typeof helperMessages>;

/** Modules unlock at the second promotion; below it the module controls and details are hidden. */
const MODULE_PHASE_INDEX = 2;

type Blackboard = NonNullable<Parameters<typeof descriptionToHtml>[1]>;

/** A description together with the blackboard that fills its `{key}` placeholders. */
interface IDescribed {
    description: string;
    blackboard: Blackboard;
}

/** A talent as it reads at the chosen promotion, level, potential and module. */
interface ITalentEffect {
    key: string;
    name: string | null;
    description: string;
    blackboard: Blackboard;
    requiredPotentialRank: number;
    /** The talent before the module rewrote it; null when the module added it. */
    base: IDescribed | null;
    modifiedByModule: boolean;
    /** Rewritten by a module merged in from CN. */
    fromCn: boolean;
    /** The rewrite's text as this server can show it, set when `fromCn`; `description` is then its whole text. */
    cnText: ICnText | null;
    /** The name is CN's own, with no counterpart on this server. */
    zhName: boolean;
    /** Which candidates and module phases produced it, to tell a talent CN reshaped from one whose values changed. */
    shape: string;
}

/** Changes that move the combat stats: base stats, trust, potentials, module bonuses. */
const CN_STAT_KINDS = ["stats_changed", "trust_bonus_changed", "potentials_changed", "module_changed"] as const;

/** Same tolerance as the backend diff: the tables carry `f32` values widened to `f64`. */
function differs(a: number | null | undefined, b: number | null | undefined): b is number {
    return typeof a === "number" && typeof b === "number" && Math.abs(a - b) > 1e-6 * Math.max(1, Math.abs(a), Math.abs(b));
}

/** Potential ranks compare by their bonuses only; the descriptions are each server's own language. */
function samePotential(a: IPotentialRank | undefined, b: IPotentialRank | undefined): boolean {
    return JSON.stringify(a?.buff ?? null) === JSON.stringify(b?.buff ?? null);
}

/** A potential rank's bonuses as a blackboard, so its description can take another server's numbers. */
function potentialBlackboard(rank: IPotentialRank | null | undefined): Blackboard {
    return (rank?.buff?.attributes.attributeModifiers ?? []).map((m) => ({ key: m.attributeType, value: m.value }));
}

/** CN's potential rank as HTML: this server's description with CN's numbers, else CN's own (tagged Chinese). */
function cnPotentialHtml(rank: IPotentialRank | null | undefined, cnRank: IPotentialRank): { html: string; lang?: string } {
    const html = rank?.description ? templateWithCnValues(rank.description, potentialBlackboard(rank), potentialBlackboard(cnRank), cnRank.description) : null;
    return html !== null && rank && html !== descriptionToHtml(rank.description) ? { html } : { html: descriptionToHtml(cnRank.description), lang: "zh-CN" };
}

/** The module the tab opens on: the first advanced one this server has, never one merged in from CN. */
function defaultModule(operator: IOperatorListItem, cnUpdates: ICnUpdates | null): IOperatorModule | undefined {
    return operator.modules.find((m) => m.type !== "INITIAL" && !cnUpdates?.moduleIds.has(m.uniEquipId));
}

/**
 * `text` repeated with CN's numbers, the changed ones marked, or CN's own `cnText` (tagged Chinese)
 * where `text` cannot take CN's numbers; null when no number in it moves.
 */
function cnNumbersHtml(text: string, blackboard: Blackboard, cnText: string | null | undefined, cnBlackboard: Blackboard | undefined): { html: string; lang?: string } | null {
    if (!text || !cnBlackboard || blackboardDiff(blackboard, cnBlackboard).length === 0) return null;
    const html = templateWithCnValues(text, blackboard, cnBlackboard, cnText);
    if (html === null) return cnText ? { html: descriptionToHtml(cnText, cnBlackboard), lang: "zh-CN" } : null;
    return html === descriptionToHtml(text, blackboard) ? null : { html };
}

/** Chinese text from CN's data, tagged as such, and marked as added when it follows a diff. */
function zhSpan(text: string, blackboard: Blackboard, added: boolean): string {
    return `<span lang="zh-CN"${added ? ` class="${DIFF_INSERT_CLASS}"` : ""}>${descriptionToHtml(text, blackboard)}</span>`;
}

/**
 * {@link renderChange} for a rewrite from a CN module: the part in this server's language diffs
 * against `base` as any module's does, and the Chinese CN adds after it follows in its own span,
 * after `separator` (HTML).
 */
function renderCnChange(base: IDescribed | null, text: ICnText, blackboard: Blackboard, showDiff: boolean, separator = ""): { html: string; baseHtml: string | null } {
    if (text.text === null) return { html: zhSpan(text.zh ?? "", blackboard, false), baseHtml: null };
    const change = renderChange(base, { description: text.text, blackboard }, showDiff);
    return text.zh ? { ...change, html: `${change.html}${separator}${zhSpan(text.zh, blackboard, change.baseHtml !== null)}` } : change;
}

function isTalentPart(part: IModulePart): boolean {
    return part.target === "TALENT" || part.target === "TALENT_DATA_ONLY";
}

function isTraitPart(part: IModulePart): boolean {
    return part.target === "TRAIT" || part.target === "TRAIT_DATA_ONLY" || part.target === "DISPLAY";
}

/** `X-Y` when the module carries a two-part type designator, else its full name. */
function moduleDesignator(module: IOperatorModule): string {
    return module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.uniEquipName;
}

/**
 * The HTML for a description that a module may have changed: the new text
 * alone, or (when the viewer wants the diff and a base exists) the base text
 * plus the new text with its changes marked.
 */
function renderChange(base: IDescribed | null, next: IDescribed, showDiff: boolean): { html: string; baseHtml: string | null } {
    if (!showDiff || !base) return { html: descriptionToHtml(next.description, next.blackboard), baseHtml: null };
    return {
        html: renderDescriptionDiffHtml(base.description, next.description, base.blackboard, next.blackboard),
        baseHtml: descriptionToHtml(base.description, base.blackboard),
    };
}

/**
 * How a module phase rewrites the operator's trait. An override replaces the
 * trait text and compares against the operator's last trait candidate (or the
 * bare description when the trait carries no override of its own); an
 * addition appends to the description and merges both blackboards.
 */
function traitChange(operator: IOperatorListItem, phase: IModulePhase, descriptionBlackboard: Blackboard): { base: IDescribed | null; next: IDescribed; part: { kind: "override" | "addition"; text: string; blackboard: Blackboard } } | null {
    const candidate = phase.parts.find(isTraitPart)?.overrideTraitDataBundle.candidates?.[0];
    if (!candidate) return null;
    const overrideDesc = candidate.overrideDescription ?? "";
    const additionalDesc = candidate.additionalDescription ?? "";
    if (overrideDesc.length === 0 && additionalDesc.length === 0) return null;

    const nextBlackboard = candidate.blackboard ?? [];
    if (overrideDesc.length > 0) {
        const traitCandidates = operator.trait?.candidates ?? [];
        const lastTrait = traitCandidates[traitCandidates.length - 1];
        const baseFromTrait = lastTrait?.overrideDescription ?? null;
        const baseDesc = baseFromTrait ?? operator.description ?? null;
        return {
            base: baseDesc ? { description: baseDesc, blackboard: baseFromTrait ? (lastTrait?.blackboard ?? []) : descriptionBlackboard } : null,
            next: { description: overrideDesc, blackboard: nextBlackboard },
            part: { kind: "override", text: overrideDesc, blackboard: nextBlackboard },
        };
    }

    const baseDesc = operator.description ?? "";
    return {
        base: baseDesc.length > 0 ? { description: baseDesc, blackboard: descriptionBlackboard } : null,
        next: { description: baseDesc.length > 0 ? `${baseDesc}\n${additionalDesc}` : additionalDesc, blackboard: [...descriptionBlackboard, ...nextBlackboard] },
        part: { kind: "addition", text: additionalDesc, blackboard: [...descriptionBlackboard, ...nextBlackboard] },
    };
}

/** The talent candidates a module phase adds or overrides, keeping only those with text to show. */
function moduleTalentCandidates(phase: IModulePhase) {
    return phase.parts.filter(isTalentPart).flatMap((part) => part.addOrOverrideTalentDataBundle.candidates?.filter((c) => c.upgradeDescription || c.description) ?? []);
}

/**
 * The operator's talents at the chosen state, with every module phase up to
 * `moduleLevel` applied in order: a candidate whose `talentIndex` names an
 * existing talent rewrites it, any other index appends a module-only talent.
 * Within one phase the highest candidate the potential rank unlocks wins.
 * `cnTexts` is set for a module merged in from CN, to read its text in this server's language.
 */
function buildTalentEffects(operator: IOperatorListItem, phaseIndex: number, level: number, potentialRank: number, module: IOperatorModule | null, moduleLevel: number, cnTexts: ICnModuleTexts | null = null): ITalentEffect[] {
    const effects: ITalentEffect[] = (operator.talents ?? []).map((entry, idx) => {
        const c = getActiveTalentCandidate(entry, phaseIndex, level, potentialRank);
        const described: IDescribed = { description: c?.description ?? "", blackboard: c?.blackboard ?? [] };
        return {
            key: `t-${idx}`,
            name: c?.name ?? null,
            ...described,
            requiredPotentialRank: c?.requiredPotentialRank ?? 0,
            base: c?.description ? described : null,
            modifiedByModule: false,
            fromCn: false,
            cnText: null,
            zhName: false,
            shape: c ? `${entry.candidates?.length}:${c.unlockCondition.phase}/${c.unlockCondition.level}/${c.requiredPotentialRank}` : "",
        };
    });

    const phases = module?.data?.phases;
    if (phaseIndex !== MODULE_PHASE_INDEX || !module || moduleLevel <= 0 || !phases) return effects;

    for (let i = 0; i < Math.min(moduleLevel, phases.length); i++) {
        for (const part of (phases[i]?.parts ?? []).filter(isTalentPart)) {
            const candidates = part.addOrOverrideTalentDataBundle?.candidates ?? [];
            let chosen: (typeof candidates)[number] | null = null;
            for (const c of candidates) {
                if (potentialRank >= c.requiredPotentialRank) chosen = c;
            }
            if (!chosen) continue;
            const description = chosen.upgradeDescription || chosen.description || "";
            const blackboard = chosen.blackboard ?? [];
            const tIdx = chosen.talentIndex;
            const cn = cnTexts?.talent({ talentIndex: tIdx, requiredPotentialRank: chosen.requiredPotentialRank, name: chosen.name, description, blackboard }) ?? null;
            const rewrite = {
                name: cn ? (cn.name.text ?? cn.name.zh) : chosen.name,
                description,
                blackboard,
                modifiedByModule: true,
                fromCn: cn !== null,
                cnText: cn?.description ?? null,
                zhName: !!cn && cn.name.text === null && !!cn.name.zh,
            };
            const shape = `+${i}/${chosen.requiredPotentialRank}`;
            if (tIdx >= 0 && tIdx < effects.length) {
                effects[tIdx] = { ...effects[tIdx], ...rewrite, name: rewrite.name ?? effects[tIdx].name, shape: effects[tIdx].shape + shape };
            } else {
                effects.push({
                    key: `t-mod-${tIdx}-${i}`,
                    ...rewrite,
                    requiredPotentialRank: chosen.requiredPotentialRank ?? 0,
                    base: null,
                    shape,
                });
            }
        }
    }
    return effects;
}

export const InfoContent = memo(function InfoContent({ operator }: IInfoContentProps) {
    const t: InfoT = useT("operators");
    const f = useFormatters();
    const levelLabelId = useId();
    const trustLabelId = useId();
    const promotionLabelId = useId();
    const potentialLabelId = useId();
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(operator.phases[operator.phases.length - 1]?.maxLevel ?? 1);
    const [trustLevel, setTrustLevel] = useState(100);
    const [potentialRank, setPotentialRank] = useState(rarityToNumber(operator.rarity) <= 4 ? 5 : 0);
    const cnUpdates = useCnUpdatesContext();
    const [moduleId, setModuleId] = useState(() => defaultModule(operator, cnUpdates)?.uniEquipId ?? "");
    const [moduleLevel, setModuleLevel] = useState(() => defaultModule(operator, cnUpdates)?.data?.phases?.length ?? 0);

    // Set once the viewer picks a module themselves, or once a default lands,
    // so late-arriving community data cannot override a deliberate choice.
    const moduleSettled = useRef(false);

    const localeServer = useGamedataServer();
    const { data: ranges, isLoading: rangesLoading, isError: rangesFailed, refetch: refetchRanges } = useQuery(rangesQueryOptions(operatorGamedataServer(operator.server, localeServer)));
    const currentRange = ranges?.[operator.phases[phaseIndex]?.rangeId ?? ""];
    // CN's range at this promotion, fetched only when CN changed a range and this one differs.
    const cnRangeId = cnUpdates?.kinds.has("range_changed") ? cnUpdates.cn.phases[phaseIndex]?.rangeId : undefined;
    const cnRangeDiffers = !!cnRangeId && cnRangeId !== operator.phases[phaseIndex]?.rangeId;
    const { data: cnRanges } = useQuery({ ...rangesQueryOptions("cn"), enabled: cnRangeDiffers });
    const cnRange = cnRangeDiffers && cnRangeId ? cnRanges?.[cnRangeId] : undefined;

    const availableModules = useMemo(() => operator.modules.filter((m) => m.type !== "INITIAL"), [operator.modules]);

    // Open on the viewer's own equipped module, then the community's most-used,
    // then the first one this component already chose. Both sources are already
    // filtered to real ADVANCED modules the operator has, so anything non-null
    // here is selectable.
    const { ownModuleId, communityModuleId, moduleShares, moduleTotal, moduleLevels, ownModuleLevels } = useCommunityDefaults(operator);
    useEffect(() => {
        if (moduleSettled.current) return;
        const id = ownModuleId ?? communityModuleId;
        if (id == null) return;
        const m = availableModules.find((x) => x.uniEquipId === id);
        if (!m) return;
        moduleSettled.current = true;
        setModuleId(id);
        setModuleLevel(m.data?.phases?.length ?? 0);
    }, [ownModuleId, communityModuleId, availableModules]);
    const currentModule = useMemo(() => (moduleId && moduleId.length > 0 ? (availableModules.find((m) => m.uniEquipId === moduleId) ?? null) : null), [moduleId, availableModules]);
    const currentModulePhase = moduleLevel > 0 ? (currentModule?.data?.phases?.[moduleLevel - 1] ?? null) : null;
    const moduleLevelBuckets = moduleId ? moduleLevels.get(moduleId) : undefined;
    const showModuleControls = phaseIndex === MODULE_PHASE_INDEX;
    const isCnModule = (id: string) => cnUpdates?.moduleIds.has(id) ?? false;
    const currentModuleIsCn = currentModule ? isCnModule(currentModule.uniEquipId) : false;
    // A module both servers have, as CN's record carries it, for marking its changed bonuses.
    const currentModuleOnCn = cnUpdates?.kinds.has("module_changed") && currentModule && !currentModuleIsCn ? cnUpdates.cn.modules.find((m) => m.uniEquipId === currentModule.uniEquipId) : undefined;
    const currentModulePhaseOnCn = moduleLevel > 0 ? (currentModuleOnCn?.data?.phases?.[moduleLevel - 1] ?? null) : null;

    const [showProfile, setShowProfile] = useState(true);
    const [showControls, setShowControls] = useState(true);
    const [showModuleDetails, setShowModuleDetails] = useState(true);
    const [showDiff, setShowDiff] = useState(true);
    const [showTalents, setShowTalents] = useState(true);

    const descriptionBlackboard = useMemo(() => combinedDescriptionBlackboard(operator), [operator]);
    const description = useMemo(() => descriptionToHtml(operator.description ?? "", descriptionBlackboard), [operator.description, descriptionBlackboard]);

    const stats = useMemo(
        () =>
            getOperatorAttributeStats(
                operator,
                {
                    phaseIndex,
                    favorPoint: trustLevel,
                    potentialRank,
                    moduleId,
                    moduleLevel,
                },
                level,
            ),
        [operator, phaseIndex, level, trustLevel, potentialRank, moduleId, moduleLevel],
    );

    // The same state read off CN's record, only when CN changed something that moves a stat. A
    // module merged in from CN is CN's own there, so it is the same module on both sides.
    const cnStats = useMemo(() => {
        if (!cnUpdates || !CN_STAT_KINDS.some((k) => cnUpdates.kinds.has(k))) return null;
        return getOperatorAttributeStats(cnUpdates.cn, { phaseIndex, favorPoint: trustLevel, potentialRank, moduleId, moduleLevel }, level);
    }, [cnUpdates, phaseIndex, level, trustLevel, potentialRank, moduleId, moduleLevel]);

    const cnModuleTexts = currentModuleIsCn ? (cnUpdates?.texts ?? null) : null;
    const talents = useMemo(() => buildTalentEffects(operator, phaseIndex, level, potentialRank, currentModule, moduleLevel, cnModuleTexts).filter((eff) => eff.name || eff.description), [operator, phaseIndex, level, potentialRank, currentModule, moduleLevel, cnModuleTexts]);
    // CN's talents at the same state, by key, where a talent's or a shared module's values changed.
    const cnTalents = useMemo(() => {
        if (!cnUpdates || !(cnUpdates.kinds.has("talents_changed") || cnUpdates.kinds.has("module_changed"))) return null;
        const cnModule = currentModule ? (cnUpdates.cn.modules.find((m) => m.uniEquipId === currentModule.uniEquipId) ?? null) : null;
        return new Map(buildTalentEffects(cnUpdates.cn, phaseIndex, level, potentialRank, cnModule, moduleLevel).map((eff) => [eff.key, eff]));
    }, [cnUpdates, phaseIndex, level, potentialRank, currentModule, moduleLevel]);
    const cnDescription = useMemo(() => (cnUpdates?.kinds.has("trait_changed") ? cnNumbersHtml(operator.description ?? "", descriptionBlackboard, cnUpdates.cn.description, combinedDescriptionBlackboard(cnUpdates.cn)) : null), [cnUpdates, operator.description, descriptionBlackboard]);

    const fmt = (n: number | undefined) => (typeof n === "number" ? f.number(Math.round(n)) : (n ?? "-"));
    const seconds = (n: number | undefined) => t("stat.seconds", { value: n ?? 0 });
    const interval = (n: number | undefined) => t("stat.seconds", { value: n?.toFixed(2) ?? "0.00" });
    /** CN's value of one stat, formatted as the row formats its own, where it differs. */
    const onCn = (key: keyof IAttributeData, format: (n: number | undefined) => string) => {
        const cnValue = cnStats?.[key];
        return differs(stats?.[key] as number | undefined, cnValue as number | undefined) ? format(cnValue as number) : undefined;
    };

    const leftStats: Stat[] = [
        { iconURL: "/stat-icons/HP.png", label: t("stat.health"), value: fmt(stats?.maxHp), cn: onCn("maxHp", fmt) },
        { iconURL: "/stat-icons/DEF.png", label: t("stat.defense"), value: fmt(stats?.def), cn: onCn("def", fmt) },
        { iconURL: "/stat-icons/RES.png", label: t("stat.artsResistance"), value: fmt(stats?.magicResistance), cn: onCn("magicResistance", fmt) },
        { iconURL: "/stat-icons/RDP.png", label: t("stat.redeployTime"), value: seconds(stats?.respawnTime), cn: onCn("respawnTime", seconds) },
    ];
    const rightStats: Stat[] = [
        { iconURL: "/stat-icons/ATK.png", label: t("stat.attackPower"), value: fmt(stats?.atk), cn: onCn("atk", fmt) },
        { iconURL: "/stat-icons/ASPD.png", label: t("stat.attackInterval"), value: interval(stats?.attackSpeed), cn: onCn("attackSpeed", interval) },
        { iconURL: "/stat-icons/BLOCK.png", label: t("stat.block"), value: fmt(stats?.blockCnt), cn: onCn("blockCnt", fmt) },
        { iconURL: "/stat-icons/COST.png", label: t("stat.dpCost"), value: fmt(stats?.cost), cn: onCn("cost", fmt) },
    ];

    const profile = operator.profile?.basicInfo;
    const unknown = t("info.profile.unknown");
    const fact = useOperatorFactLabel();

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">{t("info.title")}</h2>
                <p
                    className="wrap-break-word text-muted-foreground text-sm"
                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                    dangerouslySetInnerHTML={{ __html: description }}
                />
                {cnDescription && <CnDescription {...cnDescription} />}
            </div>

            <CollapsibleSection icon={User} title={t("info.profile")} open={showProfile} onOpenChange={setShowProfile}>
                <dl className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
                    {profile && (
                        <>
                            <ProfileCard icon={MapPin} label={t("info.profile.placeOfBirth")} value={profile.placeOfBirth ? fact.birthPlace(profile.placeOfBirth) : unknown} />
                            <ProfileCard icon={Dna} label={t("info.profile.race")} value={profile.race ? fact.race(profile.race) : unknown} />
                            <ProfileCard icon={User} label={t("info.profile.gender")} value={profile.gender ? fact.gender(profile.gender) : unknown} />
                            <ProfileCard icon={Info} label={t("info.profile.height")} value={profile.height ?? unknown} />
                        </>
                    )}
                    {operator.artists && operator.artists.length > 0 && <ProfileCard icon={Palette} label={t("info.profile.artist")} value={operator.artists.join(", ")} />}
                </dl>
            </CollapsibleSection>

            <Separator className="my-6" />

            <OperatorNotes operatorId={operator.id ?? null} />

            <Separator className="my-6" />

            <CollapsibleSection icon={Package} title={t("info.controls")} open={showControls} onOpenChange={setShowControls}>
                <div className="mt-3 space-y-4 rounded-lg border border-border/50 bg-card/30 p-4">
                    <p className="text-muted-foreground text-xs">{t("info.controls.desc")}</p>

                    <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground text-sm" id={promotionLabelId}>
                            {t("info.promotion")}
                        </span>
                        <ToggleGroup
                            aria-labelledby={promotionLabelId}
                            className="flex-wrap gap-2"
                            onValueChange={(next) => {
                                if (next[0] === undefined) return;
                                const idx = Number(next[0]);
                                setPhaseIndex(idx);
                                setLevel(operator.phases[idx].maxLevel);
                            }}
                            size="lg"
                            value={[String(phaseIndex)]}
                            variant="outline"
                        >
                            {operator.phases.map((_, idx) => (
                                // biome-ignore lint/suspicious/noArrayIndexKey: phase ordering is fixed by elite tier
                                <ToggleGroupItem key={`phase-${idx}`} value={String(idx)}>
                                    <img alt={t("info.eliteAlt", { elite: idx })} className="icon-theme-aware h-6 w-6 object-contain" decoding="async" loading="lazy" src={eliteIcon(idx, operator.server)} />
                                </ToggleGroupItem>
                            ))}
                        </ToggleGroup>
                    </div>

                    <div className="grid gap-4 md:grid-cols-2">
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm" id={levelLabelId}>
                                    {t("info.level")}
                                </span>
                                <span className="font-mono text-foreground text-sm">
                                    {level} / {operator.phases[phaseIndex]?.maxLevel ?? 1}
                                </span>
                            </div>
                            <Slider aria-labelledby={levelLabelId} min={1} max={operator.phases[phaseIndex]?.maxLevel ?? 1} step={1} value={[level]} onValueChange={(v) => setLevel(Array.isArray(v) ? (v[0] ?? 1) : v)} />
                        </div>
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="flex items-center gap-1 text-muted-foreground text-sm" id={trustLabelId}>
                                    <Heart aria-hidden="true" className="h-3.5 w-3.5" /> {t("info.trust")}
                                </span>
                                <span className="font-mono text-foreground text-sm">{trustLevel}%</span>
                            </div>
                            <Slider aria-labelledby={trustLabelId} min={0} max={200} step={1} value={[trustLevel]} onValueChange={(v) => setTrustLevel(Array.isArray(v) ? (v[0] ?? 100) : v)} />
                        </div>
                    </div>

                    <div className="space-y-2">
                        <div className="flex items-center gap-2 text-muted-foreground text-sm">
                            <span id={potentialLabelId}>{t("info.potential")}</span>
                            <Tooltip>
                                <TooltipTrigger
                                    render={(props) => (
                                        <button {...props} aria-label={t("info.potential.tip")} className="relative inline-flex cursor-help items-center justify-center rounded-sm text-muted-foreground pointer-coarse:after:absolute pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11" type="button">
                                            <Info aria-hidden="true" className="h-3 w-3" />
                                        </button>
                                    )}
                                />
                                <TooltipPopup>{t("info.potential.tip")}</TooltipPopup>
                            </Tooltip>
                        </div>
                        <ToggleGroup aria-labelledby={potentialLabelId} className="flex-wrap gap-1.5" onValueChange={(next) => next[0] !== undefined && setPotentialRank(Number(next[0]))} size="sm" value={[String(potentialRank)]} variant="outline">
                            {Array.from({
                                length: (operator.potentialRanks?.length ?? 0) + 1,
                            }).map((_, idx) => {
                                const rank = idx === 0 ? null : operator.potentialRanks?.[idx - 1];
                                const cnRank = idx > 0 && cnUpdates?.kinds.has("potentials_changed") ? cnUpdates.cn.potentialRanks?.[idx - 1] : undefined;
                                const cnRankDiffers = !!cnRank && !samePotential(rank ?? undefined, cnRank);
                                const cnRankHtml = cnRankDiffers && cnRank ? cnPotentialHtml(rank, cnRank) : null;
                                return (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: potential rank is positionally stable
                                    <Tooltip key={`pot-${idx}`}>
                                        <TooltipTrigger
                                            render={(props) => (
                                                <ToggleGroupItem {...props} value={String(idx)}>
                                                    <img alt={t("info.potential.alt", { rank: idx })} className="icon-theme-aware h-5 w-5 object-contain" decoding="async" loading="lazy" src={potentialIcon(idx, operator.server)} />
                                                    {cnRankDiffers && <span aria-hidden="true" className="absolute -top-0.5 -right-0.5 h-1.5 w-1.5 rounded-full bg-primary" />}
                                                </ToggleGroupItem>
                                            )}
                                        />
                                        <TooltipPopup>
                                            {rank ? t("info.potential.rankTip", { rank: idx, description: rank.description }) : t("info.potential.baseTip", { rank: idx })}
                                            {cnRankHtml && (
                                                <span className="mt-1 flex items-baseline gap-1.5">
                                                    <CnTag />
                                                    <span
                                                        lang={cnRankHtml.lang}
                                                        // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                                                        dangerouslySetInnerHTML={{ __html: cnRankHtml.html }}
                                                    />
                                                </span>
                                            )}
                                        </TooltipPopup>
                                    </Tooltip>
                                );
                            })}
                        </ToggleGroup>
                    </div>

                    {showModuleControls && availableModules.length > 0 && (
                        <div className="grid gap-3 md:grid-cols-2">
                            <div className="space-y-1">
                                <span className="text-muted-foreground text-xs">{t("info.module")}</span>
                                <Select
                                    value={moduleId || "none"}
                                    onValueChange={(v) => {
                                        const value = String(v);
                                        moduleSettled.current = true;
                                        if (value === "none") {
                                            setModuleId("");
                                            setModuleLevel(0);
                                            return;
                                        }
                                        setModuleId(value);
                                        const m = availableModules.find((x) => x.uniEquipId === value);
                                        setModuleLevel(m?.data?.phases?.length ?? 0);
                                    }}
                                >
                                    <SelectTrigger>
                                        <SelectValue placeholder={t("info.module.placeholder")}>
                                            {(value: string) => {
                                                if (value === "none") return t("info.module.none");
                                                const m = availableModules.find((x) => x.uniEquipId === value);
                                                if (!m) return value;
                                                return isCnModule(m.uniEquipId) ? (
                                                    <span className="flex items-center gap-2">
                                                        <span lang="zh-CN">{moduleDesignator(m)}</span>
                                                        <CnTag />
                                                    </span>
                                                ) : (
                                                    moduleDesignator(m)
                                                );
                                            }}
                                        </SelectValue>
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="none">{t("info.module.none")}</SelectItem>
                                        {availableModules.map((mod) => (
                                            <SelectItem key={mod.uniEquipId} value={mod.uniEquipId}>
                                                <span className="flex items-center gap-2">
                                                    {isCnModule(mod.uniEquipId) ? (
                                                        <>
                                                            <span lang="zh-CN">{moduleDesignator(mod)}</span>
                                                            <CnTag />
                                                        </>
                                                    ) : (
                                                        moduleDesignator(mod)
                                                    )}
                                                    <CommunitySharePill share={moduleShares.get(mod.uniEquipId)} total={moduleTotal} cohort={t("info.module.cohort")} />
                                                </span>
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                                {/* The collapsed trigger shows only the
                                    designator, so the selected module's
                                    share is restated here rather than being
                                    reachable only by opening the dropdown. */}
                                {moduleId && moduleShares.has(moduleId) && <p className="text-[11px] text-muted-foreground">{t("info.module.shareLine", { pct: Math.round((moduleShares.get(moduleId)?.share ?? 0) * 100), total: f.number(moduleTotal) })}</p>}
                            </div>
                            {currentModule?.data?.phases && currentModule.data.phases.length > 0 && (
                                <div className="space-y-1">
                                    <span className="text-muted-foreground text-xs">{t("info.module.level")}</span>
                                    <Select value={String(moduleLevel)} onValueChange={(v) => setModuleLevel(Number.parseInt(String(v), 10))}>
                                        <SelectTrigger>
                                            <SelectValue placeholder={t("info.module.levelPlaceholder")}>{(value: string) => t("info.module.levelOption", { level: value })}</SelectValue>
                                        </SelectTrigger>
                                        <SelectContent>
                                            {currentModule.data.phases.map((phase) => (
                                                <SelectItem key={phase.equipLevel} value={String(phase.equipLevel)}>
                                                    {t("info.module.levelOption", { level: phase.equipLevel })}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                            )}

                            {/* Spans both columns rather than sitting in the
                                Module cell: at md and up that cell is half
                                the row, which squeezed four labelled bars
                                into about 300px while the Module Level
                                cell beside it sat mostly empty. */}
                            {moduleLevelBuckets && (
                                <LevelBreakdown
                                    className="md:col-span-2"
                                    buckets={moduleLevelBuckets.buckets}
                                    total={moduleLevelBuckets.total}
                                    title={t("info.module.breakdown.title")}
                                    labels={[t("info.module.breakdown.notUnlocked"), t("info.module.breakdown.lv1"), t("info.module.breakdown.lv2"), t("info.module.breakdown.lv3")]}
                                    summary={t("info.module.breakdown.summary")}
                                    cohort={t("info.cohort.e2Owners")}
                                    ownLevel={ownModuleLevels.get(moduleId) ?? null}
                                />
                            )}
                        </div>
                    )}
                </div>
            </CollapsibleSection>

            <Separator className="my-6" />

            <div className="mb-6">
                <h3 className="mb-4 font-medium text-foreground">{t("info.combatStats")}</h3>
                <div className="grid grid-cols-1 divide-y divide-border rounded-lg border border-border bg-card md:grid-cols-2 md:divide-x md:divide-y-0">
                    {[leftStats, rightStats].map((column, colIdx) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: fixed two-column split, order is stable
                        <div key={colIdx} className="px-5 py-2">
                            {column.map(({ iconURL, label, value, cn: cnValue }) => (
                                <div key={label} className="flex items-center justify-between py-2">
                                    <span className="flex items-center gap-2 text-muted-foreground">
                                        <img alt="" src={iconURL} className="icon-theme-aware h-4 w-4 object-contain" decoding="async" loading="lazy" />
                                        <span className="text-sm">{label}</span>
                                    </span>
                                    <span className="flex items-baseline">
                                        <span className="font-semibold text-foreground tabular-nums">{value}</span>
                                        {cnValue && <CnValue value={cnValue} />}
                                    </span>
                                </div>
                            ))}
                        </div>
                    ))}
                </div>
            </div>
            {operator.tagList && operator.tagList.length > 0 && (
                <div className="mb-6">
                    <h3 className="mb-3 font-medium text-foreground">{t("info.tags")}</h3>
                    <div className="flex flex-wrap gap-2">
                        {operator.tagList.map((tag) => (
                            <Badge key={tag} variant="secondary" className="bg-accent">
                                {tag}
                            </Badge>
                        ))}
                    </div>
                </div>
            )}
            <div className="mb-6">
                <h3 className="mb-3 font-medium text-foreground">{t("info.attackRange")}</h3>
                {currentRange ? (
                    <OperatorRange range={currentRange} />
                ) : rangesLoading ? (
                    <Skeleton className="h-24 w-40 rounded-md" />
                ) : rangesFailed ? (
                    <LoadFailed message={t("info.attackRange.error")} onRetry={() => void refetchRanges()} />
                ) : (
                    <p className="text-muted-foreground text-sm">{t("info.attackRange.empty")}</p>
                )}
                {cnRange && (
                    <div className="mt-3 flex items-start gap-2">
                        <CnTag className="mt-1" />
                        <OperatorRange range={cnRange} />
                    </div>
                )}
            </div>

            {showModuleControls && currentModule && (
                <div className="mb-6">
                    <CollapsibleSection icon={Package} title={t("info.moduleDetails")} open={showModuleDetails} onOpenChange={setShowModuleDetails}>
                        <div className="mt-3 rounded-lg border border-border/50 bg-card/30 p-4">
                            <div className="mb-4 flex items-center gap-3">
                                {currentModule.image && <img alt={currentModule.uniEquipName} className="h-16 w-16 rounded-md object-contain" decoding="async" loading="lazy" src={asset(currentModule.image, currentModuleIsCn ? "cn" : operator.server)} />}
                                <div>
                                    <div className="flex items-center gap-2">
                                        <h4 lang={currentModuleIsCn ? "zh-CN" : undefined} className="font-semibold text-foreground">
                                            {currentModule.uniEquipName}
                                        </h4>
                                        {currentModuleIsCn && <CnTag />}
                                    </div>
                                    <div lang={currentModuleIsCn ? "zh-CN" : undefined} className="mt-1 flex gap-1">
                                        <Badge variant="outline">{currentModule.typeName1}</Badge>
                                        {currentModule.typeName2 && <Badge variant="outline">{currentModule.typeName2}</Badge>}
                                    </div>
                                </div>
                            </div>
                            {currentModule.uniEquipDesc && (
                                <div className="mb-4 max-h-32 overflow-y-auto rounded-md bg-secondary/20 p-3">
                                    <p lang={currentModuleIsCn ? "zh-CN" : undefined} className="whitespace-pre-line text-muted-foreground text-xs leading-relaxed">
                                        <GameText text={currentModule.uniEquipDesc} />
                                    </p>
                                </div>
                            )}
                            {currentModulePhase ? (
                                <div className="space-y-3">
                                    <h5 className="font-medium text-foreground text-sm">{t("info.moduleDetails.levelStats", { level: moduleLevel })}</h5>
                                    {currentModulePhase.attributeBlackboard.length > 0 ? (
                                        <div className="grid grid-cols-2 gap-2 md:grid-cols-3">
                                            {currentModulePhase.attributeBlackboard.map((attr) => {
                                                const cnValue = currentModulePhaseOnCn?.attributeBlackboard.find((x) => x.key.toLowerCase() === attr.key.toLowerCase())?.value;
                                                return (
                                                    <div className="rounded-md bg-secondary/30 p-2" key={`${attr.key}-${moduleLevel}`}>
                                                        <span className="text-muted-foreground text-xs">{formatAttributeKey(attr.key, t)}:</span>
                                                        <span className="ml-1 font-medium text-foreground text-sm">{formatStatValue(attr.value)}</span>
                                                        {differs(attr.value, cnValue) && <CnValue value={formatStatValue(cnValue)} />}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    ) : (
                                        <p className="text-muted-foreground text-xs">{t("info.moduleDetails.noBonuses")}</p>
                                    )}
                                </div>
                            ) : (
                                <p className="text-muted-foreground text-xs">{t("info.moduleDetails.pickLevel")}</p>
                            )}

                            {currentModulePhase && (
                                <div className="space-y-2">
                                    <ModuleTraitChange operator={operator} phase={currentModulePhase} descriptionBlackboard={descriptionBlackboard} showDiff={showDiff} onShowDiffChange={setShowDiff} cnTexts={cnModuleTexts} cnPhase={currentModulePhaseOnCn} cnOperator={cnUpdates?.cn ?? null} />
                                    <ModuleTalentChanges operator={operator} phase={currentModulePhase} showDiff={showDiff} cnTexts={cnModuleTexts} cnPhase={currentModulePhaseOnCn} />
                                </div>
                            )}
                        </div>
                    </CollapsibleSection>
                </div>
            )}

            <CollapsibleSection icon={LibraryBig} title={t("info.talents")} open={showTalents} onOpenChange={setShowTalents}>
                {talents.length === 0 ? (
                    <p className="mt-3 text-muted-foreground text-xs">{t("info.talents.empty")}</p>
                ) : (
                    <div className="mt-3 space-y-3">
                        {talents.map((talent) => {
                            const base = talent.modifiedByModule ? talent.base : null;
                            const change = talent.cnText ? renderCnChange(base, talent.cnText, talent.blackboard, showDiff) : renderChange(base, talent, showDiff);
                            const cnTalent = talent.fromCn ? undefined : cnTalents?.get(talent.key);
                            // A talent CN reshaped (other candidates, another module phase) is not this one with new values.
                            const cnHtml = cnTalent ? (cnTalent.shape === talent.shape ? cnNumbersHtml(talent.description, talent.blackboard, cnTalent.description, cnTalent.blackboard) : { html: descriptionToHtml(cnTalent.description, cnTalent.blackboard), lang: "zh-CN" }) : null;
                            return (
                                <div className="rounded-lg border border-border/50 bg-card/30 p-3" key={talent.key}>
                                    <div className="mb-1 flex items-center gap-2">
                                        <h4 lang={talent.zhName ? "zh-CN" : undefined} className="font-medium text-foreground text-sm">
                                            {talent.name ?? t("info.talents.unnamed")}
                                        </h4>
                                        {talent.requiredPotentialRank > 0 && (
                                            <Tooltip>
                                                <TooltipTrigger render={(props) => <img alt={t("info.potential.alt", { rank: talent.requiredPotentialRank })} className="icon-theme-aware h-4 w-4" decoding="async" loading="lazy" src={potentialIcon(talent.requiredPotentialRank, operator.server)} {...props} />} />
                                                <TooltipPopup>{t("info.talents.requiresPotential", { rank: talent.requiredPotentialRank })}</TooltipPopup>
                                            </Tooltip>
                                        )}
                                        {talent.modifiedByModule && (
                                            <Badge variant="outline" className="text-[10px]">
                                                {t("info.talents.moduleBadge")}
                                            </Badge>
                                        )}
                                        {talent.fromCn && <CnTag />}
                                    </div>
                                    <DescriptionDiff {...change} baseLabel={t("info.baseTalent")} />
                                    {cnHtml && <CnDescription {...cnHtml} />}
                                </div>
                            );
                        })}
                    </div>
                )}
            </CollapsibleSection>

            {operator.baseSkills && operator.baseSkills.length > 0 && (
                <>
                    <Separator className="my-6" />
                    <BaseSkillsSection skills={operator.baseSkills} server={operator.server} />
                </>
            )}

            {operator.drones && operator.drones.length > 0 && (
                <>
                    <Separator className="my-6" />
                    <SummonsSection drones={operator.drones} parentPhaseIndex={phaseIndex} parentLevel={level} server={operator.server} />
                </>
            )}
        </div>
    );
});

interface ICollapsibleSectionProps {
    icon: LucideIcon;
    title: string;
    open: boolean;
    onOpenChange: (open: boolean) => void;

    children: ReactNode;
}

/** Every section draws the chevron: without it a bar gave no sign that it folds, or whether it was open. */
function CollapsibleSection({ icon: Icon, title, open, onOpenChange, children }: ICollapsibleSectionProps) {
    return (
        <Collapsible open={open} onOpenChange={onOpenChange}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-2">
                    <Icon aria-hidden="true" className="h-4 w-4 text-primary" />
                    <span className="font-medium text-sm">{title}</span>
                </span>
                <ChevronDown aria-hidden="true" className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />
            </CollapsibleTrigger>
            <CollapsibleContent>{children}</CollapsibleContent>
        </Collapsible>
    );
}

/** One fact in the profile list, a boxed term over its value. */
function ProfileCard({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value: string }) {
    return (
        <div className="min-w-0 rounded-lg border border-border/50 bg-card/40 p-3">
            <dt className="flex items-center gap-1.5 text-muted-foreground">
                <Icon aria-hidden="true" className="h-3.5 w-3.5" />
                <span className="text-xs">{label}</span>
            </dt>
            <dd className="mt-1 truncate text-foreground text-sm">{value}</dd>
        </div>
    );
}

interface IDescriptionDiffProps {
    /** The new text, with its changes marked when `baseHtml` is set. */
    html: string;
    /** The text before the module changed it; null renders `html` alone. */
    baseHtml: string | null;
    baseLabel: string;
    /** A name shown before both texts, for talents. */
    prefix?: ReactNode;
}

const DIFF_LABEL_CLASS = "mb-0.5 font-medium text-[10px] text-muted-foreground uppercase tracking-wider";

/** A description a module changed: the base, struck through under its label, above the marked-up new
 *  text. The base used to be faded with opacity 50, which put xs muted text near 2:1. */
function DescriptionDiff({ html, baseHtml, baseLabel, prefix }: IDescriptionDiffProps) {
    const t: InfoT = useT("operators");
    return (
        <>
            {baseHtml && (
                <div className="mb-2 border-border/40 border-b pb-2">
                    <div className={DIFF_LABEL_CLASS}>{baseLabel}</div>
                    {prefix}
                    <span
                        className={cn("text-muted-foreground text-xs line-through decoration-muted-foreground/70", !prefix && "block")}
                        // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                        dangerouslySetInnerHTML={{ __html: baseHtml }}
                    />
                </div>
            )}
            {baseHtml && <div className={DIFF_LABEL_CLASS}>{t("info.changes")}</div>}
            {prefix}
            <span
                className="text-muted-foreground text-xs"
                // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                dangerouslySetInnerHTML={{ __html: html }}
            />
        </>
    );
}

interface IModuleTraitChangeProps {
    operator: IOperatorListItem;
    phase: IModulePhase;
    descriptionBlackboard: Blackboard;
    showDiff: boolean;
    onShowDiffChange: (show: boolean) => void;
    /** Set when the module is CN's, to read its text in this server's language where it can be. */
    cnTexts: ICnModuleTexts | null;
    /** The same phase of a module both servers have, as CN's record carries it, where CN changed the module. */
    cnPhase: IModulePhase | null;
    cnOperator: IOperatorListItem | null;
}

/** A CN module's trait part as this server can show it: an addition follows this server's trait. */
function cnTraitText(operator: IOperatorListItem, part: { kind: "override" | "addition"; text: string; blackboard: Blackboard }, texts: ICnModuleTexts): { text: ICnText; separator: string } {
    const own = texts.trait(part.text, part.kind, part.blackboard);
    const base = operator.description ?? "";
    if (part.kind === "override" || base.length === 0) return { text: own, separator: "" };
    if (own.text !== null) return { text: { text: `${base}\n${own.text}`, zh: own.zh }, separator: "" };
    return { text: { text: base, zh: own.zh }, separator: "<br/>" };
}

function ModuleTraitChange({ operator, phase, descriptionBlackboard, showDiff, onShowDiffChange, cnTexts, cnPhase, cnOperator }: IModuleTraitChangeProps) {
    const t: InfoT = useT("operators");
    const change = traitChange(operator, phase, descriptionBlackboard);
    if (!change) return null;
    const cn = cnTexts ? cnTraitText(operator, change.part, cnTexts) : null;
    const rendered = cn ? renderCnChange(change.base, cn.text, change.next.blackboard, showDiff, cn.separator) : renderChange(change.base, change.next, showDiff);
    const canDiff = change.base !== null && (cn === null || cn.text.text !== null);
    const cnChange = cnPhase && cnOperator ? traitChange(cnOperator, cnPhase, combinedDescriptionBlackboard(cnOperator)) : null;
    const cnHtml = cnChange ? cnNumbersHtml(change.next.description, change.next.blackboard, cnChange.next.description, cnChange.next.blackboard) : null;
    return (
        <>
            <div className="mb-1 flex items-center justify-between">
                <h6 className="font-medium text-foreground text-xs">{t("info.traitChanges")}</h6>
                {canDiff && (
                    <Tooltip>
                        <TooltipTrigger
                            render={(props) => (
                                <span {...props}>
                                    <Switch aria-label={t("info.showDiff.tip")} checked={showDiff} onCheckedChange={onShowDiffChange} />
                                </span>
                            )}
                        />
                        <TooltipPopup>{t("info.showDiff.tip")}</TooltipPopup>
                    </Tooltip>
                )}
            </div>
            <div className="rounded-md bg-secondary/20 p-2">
                <DescriptionDiff {...rendered} baseLabel={t("info.baseTrait")} />
                {cnHtml && <CnDescription {...cnHtml} />}
            </div>
        </>
    );
}

interface IModuleTalentChangesProps {
    operator: IOperatorListItem;
    phase: IModulePhase;
    showDiff: boolean;
    cnTexts: ICnModuleTexts | null;
    cnPhase: IModulePhase | null;
}

function ModuleTalentChanges({ operator, phase, showDiff, cnTexts, cnPhase }: IModuleTalentChangesProps) {
    const t: InfoT = useT("operators");
    const candidates = moduleTalentCandidates(phase);
    if (candidates.length === 0) return null;
    const cnCandidates = cnPhase ? moduleTalentCandidates(cnPhase) : [];
    return (
        <div>
            <h6 className="mb-1 font-medium text-foreground text-xs">{t("info.talentChanges")}</h6>
            {candidates.map((c, cIdx) => {
                const oldTalent = operator.talents?.[c.talentIndex];
                const oldCand = oldTalent?.candidates?.[oldTalent.candidates.length - 1];
                const base = oldCand?.description ? { description: oldCand.description, blackboard: oldCand.blackboard ?? [] } : null;
                const next = { description: c.upgradeDescription || c.description || "", blackboard: c.blackboard ?? [] };
                const cn = cnTexts?.talent({ talentIndex: c.talentIndex, requiredPotentialRank: c.requiredPotentialRank, name: c.name, description: next.description, blackboard: next.blackboard });
                const rendered = cn ? renderCnChange(base, cn.description, next.blackboard, showDiff) : renderChange(base, next, showDiff);
                const shownName = cn ? (cn.name.text ?? cn.name.zh) : c.name;
                const name = shownName ? (
                    <span lang={cn && cn.name.text === null ? "zh-CN" : undefined} className="font-medium text-foreground text-xs">
                        {shownName}:{" "}
                    </span>
                ) : null;
                // The same candidate of a module both servers have, where CN changed the module.
                const onCn = cnCandidates[cIdx];
                const cnHtml = onCn && onCn.talentIndex === c.talentIndex && onCn.requiredPotentialRank === c.requiredPotentialRank ? cnNumbersHtml(next.description, next.blackboard, onCn.upgradeDescription || onCn.description, onCn.blackboard ?? []) : null;
                return (
                    // biome-ignore lint/suspicious/noArrayIndexKey: candidate order is determined by module phase data
                    <div className="rounded-md bg-secondary/20 px-2 py-1" key={`tc-${cIdx}-${c.name ?? ""}`}>
                        <DescriptionDiff {...rendered} baseLabel={t("info.baseTalent")} prefix={name} />
                        {cnHtml && <CnDescription {...cnHtml} />}
                    </div>
                );
            })}
        </div>
    );
}
