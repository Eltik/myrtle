import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Dna, Heart, Info, LibraryBig, type LucideIcon, MapPin, Package, Palette, User } from "lucide-react";
import { memo, type ReactNode, useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Separator } from "#/components/ui/separator";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { operatorGamedataServer } from "#/lib/api/gamedata";
import { rangesQueryOptions } from "#/lib/api/ranges";
import { GameText } from "#/lib/gamedata/GameText";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, rarityToNumber } from "#/lib/utils";
import type { IModulePart, IModulePhase, IOperatorListItem, IOperatorModule } from "#/types/operators";
import { asset, eliteIcon, potentialIcon } from "../../assets";
import type { messages as detailConstantsMessages } from "../../constants.messages";
import { descriptionToHtml, renderDescriptionDiffHtml } from "../../description";
import { combinedDescriptionBlackboard, formatAttributeKey, formatStatValue, getActiveTalentCandidate, getOperatorAttributeStats } from "../../helpers";
import type { messages as helperMessages } from "../../helpers.messages";
import { useCommunityDefaults } from "../../useCommunityDefaults";
import { BaseSkillsSection } from "../BaseSkillsSection";
import { CommunitySharePill } from "../CommunitySharePill";
import { LevelBreakdown } from "../LevelBreakdown";
import { OperatorNotes } from "../OperatorNotes";
import { OperatorRange } from "../OperatorRange";
import { SummonsSection } from "../SummonsSection";
import type { messages } from "./InfoContent.messages";

interface IInfoContentProps {
    operator: IOperatorListItem;
}

type Stat = { iconURL: string; label: string; value: string };

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
function traitChange(operator: IOperatorListItem, phase: IModulePhase, descriptionBlackboard: Blackboard): { base: IDescribed | null; next: IDescribed } | null {
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
        };
    }

    const baseDesc = operator.description ?? "";
    return {
        base: baseDesc.length > 0 ? { description: baseDesc, blackboard: descriptionBlackboard } : null,
        next: { description: baseDesc.length > 0 ? `${baseDesc}\n${additionalDesc}` : additionalDesc, blackboard: [...descriptionBlackboard, ...nextBlackboard] },
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
 */
function buildTalentEffects(operator: IOperatorListItem, phaseIndex: number, level: number, potentialRank: number, module: IOperatorModule | null, moduleLevel: number): ITalentEffect[] {
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
            const tIdx = chosen.talentIndex;
            if (tIdx >= 0 && tIdx < effects.length) {
                effects[tIdx] = { ...effects[tIdx], name: chosen.name ?? effects[tIdx].name, description, blackboard: chosen.blackboard ?? [], modifiedByModule: true };
            } else {
                effects.push({
                    key: `t-mod-${tIdx}-${i}`,
                    name: chosen.name,
                    description,
                    blackboard: chosen.blackboard ?? [],
                    requiredPotentialRank: chosen.requiredPotentialRank ?? 0,
                    base: null,
                    modifiedByModule: true,
                });
            }
        }
    }
    return effects;
}

