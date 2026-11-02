import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useCnChangeSummary } from "#/components/operators/list/impl/components/CnUpdateBadge";
import { Badge } from "#/components/ui/badge";
import { cnChangesQueryOptions, operatorQueryOptions } from "#/lib/api/operators";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type IValueDiff, moduleStageDiffs, phaseStatDiffs, skillLevelDiffs, talentDiffs, traitDiffs, trustDiffs } from "#/lib/operators/cn-changes";
import type { OperatorChange } from "#/types/generated/OperatorChange";
import type { IModulePhase, IOperatorListItem, IOperatorModule } from "#/types/operators";
import { asset } from "../assets";
import { descriptionToHtml } from "../description";
import { formatAttributeKey, formatSkillLevel, formatStatValue } from "../helpers";
import type { messages as helperMessages } from "../helpers.messages";
import type { messages } from "./CnUpdatesSection.messages";

type SectionT = TypedT<typeof messages & typeof helperMessages>;

const EMPTY: OperatorChange[] = [];

/** `respawnTime` -> `respawn_time`, the spelling the attribute labels are keyed by. */
function snakeCase(key: string): string {
    return key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);
}

function formatNumber(value: number | null, t: SectionT): string {
    return value === null ? t("cnUpdates.section.absent") : String(Number.parseFloat(value.toPrecision(6)));
}

interface ICnUpdatesSectionProps {
    operator: IOperatorListItem;
    /** The game-data server the page reads, which the changes are diffed against. */
    server: string;
}

/**
 * What CN's data has for this operator that `server`'s does not, rendered from CN's own record.
 * Mounted only while the "Show CN updates" setting is on, so with it off the page makes neither
 * request; and it renders nothing for an operator the diff did not flag.
 */
export function CnUpdatesSection({ operator, server }: ICnUpdatesSectionProps) {
    const t: SectionT = useT("operators");
    const id = operator.id ?? "";
    const { data: all } = useQuery(cnChangesQueryOptions(server));
    const changes = all?.find((row) => row.id === id)?.changes ?? EMPTY;
    const { data: cn, isLoading, isError } = useQuery({ ...operatorQueryOptions(id, "cn"), enabled: changes.length > 0 });
    const summary = useCnChangeSummary(changes);
    if (changes.length === 0) return null;

    return (
        <section aria-labelledby="cn-updates-title" className="mb-6 rounded-xl border border-primary/30 bg-card/60 p-4 backdrop-blur-md sm:p-5">
            <div className="mb-2 flex items-center gap-2">
                <Badge variant="outline" className="border-primary/40 font-mono text-primary">
                    {t("cnUpdates.section.badge")}
                </Badge>
                <h2 id="cn-updates-title" className="font-semibold text-foreground">
                    {t("cnUpdates.section.title")}
                </h2>
            </div>
            <p className="mb-4 text-muted-foreground text-sm">{t("cnUpdates.section.desc", { changes: summary })}</p>
            {isLoading ? (
                <p className="text-muted-foreground text-sm">{t("cnUpdates.section.loading")}</p>
            ) : isError || !cn ? (
                <p className="text-muted-foreground text-sm">{t("cnUpdates.section.unavailable")}</p>
            ) : (
                <div className="space-y-5">
                    {changes.map((change) => (
                        <ChangeBlock key={changeKey(change)} change={change} en={operator} cn={cn} />
                    ))}
                </div>
            )}
        </section>
    );
}

function changeKey(change: OperatorChange): string {
    if ("moduleId" in change) return `${change.kind}:${change.moduleId}`;
    if ("skillId" in change) return `${change.kind}:${change.skillId}`;
    return change.kind;
}

