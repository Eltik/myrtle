import { useQuery } from "@tanstack/react-query";
import { ChevronDown, Dna, Heart, Info, LibraryBig, MapPin, Package, Palette, User } from "lucide-react";
import { memo, useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "#/components/ui/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Separator } from "#/components/ui/separator";
import { Slider } from "#/components/ui/slider";
import { Switch } from "#/components/ui/switch";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { rangesQueryOptions } from "#/lib/api/ranges";
import { cn, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { asset, eliteIcon, potentialIcon } from "../../assets";
import { descriptionToHtml, renderDescriptionDiffHtml } from "../../description";
import { combinedDescriptionBlackboard, formatAttributeKey, formatStatValue, getActiveTalentCandidate, getOperatorAttributeStats } from "../../helpers";
import { OperatorRange } from "../OperatorRange";

interface IInfoContentProps {
    operator: IOperatorListItem;
}

type Stat = { iconURL: string; label: string; value: string };

export const InfoContent = memo(function InfoContent({ operator }: IInfoContentProps) {
    const [phaseIndex, setPhaseIndex] = useState(operator.phases.length - 1);
    const [level, setLevel] = useState(operator.phases[operator.phases.length - 1]?.maxLevel ?? 1);
    const [trustLevel, setTrustLevel] = useState(100);
    const [potentialRank, setPotentialRank] = useState(rarityToNumber(operator.rarity) <= 4 ? 5 : 0);
    const [moduleId, setModuleId] = useState(operator.modules.find((m) => m.type !== "INITIAL")?.uniEquipId ?? "");
    const [moduleLevel, setModuleLevel] = useState(() => {
        const initialModule = operator.modules.find((m) => m.type !== "INITIAL");
        return initialModule?.data?.phases?.length ?? 0;
    });

    const { data: ranges } = useQuery(rangesQueryOptions());
    const currentRange = ranges?.[operator.phases[phaseIndex]?.rangeId ?? ""];

    const availableModules = useMemo(() => operator.modules.filter((m) => m.type !== "INITIAL"), [operator.modules]);
    const currentModule = useMemo(() => (moduleId && moduleId.length > 0 ? (availableModules.find((m) => m.uniEquipId === moduleId) ?? null) : null), [moduleId, availableModules]);

    const [showProfile, setShowProfile] = useState(true);
    const [showControls, setShowControls] = useState(true);
    const [showModuleDetails, setShowModuleDetails] = useState(true);
    const [showDiff, setShowDiff] = useState(true);
    const [showTalents, setShowTalents] = useState(true);

    const descriptionBlackboard = useMemo(() => combinedDescriptionBlackboard(operator), [operator]);
    const description = useMemo(() => descriptionToHtml(operator.description ?? "", descriptionBlackboard), [operator.description, descriptionBlackboard]);

    const stats = useMemo(() => getOperatorAttributeStats(operator, { phaseIndex, favorPoint: trustLevel, potentialRank, moduleId, moduleLevel }, level), [operator, phaseIndex, level, trustLevel, potentialRank, moduleId, moduleLevel]);

    const fmt = (n: number | undefined) => (typeof n === "number" ? Math.round(n).toLocaleString() : (n ?? "—"));

    const leftStats: Stat[] = [
        { iconURL: "/stat-icons/HP.png", label: "Health", value: fmt(stats?.maxHp) },
        { iconURL: "/stat-icons/DEF.png", label: "Defense", value: fmt(stats?.def) },
        { iconURL: "/stat-icons/RES.png", label: "Arts Resistance", value: fmt(stats?.magicResistance) },
        { iconURL: "/stat-icons/RDP.png", label: "Redeploy Time", value: `${stats?.respawnTime ?? 0} sec` },
    ];
    const rightStats: Stat[] = [
        { iconURL: "/stat-icons/ATK.png", label: "Attack Power", value: fmt(stats?.atk) },
        { iconURL: "/stat-icons/ASPD.png", label: "Attack Interval", value: `${stats?.attackSpeed?.toFixed(2) ?? "0.00"} sec` },
        { iconURL: "/stat-icons/BLOCK.png", label: "Block", value: fmt(stats?.blockCnt) },
        { iconURL: "/stat-icons/COST.png", label: "DP Cost", value: fmt(stats?.cost) },
    ];

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Operator Information</h2>
                <p
                    className="text-muted-foreground text-sm wrap-break-word"
                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
                    dangerouslySetInnerHTML={{ __html: description }}
                />
            </div>

            <Collapsible open={showProfile} onOpenChange={setShowProfile}>
                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                    <span className="flex items-center gap-2">
                        <User className="h-4 w-4 text-primary" />
                        <span className="font-medium text-sm">Profile</span>
                    </span>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <div className="mt-3 grid grid-cols-2 gap-3 md:grid-cols-4">
                        {operator.profile?.basicInfo && (
                            <>
                                <div className="rounded-lg border border-border/50 bg-card/40 p-3">
                                    <div className="flex items-center gap-1.5 text-muted-foreground">
                                        <MapPin className="h-3.5 w-3.5" />
                                        <span className="text-xs">Place of Birth</span>
                                    </div>
                                    <div className="mt-1 truncate text-foreground text-sm">{operator.profile.basicInfo.placeOfBirth ?? "Unknown"}</div>
                                </div>
                                <div className="rounded-lg border border-border/50 bg-card/40 p-3">
                                    <div className="flex items-center gap-1.5 text-muted-foreground">
                                        <Dna className="h-3.5 w-3.5" />
                                        <span className="text-xs">Race</span>
                                    </div>
                                    <div className="mt-1 truncate text-foreground text-sm">{operator.profile.basicInfo.race ?? "Unknown"}</div>
                                </div>
                                <div className="rounded-lg border border-border/50 bg-card/40 p-3">
                                    <div className="flex items-center gap-1.5 text-muted-foreground">
                                        <User className="h-3.5 w-3.5" />
                                        <span className="text-xs">Gender</span>
                                    </div>
                                    <div className="mt-1 truncate text-foreground text-sm">{operator.profile.basicInfo.gender ?? "Unknown"}</div>
                                </div>
                                <div className="rounded-lg border border-border/50 bg-card/40 p-3">
                                    <div className="flex items-center gap-1.5 text-muted-foreground">
                                        <Info className="h-3.5 w-3.5" />
                                        <span className="text-xs">Height</span>
                                    </div>
                                    <div className="mt-1 truncate text-foreground text-sm">{operator.profile.basicInfo.height ?? "Unknown"}</div>
                                </div>
                            </>
                        )}
                        {operator.artists && operator.artists.length > 0 && (
                            <div className="rounded-lg border border-border/50 bg-card/40 p-3">
                                <div className="flex items-center gap-1.5 text-muted-foreground">
                                    <Palette className="h-3.5 w-3.5" />
                                    <span className="text-xs">Artist</span>
                                </div>
                                <div className="mt-1 truncate text-foreground text-sm">{operator.artists.join(", ")}</div>
                            </div>
                        )}
                    </div>
                </CollapsibleContent>
            </Collapsible>

            <Separator className="my-6" />

            <Collapsible open={showControls} onOpenChange={setShowControls}>
                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                    <span className="flex items-center gap-2">
                        <Package className="h-4 w-4 text-primary" />
                        <span className="font-medium text-sm">Operator Controls</span>
                    </span>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    <div className="mt-3 space-y-4 rounded-lg border border-border/50 bg-card/30 p-4">
                        <p className="text-muted-foreground text-xs">Adjust to see how stats change at different levels, promotions, potentials, modules, and trust.</p>

                        <div className="flex flex-wrap items-center gap-2">
                            <span className="text-muted-foreground text-sm">Promotion:</span>
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
                                    <img alt={`Elite ${idx}`} className="h-6 w-6 object-contain icon-theme-aware" decoding="async" loading="lazy" src={eliteIcon(idx)} />
                                </button>
                            ))}
                        </div>

                        <div className="grid gap-4 md:grid-cols-2">
                            <div className="space-y-2">
                                <div className="flex items-center justify-between">
                                    <span className="text-muted-foreground text-sm">Level</span>
                                    <span className="font-mono text-foreground text-sm">
                                        {level} / {operator.phases[phaseIndex]?.maxLevel ?? 1}
                                    </span>
                                </div>
                                <Slider min={1} max={operator.phases[phaseIndex]?.maxLevel ?? 1} step={1} value={[level]} onValueChange={(v) => setLevel(Array.isArray(v) ? (v[0] ?? 1) : v)} />
                            </div>
                            <div className="space-y-2">
                                <div className="flex items-center justify-between">
                                    <span className="flex items-center gap-1 text-muted-foreground text-sm">
                                        <Heart className="h-3.5 w-3.5" /> Trust
                                    </span>
                                    <span className="font-mono text-foreground text-sm">{trustLevel}%</span>
                                </div>
                                <Slider min={0} max={200} step={1} value={[trustLevel]} onValueChange={(v) => setTrustLevel(Array.isArray(v) ? (v[0] ?? 100) : v)} />
                            </div>
                        </div>

                        <div className="space-y-2">
                            <div className="flex items-center gap-2 text-muted-foreground text-sm">
                                Potential
                                <Tooltip>
                                    <TooltipTrigger render={(props) => <Info className="h-3 w-3 cursor-help text-muted-foreground" {...props} />} />
                                    <TooltipPopup>Select a potential rank to see stat bonuses.</TooltipPopup>
                                </Tooltip>
                            </div>
                            <div className="flex flex-wrap gap-1.5">
                                {Array.from({ length: (operator.potentialRanks?.length ?? 0) + 1 }).map((_, idx) => {
                                    const rank = idx === 0 ? null : operator.potentialRanks?.[idx - 1];
                                    return (
                                        // biome-ignore lint/suspicious/noArrayIndexKey: potential rank is positionally stable
                                        <Tooltip key={`pot-${idx}`}>
                                            <TooltipTrigger
                                                render={(props) => (
                                                    <button
                                                        {...props}
                                                        type="button"
                                                        onClick={() => setPotentialRank(idx)}
                                                        className={cn("flex h-8 w-8 items-center justify-center rounded-md border transition-colors", potentialRank === idx ? "border-primary bg-primary/10" : "border-border bg-card hover:border-primary/50")}
                                                    >
                                                        <img alt={`Pot ${idx}`} className="h-5 w-5 object-contain" decoding="async" loading="lazy" src={potentialIcon(idx)} />
                                                    </button>
                                                )}
                                            />
                                            <TooltipPopup>{rank ? `Pot ${idx}: ${rank.description}` : `Pot ${idx}: Base potential`}</TooltipPopup>
                                        </Tooltip>
                                    );
                                })}
                            </div>
                        </div>

                        {phaseIndex === 2 && availableModules.length > 0 && (
                            <div className="grid gap-3 md:grid-cols-2">
                                <div className="space-y-1">
                                    <span className="text-muted-foreground text-xs">Module</span>
                                    <Select
                                        value={moduleId || "none"}
                                        onValueChange={(v) => {
                                            const value = String(v);
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
                                            <SelectValue placeholder="Select module">
                                                {(value: string) => {
                                                    if (value === "none") return "No Module";
                                                    const m = availableModules.find((x) => x.uniEquipId === value);
                                                    if (!m) return value;
                                                    return m.typeName1 && m.typeName2 ? `${m.typeName1}-${m.typeName2}` : m.uniEquipName;
                                                }}
                                            </SelectValue>
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="none">No Module</SelectItem>
                                            {availableModules.map((mod) => (
                                                <SelectItem key={mod.uniEquipId} value={mod.uniEquipId}>
                                                    {mod.typeName1 && mod.typeName2 ? `${mod.typeName1}-${mod.typeName2}` : mod.uniEquipName}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                                {currentModule?.data?.phases && currentModule.data.phases.length > 0 && (
                                    <div className="space-y-1">
                                        <span className="text-muted-foreground text-xs">Module Level</span>
                                        <Select value={String(moduleLevel)} onValueChange={(v) => setModuleLevel(Number.parseInt(String(v), 10))}>
                                            <SelectTrigger>
                                                <SelectValue placeholder="Select level">{(value: string) => `Level ${value}`}</SelectValue>
                                            </SelectTrigger>
                                            <SelectContent>
                                                {currentModule.data.phases.map((phase) => (
                                                    <SelectItem key={phase.equipLevel} value={String(phase.equipLevel)}>
                                                        Level {phase.equipLevel}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                </CollapsibleContent>
            </Collapsible>

            <Separator className="my-6" />

            <div className="mb-6">
                <h3 className="mb-4 font-medium text-foreground">Combat Stats</h3>
                <div className="grid grid-cols-1 divide-y divide-border rounded-lg border border-border bg-card md:grid-cols-2 md:divide-x md:divide-y-0">
                    {[leftStats, rightStats].map((column, colIdx) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: fixed two-column split, order is stable
                        <div key={colIdx} className="px-5 py-2">
                            {column.map(({ iconURL, label, value }) => (
                                <div key={label} className="flex items-center justify-between py-2">
                                    <span className="flex items-center gap-2 text-muted-foreground">
                                        <img alt={label} src={iconURL} className="h-4 w-4 object-contain icon-theme-aware" decoding="async" loading="lazy" />
                                        <span className="text-sm">{label}</span>
                                    </span>
                                    <span className="font-semibold tabular-nums text-foreground">{value}</span>
                                </div>
                            ))}
                        </div>
                    ))}
                </div>
            </div>
            {operator.tagList && operator.tagList.length > 0 && (
                <div className="mb-6">
                    <h3 className="mb-3 font-medium text-foreground">Tags</h3>
                    <div className="flex flex-wrap gap-2">
                        {operator.tagList.map((t) => (
                            <Badge key={t} variant="secondary" className="bg-accent">
                                {t}
                            </Badge>
                        ))}
                    </div>
                </div>
            )}
            <div className="mb-6">
                <h3 className="mb-3 font-medium text-foreground">Attack Range</h3>
                {currentRange ? <OperatorRange range={currentRange} /> : <p className="text-muted-foreground text-sm">No range data available.</p>}
            </div>

            {phaseIndex === 2 && currentModule && (
                <div className="mb-6">
                    <Collapsible open={showModuleDetails} onOpenChange={setShowModuleDetails}>
                        <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                            <span className="flex items-center gap-2">
                                <Package className="h-4 w-4 text-primary" />
                                <span className="font-medium text-sm">Module Details</span>
                            </span>
                            <ChevronDown className={cn("h-4 w-4 transition-transform", showModuleDetails && "rotate-180")} />
                        </CollapsibleTrigger>
                        <CollapsibleContent>
                            <div className="mt-3 rounded-lg border border-border/50 bg-card/30 p-4">
                                <div className="mb-4 flex items-center gap-3">
                                    {currentModule.image && <img alt={currentModule.uniEquipName} className="h-16 w-16 rounded-md object-contain" decoding="async" loading="lazy" src={asset(currentModule.image)} />}
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
                                        <p className="whitespace-pre-line text-muted-foreground text-xs leading-relaxed">{currentModule.uniEquipDesc}</p>
                                    </div>
                                )}
                                {moduleLevel > 0 && currentModule.data?.phases?.[moduleLevel - 1] ? (
                                    <div className="space-y-3">
                                        <h5 className="font-medium text-foreground text-sm">Level {moduleLevel} Stats</h5>
                                        {currentModule.data?.phases?.[moduleLevel - 1].attributeBlackboard && currentModule.data?.phases?.[moduleLevel - 1].attributeBlackboard.length > 0 ? (
                                            <div className="grid grid-cols-2 gap-2 md:grid-cols-3">
                                                {currentModule.data?.phases?.[moduleLevel - 1].attributeBlackboard.map((attr) => (
                                                    <div className="rounded-md bg-secondary/30 p-2" key={`${attr.key}-${moduleLevel}`}>
                                                        <span className="text-muted-foreground text-xs">{formatAttributeKey(attr.key)}:</span>
                                                        <span className="ml-1 font-medium text-foreground text-sm">{formatStatValue(attr.value)}</span>
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <p className="text-muted-foreground text-xs">No stat bonuses at this level.</p>
                                        )}
                                    </div>
                                ) : (
                                    <p className="text-muted-foreground text-xs">Select a module level to see its effects.</p>
                                )}

                                {moduleLevel > 0 && currentModule.data?.phases?.[moduleLevel - 1] ? (
                                    <div className="space-y-2">
                                        {(() => {
                                            const phase = currentModule.data.phases[moduleLevel - 1];
                                            const traitPart = phase.parts.find((p) => p.target === "TRAIT" || p.target === "TRAIT_DATA_ONLY" || p.target === "DISPLAY");
                                            const newTraitCand = traitPart?.overrideTraitDataBundle.candidates?.[0];
                                            if (!newTraitCand) return null;
                                            const overrideDesc = newTraitCand.overrideDescription ?? "";
                                            const additionalDesc = newTraitCand.additionalDescription ?? "";
                                            if (overrideDesc.length === 0 && additionalDesc.length === 0) return null;
                                            const oldTraitCand = operator.trait?.candidates?.[(operator.trait?.candidates?.length ?? 0) - 1];
                                            const newBb = newTraitCand.blackboard ?? [];
                                            let oldDesc: string | null;
                                            let oldBb: typeof descriptionBlackboard;
                                            let newDesc: string;
                                            let mergedNewBb: typeof newBb;
                                            if (overrideDesc.length > 0) {
                                                const oldFromTrait = oldTraitCand?.overrideDescription ?? null;
                                                oldDesc = oldFromTrait ?? operator.description ?? null;
                                                oldBb = oldFromTrait ? (oldTraitCand?.blackboard ?? []) : descriptionBlackboard;
                                                newDesc = overrideDesc;
                                                mergedNewBb = newBb;
                                            } else {
                                                const baseDesc = operator.description ?? "";
                                                oldDesc = baseDesc.length > 0 ? baseDesc : null;
                                                oldBb = descriptionBlackboard;
                                                newDesc = baseDesc.length > 0 ? `${baseDesc}\n${additionalDesc}` : additionalDesc;
                                                mergedNewBb = [...descriptionBlackboard, ...newBb];
                                            }
                                            const html = showDiff && oldDesc ? renderDescriptionDiffHtml(oldDesc, newDesc, oldBb, mergedNewBb) : descriptionToHtml(newDesc, mergedNewBb);
                                            return (
                                                <>
                                                    <div className="mb-1 flex items-center justify-between">
                                                        <h6 className="font-medium text-foreground text-xs">Trait Changes</h6>
                                                        <Tooltip>
                                                            <TooltipTrigger
                                                                render={(props) => (
                                                                    <span {...props}>
                                                                        <Switch checked={showDiff} onCheckedChange={setShowDiff} />
                                                                    </span>
                                                                )}
                                                            />
                                                            <TooltipPopup>Show diff vs. base</TooltipPopup>
                                                        </Tooltip>
                                                    </div>
                                                    <div className="rounded-md bg-secondary/20 p-2">
                                                        <span
                                                            className="text-muted-foreground text-xs"
                                                            // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized
                                                            dangerouslySetInnerHTML={{ __html: html }}
                                                        />
                                                    </div>
                                                </>
                                            );
                                        })()}
                                        {(() => {
                                            const phase = currentModule.data.phases[moduleLevel - 1];
                                            const talentParts = phase.parts.filter((p) => p.target === "TALENT" || p.target === "TALENT_DATA_ONLY");
                                            const talentCandidates = talentParts.flatMap((p) => p.addOrOverrideTalentDataBundle.candidates?.filter((c) => c.upgradeDescription || c.description) ?? []);
                                            if (talentCandidates.length === 0) return null;
                                            return (
                                                <div>
                                                    <h6 className="mb-1 font-medium text-foreground text-xs">Talent Changes</h6>
                                                    {talentCandidates.map((c, cIdx) => {
                                                        const oldTalent = operator.talents?.[c.talentIndex];
                                                        const oldCand = oldTalent?.candidates?.[oldTalent.candidates.length - 1];
                                                        const newDesc = c.upgradeDescription || c.description || "";
                                                        const html = showDiff && oldCand?.description ? renderDescriptionDiffHtml(oldCand.description, newDesc, oldCand.blackboard ?? [], c.blackboard ?? []) : descriptionToHtml(newDesc, c.blackboard ?? []);
                                                        return (
                                                            // biome-ignore lint/suspicious/noArrayIndexKey: candidate order is determined by module phase data
                                                            <div className="rounded-md bg-secondary/20 px-2 py-1" key={`tc-${cIdx}-${c.name ?? ""}`}>
                                                                {c.name && <span className="font-medium text-foreground text-xs">{c.name}: </span>}
                                                                <span
                                                                    className="text-muted-foreground text-xs"
                                                                    // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized
                                                                    dangerouslySetInnerHTML={{ __html: html }}
                                                                />
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            );
                                        })()}
                                    </div>
                                ) : null}
                            </div>
                        </CollapsibleContent>
                    </Collapsible>
                </div>
            )}

            <Collapsible open={showTalents} onOpenChange={setShowTalents}>
                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                    <span className="flex items-center gap-2">
                        <LibraryBig className="h-4 w-4 text-primary" />
                        <span className="font-medium text-sm">Talents</span>
                    </span>
                </CollapsibleTrigger>
                <CollapsibleContent>
                    {(() => {
                        type Eff = {
                            key: string;
                            name: string | null;
                            description: string;
                            blackboard: { key: string; value: number; valueStr?: string | null }[];
                            requiredPotentialRank: number;
                            baseDescription: string | null;
                            baseBlackboard: { key: string; value: number; valueStr?: string | null }[];
                            modifiedByModule: boolean;
                        };

                        const baseList: Eff[] = (operator.talents ?? []).map((t, idx) => {
                            const c = getActiveTalentCandidate(t, phaseIndex, level, potentialRank);
                            return {
                                key: `t-${idx}`,
                                name: c?.name ?? null,
                                description: c?.description ?? "",
                                blackboard: c?.blackboard ?? [],
                                requiredPotentialRank: c?.requiredPotentialRank ?? 0,
                                baseDescription: c?.description ?? null,
                                baseBlackboard: c?.blackboard ?? [],
                                modifiedByModule: false,
                            };
                        });

                        if (phaseIndex === 2 && currentModule && moduleLevel > 0 && currentModule.data?.phases) {
                            for (let i = 0; i < Math.min(moduleLevel, currentModule.data.phases.length); i++) {
                                const phase = currentModule.data.phases[i];
                                const talentParts = (phase?.parts ?? []).filter((p) => p.target === "TALENT" || p.target === "TALENT_DATA_ONLY");
                                for (const part of talentParts) {
                                    const cands = part.addOrOverrideTalentDataBundle?.candidates ?? [];
                                    let chosen: (typeof cands)[number] | null = null;
                                    for (const c of cands) {
                                        if (potentialRank >= c.requiredPotentialRank) chosen = c;
                                    }
                                    if (!chosen) continue;
                                    const newDesc = chosen.upgradeDescription || chosen.description || "";
                                    const tIdx = chosen.talentIndex;
                                    if (tIdx >= 0 && tIdx < baseList.length) {
                                        baseList[tIdx] = {
                                            ...baseList[tIdx],
                                            name: chosen.name ?? baseList[tIdx].name,
                                            description: newDesc,
                                            blackboard: chosen.blackboard ?? [],
                                            modifiedByModule: true,
                                        };
                                    } else {
                                        baseList.push({
                                            key: `t-mod-${tIdx}-${i}`,
                                            name: chosen.name,
                                            description: newDesc,
                                            blackboard: chosen.blackboard ?? [],
                                            requiredPotentialRank: chosen.requiredPotentialRank ?? 0,
                                            baseDescription: null,
                                            baseBlackboard: [],
                                            modifiedByModule: true,
                                        });
                                    }
                                }
                            }
                        }

                        const visible = baseList.filter((t) => t.name || t.description);
                        if (visible.length === 0) {
                            return <p className="mt-3 text-muted-foreground text-xs">No talents unlocked at this configuration.</p>;
                        }

                        return (
                            <div className="mt-3 space-y-3">
                                {visible.map((t) => {
                                    const html = t.modifiedByModule && showDiff && t.baseDescription ? renderDescriptionDiffHtml(t.baseDescription, t.description, t.baseBlackboard, t.blackboard) : descriptionToHtml(t.description, t.blackboard);
                                    return (
                                        <div className="rounded-lg border border-border/50 bg-card/30 p-3" key={t.key}>
                                            <div className="mb-1 flex items-center gap-2">
                                                <h4 className="font-medium text-foreground text-sm">{t.name ?? "Unnamed Talent"}</h4>
                                                {t.requiredPotentialRank > 0 && (
                                                    <Tooltip>
                                                        <TooltipTrigger render={(props) => <img alt={`Pot ${t.requiredPotentialRank}`} className="h-4 w-4" decoding="async" loading="lazy" src={potentialIcon(t.requiredPotentialRank)} {...props} />} />
                                                        <TooltipPopup>Requires Potential {t.requiredPotentialRank}</TooltipPopup>
                                                    </Tooltip>
                                                )}
                                                {t.modifiedByModule && (
                                                    <Badge variant="outline" className="text-[10px]">
                                                        Module
                                                    </Badge>
                                                )}
                                            </div>
                                            <span
                                                className="text-muted-foreground text-xs"
                                                // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized
                                                dangerouslySetInnerHTML={{ __html: html }}
                                            />
                                        </div>
                                    );
                                })}
                            </div>
                        );
                    })()}
                </CollapsibleContent>
            </Collapsible>
        </div>
    );
});