export const InfoContent = memo(function InfoContent({ operator }: IInfoContentProps) {
    const t: InfoT = useT("operators");
    const f = useFormatters();
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(operator.phases[operator.phases.length - 1]?.maxLevel ?? 1);
    const [trustLevel, setTrustLevel] = useState(100);
    const [potentialRank, setPotentialRank] = useState(rarityToNumber(operator.rarity) <= 4 ? 5 : 0);
    const [moduleId, setModuleId] = useState(operator.modules.find((m) => m.type !== "INITIAL")?.uniEquipId ?? "");
    const [moduleLevel, setModuleLevel] = useState(() => {
        const initialModule = operator.modules.find((m) => m.type !== "INITIAL");
        return initialModule?.data?.phases?.length ?? 0;
    });

    // Set once the viewer picks a module themselves, or once a default lands,
    // so late-arriving community data cannot override a deliberate choice.
    const moduleSettled = useRef(false);

    const localeServer = useGamedataServer();
    const { data: ranges } = useQuery(rangesQueryOptions(operatorGamedataServer(operator.server, localeServer)));
    const currentRange = ranges?.[operator.phases[phaseIndex]?.rangeId ?? ""];

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

    const talents = useMemo(() => buildTalentEffects(operator, phaseIndex, level, potentialRank, currentModule, moduleLevel).filter((eff) => eff.name || eff.description), [operator, phaseIndex, level, potentialRank, currentModule, moduleLevel]);

    const fmt = (n: number | undefined) => (typeof n === "number" ? f.number(Math.round(n)) : (n ?? "-"));

    const leftStats: Stat[] = [
        { iconURL: "/stat-icons/HP.png", label: t("stat.health"), value: fmt(stats?.maxHp) },
        { iconURL: "/stat-icons/DEF.png", label: t("stat.defense"), value: fmt(stats?.def) },
        { iconURL: "/stat-icons/RES.png", label: t("stat.artsResistance"), value: fmt(stats?.magicResistance) },
        { iconURL: "/stat-icons/RDP.png", label: t("stat.redeployTime"), value: t("stat.seconds", { value: stats?.respawnTime ?? 0 }) },
    ];
    const rightStats: Stat[] = [
        { iconURL: "/stat-icons/ATK.png", label: t("stat.attackPower"), value: fmt(stats?.atk) },
        { iconURL: "/stat-icons/ASPD.png", label: t("stat.attackInterval"), value: t("stat.seconds", { value: stats?.attackSpeed?.toFixed(2) ?? "0.00" }) },
        { iconURL: "/stat-icons/BLOCK.png", label: t("stat.block"), value: fmt(stats?.blockCnt) },
        { iconURL: "/stat-icons/COST.png", label: t("stat.dpCost"), value: fmt(stats?.cost) },
    ];

    const profile = operator.profile?.basicInfo;
    const unknown = t("info.profile.unknown");

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">{t("info.title")}</h2>
                <p
                    className="wrap-break-word text-muted-foreground text-sm"
                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                    dangerouslySetInnerHTML={{ __html: description }}
                />
            </div>

            <CollapsibleSection icon={User} title={t("info.profile")} open={showProfile} onOpenChange={setShowProfile}>
                <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
                    {profile && (
                        <>
                            <ProfileCard icon={MapPin} label={t("info.profile.placeOfBirth")} value={profile.placeOfBirth ?? unknown} />
                            <ProfileCard icon={Dna} label={t("info.profile.race")} value={profile.race ?? unknown} />
                            <ProfileCard icon={User} label={t("info.profile.gender")} value={profile.gender ?? unknown} />
                            <ProfileCard icon={Info} label={t("info.profile.height")} value={profile.height ?? unknown} />
                        </>
                    )}
                    {operator.artists && operator.artists.length > 0 && <ProfileCard icon={Palette} label={t("info.profile.artist")} value={operator.artists.join(", ")} />}
                </div>
            </CollapsibleSection>

            <Separator className="my-6" />

            <OperatorNotes operatorId={operator.id ?? null} />

            <Separator className="my-6" />

            <CollapsibleSection icon={Package} title={t("info.controls")} open={showControls} onOpenChange={setShowControls}>
                <div className="mt-3 space-y-4 rounded-lg border border-border/50 bg-card/30 p-4">
                    <p className="text-muted-foreground text-xs">{t("info.controls.desc")}</p>

                    <div className="flex flex-wrap items-center gap-2">
                        <span className="text-muted-foreground text-sm">{t("info.promotion")}</span>
                        {operator.phases.map((_, idx) => (
                            <button
                                className={cn("flex h-10 w-10 items-center justify-center rounded-lg border transition-colors", phaseIndex === idx ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}
                                // biome-ignore lint/suspicious/noArrayIndexKey: phase ordering is fixed by elite tier
                                key={`phase-${idx}`}
                                onClick={() => {
                                    setPhaseIndex(idx);
                                    setLevel(operator.phases[idx].maxLevel);
                                }}
                                type="button"
                            >
                                <img alt={t("info.eliteAlt", { elite: idx })} className="icon-theme-aware h-6 w-6 object-contain" decoding="async" loading="lazy" src={eliteIcon(idx, operator.server)} />
                            </button>
                        ))}
                    </div>

                    <div className="grid gap-4 md:grid-cols-2">
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="text-muted-foreground text-sm">{t("info.level")}</span>
                                <span className="font-mono text-foreground text-sm">
                                    {level} / {operator.phases[phaseIndex]?.maxLevel ?? 1}
                                </span>
                            </div>
                            <Slider min={1} max={operator.phases[phaseIndex]?.maxLevel ?? 1} step={1} value={[level]} onValueChange={(v) => setLevel(Array.isArray(v) ? (v[0] ?? 1) : v)} />
                        </div>
                        <div className="space-y-2">
                            <div className="flex items-center justify-between">
                                <span className="flex items-center gap-1 text-muted-foreground text-sm">
                                    <Heart className="h-3.5 w-3.5" /> {t("info.trust")}
                                </span>
                                <span className="font-mono text-foreground text-sm">{trustLevel}%</span>
                            </div>
                            <Slider min={0} max={200} step={1} value={[trustLevel]} onValueChange={(v) => setTrustLevel(Array.isArray(v) ? (v[0] ?? 100) : v)} />
                        </div>
                    </div>

                    <div className="space-y-2">
                        <div className="flex items-center gap-2 text-muted-foreground text-sm">
                            {t("info.potential")}
                            <Tooltip>
                                <TooltipTrigger render={(props) => <Info className="h-3 w-3 cursor-help text-muted-foreground" {...props} />} />
                                <TooltipPopup>{t("info.potential.tip")}</TooltipPopup>
                            </Tooltip>
                        </div>
                        <div className="flex flex-wrap gap-1.5">
                            {Array.from({
                                length: (operator.potentialRanks?.length ?? 0) + 1,
                            }).map((_, idx) => {
                                const rank = idx === 0 ? null : operator.potentialRanks?.[idx - 1];
                                return (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: potential rank is positionally stable
                                    <Tooltip key={`pot-${idx}`}>
                                        <TooltipTrigger
                                            render={(props) => (
                                                <button {...props} type="button" onClick={() => setPotentialRank(idx)} className={cn("flex h-8 w-8 items-center justify-center rounded-md border transition-colors", potentialRank === idx ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}>
                                                    <img alt={t("info.potential.alt", { rank: idx })} className="icon-theme-aware h-5 w-5 object-contain" decoding="async" loading="lazy" src={potentialIcon(idx, operator.server)} />
                                                </button>
                                            )}
                                        />
                                        <TooltipPopup>{rank ? t("info.potential.rankTip", { rank: idx, description: rank.description }) : t("info.potential.baseTip", { rank: idx })}</TooltipPopup>
                                    </Tooltip>
                                );
                            })}
                        </div>
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
                                                return m ? moduleDesignator(m) : value;
                                            }}
                                        </SelectValue>
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="none">{t("info.module.none")}</SelectItem>
                                        {availableModules.map((mod) => (
                                            <SelectItem key={mod.uniEquipId} value={mod.uniEquipId}>
                                                <span className="flex items-center gap-2">
                                                    {moduleDesignator(mod)}
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
                            {column.map(({ iconURL, label, value }) => (
                                <div key={label} className="flex items-center justify-between py-2">
                                    <span className="flex items-center gap-2 text-muted-foreground">
                                        <img alt={label} src={iconURL} className="icon-theme-aware h-4 w-4 object-contain" decoding="async" loading="lazy" />
                                        <span className="text-sm">{label}</span>
                                    </span>
                                    <span className="font-semibold text-foreground tabular-nums">{value}</span>
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
                {currentRange ? <OperatorRange range={currentRange} /> : <p className="text-muted-foreground text-sm">{t("info.attackRange.empty")}</p>}
            </div>

            {showModuleControls && currentModule && (
                <div className="mb-6">
                    <CollapsibleSection icon={Package} title={t("info.moduleDetails")} open={showModuleDetails} onOpenChange={setShowModuleDetails} chevron>
                        <div className="mt-3 rounded-lg border border-border/50 bg-card/30 p-4">
                            <div className="mb-4 flex items-center gap-3">
                                {currentModule.image && <img alt={currentModule.uniEquipName} className="h-16 w-16 rounded-md object-contain" decoding="async" loading="lazy" src={asset(currentModule.image, operator.server)} />}
                                <div>
                                    <h4 className="font-semibold text-foreground">{currentModule.uniEquipName}</h4>
                                    <div className="mt-1 flex gap-1">
                                        <Badge variant="outline">{currentModule.typeName1}</Badge>
                                        {currentModule.typeName2 && <Badge variant="outline">{currentModule.typeName2}</Badge>}
                                    </div>
                                </div>
                            </div>
                            {currentModule.uniEquipDesc && (
                                <div className="mb-4 max-h-32 overflow-y-auto rounded-md bg-secondary/20 p-3">
                                    <p className="whitespace-pre-line text-muted-foreground text-xs leading-relaxed">
                                        <GameText text={currentModule.uniEquipDesc} />
                                    </p>
                                </div>
                            )}
                            {currentModulePhase ? (
                                <div className="space-y-3">
                                    <h5 className="font-medium text-foreground text-sm">{t("info.moduleDetails.levelStats", { level: moduleLevel })}</h5>
                                    {currentModulePhase.attributeBlackboard.length > 0 ? (
                                        <div className="grid grid-cols-2 gap-2 md:grid-cols-3">
                                            {currentModulePhase.attributeBlackboard.map((attr) => (
                                                <div className="rounded-md bg-secondary/30 p-2" key={`${attr.key}-${moduleLevel}`}>
                                                    <span className="text-muted-foreground text-xs">{formatAttributeKey(attr.key, t)}:</span>
                                                    <span className="ml-1 font-medium text-foreground text-sm">{formatStatValue(attr.value)}</span>
                                                </div>
                                            ))}
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
                                    <ModuleTraitChange operator={operator} phase={currentModulePhase} descriptionBlackboard={descriptionBlackboard} showDiff={showDiff} onShowDiffChange={setShowDiff} />
                                    <ModuleTalentChanges operator={operator} phase={currentModulePhase} showDiff={showDiff} />
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
                            const change = renderChange(talent.modifiedByModule ? talent.base : null, talent, showDiff);
                            return (
                                <div className="rounded-lg border border-border/50 bg-card/30 p-3" key={talent.key}>
                                    <div className="mb-1 flex items-center gap-2">
                                        <h4 className="font-medium text-foreground text-sm">{talent.name ?? t("info.talents.unnamed")}</h4>
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
                                    </div>
                                    <DescriptionDiff {...change} baseLabel="Base Talent" />
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
    /** Show the open/closed chevron; the always-open-by-default sections omit it. */
    chevron?: boolean;
    children: ReactNode;
}

function CollapsibleSection({ icon: Icon, title, open, onOpenChange, chevron = false, children }: ICollapsibleSectionProps) {
    return (
        <Collapsible open={open} onOpenChange={onOpenChange}>
            <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                <span className="flex items-center gap-2">
                    <Icon className="h-4 w-4 text-primary" />
                    <span className="font-medium text-sm">{title}</span>
                </span>
                {chevron && <ChevronDown className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />}
            </CollapsibleTrigger>
            <CollapsibleContent>{children}</CollapsibleContent>
        </Collapsible>
    );
}

function ProfileCard({ icon: Icon, label, value }: { icon: LucideIcon; label: string; value: string }) {
    return (
        <div className="rounded-lg border border-border/50 bg-card/40 p-3">
            <div className="flex items-center gap-1.5 text-muted-foreground">
                <Icon className="h-3.5 w-3.5" />
                <span className="text-xs">{label}</span>
            </div>
            <div className="mt-1 truncate text-foreground text-sm">{value}</div>
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

/** A description a module changed, shown as the faded base above the marked-up new text. */
function DescriptionDiff({ html, baseHtml, baseLabel, prefix }: IDescriptionDiffProps) {
    return (
        <>
            {baseHtml && (
                <div className="mb-2 border-border/40 border-b pb-2 opacity-50">
                    <div className={DIFF_LABEL_CLASS}>{baseLabel}</div>
                    {prefix}
                    <span
                        className={cn("text-muted-foreground text-xs", !prefix && "block")}
                        // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                        dangerouslySetInnerHTML={{ __html: baseHtml }}
                    />
                </div>
            )}
            {baseHtml && <div className={DIFF_LABEL_CLASS}>Changes</div>}
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
}

function ModuleTraitChange({ operator, phase, descriptionBlackboard, showDiff, onShowDiffChange }: IModuleTraitChangeProps) {
    const t: InfoT = useT("operators");
    const change = traitChange(operator, phase, descriptionBlackboard);
    if (!change) return null;
    const rendered = renderChange(change.base, change.next, showDiff);
    return (
        <>
            <div className="mb-1 flex items-center justify-between">
                <h6 className="font-medium text-foreground text-xs">{t("info.traitChanges")}</h6>
                <Tooltip>
                    <TooltipTrigger
                        render={(props) => (
                            <span {...props}>
                                <Switch checked={showDiff} onCheckedChange={onShowDiffChange} />
                            </span>
                        )}
                    />
                    <TooltipPopup>{t("info.showDiff.tip")}</TooltipPopup>
                </Tooltip>
            </div>
            <div className="rounded-md bg-secondary/20 p-2">
                <DescriptionDiff {...rendered} baseLabel={t("info.baseTrait")} />
            </div>
        </>
    );
}

function ModuleTalentChanges({ operator, phase, showDiff }: { operator: IOperatorListItem; phase: IModulePhase; showDiff: boolean }) {
    const t: InfoT = useT("operators");
    const candidates = moduleTalentCandidates(phase);
    if (candidates.length === 0) return null;
    return (
        <div>
            <h6 className="mb-1 font-medium text-foreground text-xs">{t("info.talentChanges")}</h6>
            {candidates.map((c, cIdx) => {
                const oldTalent = operator.talents?.[c.talentIndex];
                const oldCand = oldTalent?.candidates?.[oldTalent.candidates.length - 1];
                const base = oldCand?.description ? { description: oldCand.description, blackboard: oldCand.blackboard ?? [] } : null;
                const rendered = renderChange(base, { description: c.upgradeDescription || c.description || "", blackboard: c.blackboard ?? [] }, showDiff);
                const name = c.name ? <span className="font-medium text-foreground text-xs">{c.name}: </span> : null;
                return (
                    // biome-ignore lint/suspicious/noArrayIndexKey: candidate order is determined by module phase data
                    <div className="rounded-md bg-secondary/20 px-2 py-1" key={`tc-${cIdx}-${c.name ?? ""}`}>
                        <DescriptionDiff {...rendered} baseLabel="Base Talent" prefix={name} />
                    </div>
                );
            })}
        </div>
    );
}