function ChangeBlock({ change, en, cn }: { change: OperatorChange; en: IOperatorListItem; cn: IOperatorListItem }) {
    const t: SectionT = useT("operators");
    switch (change.kind) {
        case "module_added": {
            const module = cn.modules.find((m) => m.uniEquipId === change.moduleId);
            return module ? <NewModule module={module} /> : null;
        }
        case "module_changed": {
            const stages = moduleStageDiffs(
                en.modules.find((m) => m.uniEquipId === change.moduleId),
                cn.modules.find((m) => m.uniEquipId === change.moduleId),
            );
            return (
                <Block title={`${t("cnUpdates.section.changedModule")}: ${change.moduleId}`}>
                    {stages.map((s) => (
                        <DiffTable key={s.stage} caption={t("cnUpdates.section.moduleStage", { stage: s.stage })} rows={s.values} label={(k) => formatAttributeKey(k, t)} />
                    ))}
                </Block>
            );
        }
        case "skill_added": {
            const skill = cn.skills.find((s) => s.skillId === change.skillId);
            const last = skill?.static?.levels?.[skill.static.levels.length - 1];
            return <Block title={`${t("cnUpdates.section.newSkill")}: ${last?.name ?? change.skillId}`}>{last && <CnText text={last.description} blackboard={last.blackboard} />}</Block>;
        }
        case "skill_values_changed": {
            const enSkill = en.skills.find((s) => s.skillId === change.skillId);
            const levels = skillLevelDiffs(
                enSkill,
                cn.skills.find((s) => s.skillId === change.skillId),
            );
            const name = enSkill?.static?.levels?.[0]?.name ?? change.skillId;
            return (
                <Block title={`${t("cnUpdates.section.skillValues")}: ${name}`}>
                    {levels.map((l) => (
                        <DiffTable key={l.level} caption={formatSkillLevel(l.level - 1, t)} rows={l.values} />
                    ))}
                </Block>
            );
        }
        case "stats_changed":
            return (
                <Block title={t("cnUpdates.section.stats")}>
                    {phaseStatDiffs(en, cn).map((p) => (
                        <DiffTable key={p.phase} caption={t("cnUpdates.section.phase", { phase: p.phase, level: p.level })} rows={p.values} label={(k) => formatAttributeKey(snakeCase(k), t)} />
                    ))}
                </Block>
            );
        case "trust_bonus_changed":
            return (
                <Block title={t("cnUpdates.section.trust")}>
                    <DiffTable rows={trustDiffs(en, cn)} label={(k) => formatAttributeKey(snakeCase(k), t)} />
                </Block>
            );
        case "talents_changed":
            return (
                <Block title={t("cnUpdates.section.talents")}>
                    {talentDiffs(en, cn).map((d) => (
                        <DiffTable key={`${d.talent}-${d.candidate}`} caption={en.talents?.[d.talent]?.candidates?.[d.candidate]?.name ?? undefined} rows={d.values} />
                    ))}
                </Block>
            );
        case "trait_changed":
            return (
                <Block title={t("cnUpdates.section.trait")}>
                    {traitDiffs(en, cn).map((rows, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: trait candidates have no id, their order is the data's
                        <DiffTable key={i} rows={rows} />
                    ))}
                </Block>
            );
        case "range_changed":
            return <p className="text-muted-foreground text-sm">{t("cnUpdates.section.range")}</p>;
        case "potentials_changed":
            return <p className="text-muted-foreground text-sm">{t("cnUpdates.section.potentials")}</p>;
    }
}

function Block({ title, children }: { title: string; children: ReactNode }) {
    return (
        <div>
            <h3 className="mb-2 font-medium text-foreground text-sm">{title}</h3>
            <div className="space-y-3">{children}</div>
        </div>
    );
}

/** CN's own text with its `{key}` placeholders filled, marked as Chinese for screen readers and fonts. */
function CnText({ text, blackboard }: { text: string | null | undefined; blackboard: { key: string; value: number }[] | null | undefined }) {
    if (!text) return null;
    return (
        <span
            lang="zh-CN"
            className="block text-muted-foreground text-xs leading-relaxed"
            // biome-ignore lint/security/noDangerouslySetInnerHtml: Sanitized in description-to-html
            dangerouslySetInnerHTML={{ __html: descriptionToHtml(text, blackboard ?? []) }}
        />
    );
}

function DiffTable({ caption, rows, label = (k) => k }: { caption?: string; rows: IValueDiff[]; label?: (key: string) => string }) {
    const t: SectionT = useT("operators");
    if (rows.length === 0) return null;
    return (
        <table className="w-full max-w-md text-sm">
            {caption && <caption className="mb-1 text-left text-muted-foreground text-xs">{caption}</caption>}
            <thead>
                <tr className="text-[11px] text-muted-foreground uppercase tracking-wider">
                    <th className="py-1 text-left font-medium">{t("cnUpdates.section.column.value")}</th>
                    <th className="py-1 text-right font-medium">{t("cnUpdates.section.column.here")}</th>
                    <th className="py-1 text-right font-medium">{t("cnUpdates.section.column.cn")}</th>
                </tr>
            </thead>
            <tbody>
                {rows.map((r) => (
                    <tr key={r.key} className="border-border/40 border-t">
                        <td className="py-1 text-muted-foreground">{label(r.key)}</td>
                        <td className="py-1 text-right text-muted-foreground tabular-nums">{formatNumber(r.en, t)}</td>
                        <td className="py-1 text-right font-semibold text-foreground tabular-nums">{formatNumber(r.cn, t)}</td>
                    </tr>
                ))}
            </tbody>
        </table>
    );
}

type StageText = { text: string; blackboard: { key: string; value: number }[] };

/**
 * The trait and talent texts one module stage carries, with the blackboards that fill them. Each
 * part's first candidate only, the one with no potential requirement, as the Info tab reads it.
 */
function stageTexts(phase: IModulePhase): { trait: StageText[]; talent: StageText[] } {
    const trait = phase.parts.flatMap((p) => {
        const c = p.overrideTraitDataBundle.candidates?.[0];
        return c ? [{ text: c.overrideDescription || c.additionalDescription || "", blackboard: c.blackboard ?? [] }] : [];
    });
    const talent = phase.parts.flatMap((p) => {
        const c = p.addOrOverrideTalentDataBundle.candidates?.[0];
        return c ? [{ text: c.upgradeDescription || c.description || "", blackboard: c.blackboard ?? [] }] : [];
    });
    return { trait: trait.filter((x) => x.text), talent: talent.filter((x) => x.text) };
}

function NewModule({ module }: { module: IOperatorModule }) {
    const t: SectionT = useT("operators");
    const designator = module.typeName1 && module.typeName2 ? `${module.typeName1}-${module.typeName2}` : module.typeName1;
    const phases = module.data?.phases ?? [];
    return (
        <div>
            <h3 className="mb-2 font-medium text-foreground text-sm">{t("cnUpdates.section.newModule")}</h3>
            <div className="rounded-lg border border-border/50 bg-card/30 p-4">
                <div className="mb-4 flex items-center gap-3">
                    {module.image && <img alt={module.uniEquipName} className="h-14 w-14 rounded-md object-contain" decoding="async" loading="lazy" src={asset(module.image, "cn")} />}
                    <div>
                        <h4 lang="zh-CN" className="font-semibold text-foreground">
                            {module.uniEquipName}
                        </h4>
                        {designator && (
                            <Badge variant="outline" className="mt-1">
                                {designator}
                            </Badge>
                        )}
                    </div>
                </div>
                <div className="grid gap-3 md:grid-cols-3">
                    {phases.map((phase) => {
                        const texts = stageTexts(phase);
                        return (
                            <div key={phase.equipLevel} className="rounded-md bg-secondary/20 p-3">
                                <h5 className="mb-2 font-medium text-foreground text-xs">{t("cnUpdates.section.moduleStage", { stage: phase.equipLevel })}</h5>
                                {phase.attributeBlackboard.length > 0 ? (
                                    <ul className="mb-2 flex flex-wrap gap-x-3 gap-y-1">
                                        {phase.attributeBlackboard.map((attr) => (
                                            <li key={attr.key} className="text-xs">
                                                <span className="text-muted-foreground">{formatAttributeKey(attr.key, t)} </span>
                                                <span className="font-medium text-foreground tabular-nums">{formatStatValue(attr.value)}</span>
                                            </li>
                                        ))}
                                    </ul>
                                ) : (
                                    <p className="mb-2 text-muted-foreground text-xs">{t("cnUpdates.section.noBonuses")}</p>
                                )}
                                {texts.trait.map((x, i) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: a stage's candidates have no id, their order is the data's
                                    <div key={`trait-${i}`} className="mb-2">
                                        <div className="mb-0.5 font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("cnUpdates.section.moduleTrait")}</div>
                                        <CnText text={x.text} blackboard={x.blackboard} />
                                    </div>
                                ))}
                                {texts.talent.map((x, i) => (
                                    // biome-ignore lint/suspicious/noArrayIndexKey: a stage's candidates have no id, their order is the data's
                                    <div key={`talent-${i}`} className="mb-2">
                                        <div className="mb-0.5 font-medium text-[10px] text-muted-foreground uppercase tracking-wider">{t("cnUpdates.section.moduleTalent")}</div>
                                        <CnText text={x.text} blackboard={x.blackboard} />
                                    </div>
                                ))}
                            </div>
                        );
                    })}
                </div>
            </div>
        </div>
    );
}
