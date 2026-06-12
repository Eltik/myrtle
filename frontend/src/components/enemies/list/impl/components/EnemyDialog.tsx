import type React from "react";
import { useMemo, useState } from "react";
import { Dialog, DialogPopup } from "#/components/ui/dialog";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { env } from "#/env";
import type { IEnemyAttributes, IEnemyLevelStats, IEnemySkill } from "#/lib/api/enemies";
import { APPLY_WAY_DISPLAY } from "../constants";
import { DAMAGE_TOKENS, LEVEL_TOKENS } from "../tokens";
import type { IEnemyView } from "../types";
import { LevelBadge } from "./atoms";
import { EnemyChibiTab } from "./EnemyChibi";
import { EnemyPlaceholder } from "./EnemyPlaceholder";

interface IEnemyDialogProps {
    enemy: IEnemyView | null;
    onClose: () => void;
}

export function EnemyDialog({ enemy, onClose }: IEnemyDialogProps) {
    const open = enemy !== null;
    return (
        <Dialog open={open} onOpenChange={(o) => (!o ? onClose() : undefined)}>
            {enemy ? (
                <DialogPopup className="flex h-[min(700px,90vh)] w-full max-w-190 flex-col overflow-hidden rounded-[14px] border border-border bg-card p-0">
                    <DialogHeader enemy={enemy} />
                    <DialogBody enemy={enemy} />
                </DialogPopup>
            ) : null}
        </Dialog>
    );
}

function DialogHeader({ enemy }: { enemy: IEnemyView }) {
    const [imgError, setImgError] = useState(false);
    const tok = LEVEL_TOKENS[enemy.enemyLevel];
    const hasPortrait = !!enemy.portrait && !imgError;
    const portraitSrc = hasPortrait ? `${env.VITE_BACKEND_URL ?? ""}/api/assets${enemy.portrait}` : undefined;
    const meta = [enemy.enemyIndex, enemy.race, enemy.applyWay ? APPLY_WAY_DISPLAY[enemy.applyWay] : null].filter(Boolean).join(" · ");

    return (
        <div className="flex shrink-0 gap-3 border-border border-b px-4 py-3.5 sm:gap-4.5 sm:px-5 sm:py-4.5">
            <div
                className="relative h-16 w-16 shrink-0 overflow-hidden rounded-[10px] border bg-[color-mix(in_oklch,var(--muted)_40%,transparent)] sm:h-24 sm:w-24"
                style={{
                    borderColor: enemy.enemyLevel === "NORMAL" ? "var(--border)" : tok.badgeBorder,
                }}
            >
                {hasPortrait && portraitSrc ? <img src={portraitSrc} alt={enemy.name} onError={() => setImgError(true)} className="absolute inset-0 h-full w-full object-contain" /> : <EnemyPlaceholder className="absolute inset-0 h-full w-full p-2" />}
                {enemy.enemyLevel !== "NORMAL" && <span aria-hidden="true" className="absolute right-0 bottom-0 left-0 h-0.5 sm:h-0.75" style={{ background: tok.accent }} />}
            </div>

            <div className="flex min-w-0 flex-1 flex-col justify-center gap-1 pr-9 sm:gap-1.5 sm:pr-8">
                <div className="flex items-center gap-2">
                    <h2 className="m-0 truncate font-bold font-sans text-[17px] text-foreground leading-[1.15] tracking-tight sm:text-[22px] sm:leading-[1.1]">{enemy.name}</h2>
                    <LevelBadge level={enemy.enemyLevel} />
                </div>
                {meta && <div className="truncate font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em] sm:text-[11px] sm:tracking-[0.14em]">{meta}</div>}

                {enemy.damageType.length > 0 && (
                    <div className="mt-0.5 flex flex-wrap gap-1.5 sm:mt-1">
                        {enemy.damageType.map((t) => {
                            const tk = DAMAGE_TOKENS[t];
                            return (
                                <span
                                    key={t}
                                    className="inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 font-medium font-sans text-[10.5px] text-foreground leading-none sm:px-2.25 sm:py-0.75 sm:text-[11px]"
                                    style={{
                                        borderColor: `color-mix(in oklch, ${tk.color} 40%, transparent)`,
                                        background: `color-mix(in oklch, ${tk.color} 12%, transparent)`,
                                    }}
                                >
                                    <span className="h-1.5 w-1.5 rounded-[1.5px]" style={{ background: tk.color }} />
                                    {tk.label}
                                </span>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
}

function DialogBody({ enemy }: { enemy: IEnemyView }) {
    const phases = enemy.stats?.levels ?? [];
    return (
        <Tabs defaultValue="overview" className="flex min-h-0 flex-1 flex-col gap-0">
            <div className="shrink-0 border-border border-b px-4 pt-3 sm:px-5">
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
                    <TabsTab value="chibi" className="h-8 text-[13px]">
                        Chibi
                    </TabsTab>
                </TabsList>
            </div>
            <div className="min-h-0 flex-1 overflow-y-auto">
                <TabsPanel value="overview">
                    <OverviewTab enemy={enemy} />
                </TabsPanel>
                <TabsPanel value="stats">
                    <StatsTab enemy={enemy} phases={phases} />
                </TabsPanel>
                <TabsPanel value="skills">
                    <SkillsTab enemy={enemy} phases={phases} />
                </TabsPanel>
                <TabsPanel value="chibi">
                    <EnemyChibiTab key={enemy.enemyId} enemyId={enemy.enemyId} />
                </TabsPanel>
            </div>
        </Tabs>
    );
}

function Kicker({ children }: { children: React.ReactNode }) {
    return <span className="font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.14em]">{children}</span>;
}

function SectionHead({ children }: { children: React.ReactNode }) {
    return (
        <div className="mb-3 flex items-center gap-2.5">
            <Kicker>{children}</Kicker>
            <span className="h-px flex-1 bg-border" />
        </div>
    );
}

function descToHtml(text: string): string {
    return text.replace(/<@ba\.[a-z.]+>(.*?)<\/>/g, (_m, inner: string) => `<strong style="color: var(--foreground)">${inner}</strong>`);
}

interface IAbilityGroup {
    title: string | null;
    items: { text: string; textFormat: string }[];
}

function OverviewTab({ enemy }: { enemy: IEnemyView }) {
    const groups = useMemo<IAbilityGroup[]>(() => {
        const out: IAbilityGroup[] = [];
        let cur: IAbilityGroup = { title: null, items: [] };
        for (const ab of enemy.abilityList ?? []) {
            if (ab.textFormat === "TITLE") {
                if (cur.items.length || cur.title) out.push(cur);
                cur = { title: ab.text, items: [] };
            } else {
                cur.items.push(ab);
            }
        }
        if (cur.items.length || cur.title) out.push(cur);
        return out;
    }, [enemy.abilityList]);

    return (
        <div className="flex flex-col gap-5 p-4 sm:gap-5.5 sm:p-5">
            {enemy.description && (
                <section>
                    <SectionHead>Description</SectionHead>
                    <p className="m-0 text-pretty font-sans text-[13.5px] text-foreground leading-relaxed">{enemy.description}</p>
                </section>
            )}

            {groups.length > 0 && (
                <section>
                    <SectionHead>Traits</SectionHead>
                    <div className="flex flex-col gap-3.5">
                        {groups.map((g, gi) => (
                            <div key={g.title ?? `g-${gi}`}>
                                {g.title && <h4 className="mb-1.5 font-sans font-semibold text-[12.5px] text-primary uppercase leading-none tracking-[0.04em]">{g.title}</h4>}
                                {g.items.length > 0 && (
                                    <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
                                        {g.items.map((ab) => (
                                            <li key={`${ab.text}-${ab.textFormat}`} className="flex gap-2.5 font-sans text-[13.5px] leading-[1.55]">
                                                <span className="mt-2 h-1.25 w-1.25 shrink-0 rounded-full bg-[color-mix(in_oklch,var(--muted-foreground)_50%,transparent)]" />
                                                {/* biome-ignore lint/security/noDangerouslySetInnerHtml: text is from trusted backend handbook data and is sanitized via descToHtml */}
                                                <span className="text-foreground" dangerouslySetInnerHTML={{ __html: descToHtml(ab.text) }} />
                                            </li>
                                        ))}
                                    </ul>
                                )}
                            </div>
                        ))}
                    </div>
                </section>
            )}

            {enemy.enemyTags && enemy.enemyTags.length > 0 && (
                <section>
                    <SectionHead>Tags</SectionHead>
                    <div className="flex flex-wrap gap-1.5">
                        {enemy.enemyTags.map((tag) => (
                            <span key={tag} className="rounded-full border border-border px-2.25 py-0.75 font-medium font-sans text-[11px] text-muted-foreground leading-none">
                                {tag}
                            </span>
                        ))}
                    </div>
                </section>
            )}

            <section>
                <SectionHead>Metadata</SectionHead>
                <div className="grid gap-2" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(140px, 1fr))" }}>
                    <Meta label="Enemy ID" value={enemy.enemyId} />
                    <Meta label="Sort ID" value={String(enemy.sortId)} />
                    <Meta label="Race" value={enemy.race ?? "-"} />
                    <Meta label="In Handbook" value={enemy.hideInHandbook ? "Hidden" : "Visible"} />
                </div>
            </section>
        </div>
    );
}

function Meta({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-md bg-[color-mix(in_oklch,var(--muted)_50%,transparent)] px-2.5 py-2">
            <div className="mb-1.25 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{label}</div>
            <div className="break-all font-medium font-sans text-[12.5px] text-foreground leading-[1.2]">{value}</div>
        </div>
    );
}

function PhaseHeader({ phase, label }: { phase: IEnemyLevelStats; label: string }) {
    return (
        <div className="flex items-center gap-2 rounded-md border border-[color-mix(in_oklch,var(--primary)_45%,transparent)] bg-[color-mix(in_oklch,var(--primary)_14%,transparent)] px-2.5 py-1.5 font-medium font-mono text-[10.5px] text-primary uppercase leading-none tracking-[0.14em]">
            <span aria-hidden="true" className="inline-block h-1.25 w-1.25 rounded-full bg-primary" />
            <span className="truncate">{label}</span>
            <span className="ml-auto font-mono text-[9.5px] text-muted-foreground tracking-[0.12em]">L{phase.level}</span>
        </div>
    );
}

interface IPhaseMeta {
    labels: string[];
    fromNarrative: boolean;
    narrativeFormCount: number;
}

interface IPhaseMetaResult extends IPhaseMeta {
    orderedPhases: IEnemyLevelStats[];
}

function getPhaseMeta(enemy: IEnemyView, phases: IEnemyLevelStats[]): IPhaseMetaResult {
    const titles = (enemy.abilityList ?? [])
        .filter((a) => a.textFormat === "TITLE")
        .map((a) => a.text.trim())
        .filter(Boolean);
    if (titles.length === phases.length && titles.length > 1) {
        return {
            labels: titles,
            fromNarrative: true,
            narrativeFormCount: titles.length,
            orderedPhases: phases.slice().reverse(),
        };
    }
    if (phases.length === 1) {
        return {
            labels: ["Stats"],
            fromNarrative: false,
            narrativeFormCount: titles.length,
            orderedPhases: phases,
        };
    }
    return {
        labels: phases.map((p, i) => `Level ${p.level ?? i}`),
        fromNarrative: false,
        narrativeFormCount: titles.length,
        orderedPhases: phases,
    };
}

interface IStatRow {
    label: string;
    pick: (a: IEnemyAttributes) => number;
    format: (v: number) => string;
}

const STAT_ROWS: IStatRow[] = [
    { label: "Max HP", pick: (a) => a.maxHp, format: (v) => v.toLocaleString() },
    { label: "ATK", pick: (a) => a.atk, format: (v) => v.toLocaleString() },
    { label: "DEF", pick: (a) => a.def, format: (v) => v.toLocaleString() },
    { label: "RES %", pick: (a) => a.magicResistance, format: (v) => String(v) },
    { label: "Move Speed", pick: (a) => a.moveSpeed, format: (v) => v.toFixed(2) },
    { label: "ASPD", pick: (a) => a.attackSpeed, format: (v) => v.toFixed(2) },
    { label: "Base ATK Time", pick: (a) => a.baseAttackTime, format: (v) => v.toFixed(2) },
    { label: "Weight", pick: (a) => a.massLevel, format: (v) => String(v) },
];

const IMMUNITY_ROWS: { label: string; pick: (a: IEnemyAttributes) => boolean }[] = [
    { label: "Stun", pick: (a) => !!a.stunImmune },
    { label: "Silence", pick: (a) => !!a.silenceImmune },
    { label: "Sleep", pick: (a) => !!a.sleepImmune },
    { label: "Frozen", pick: (a) => !!a.frozenImmune },
    { label: "Levitate", pick: (a) => !!a.levitateImmune },
];

function PhaseStatsBlock({ phase, label }: { phase: IEnemyLevelStats; label: string }) {
    return (
        <div className="flex flex-col gap-2">
            <PhaseHeader phase={phase} label={label} />
            <div className="rounded-[10px] border border-border bg-[color-mix(in_oklch,var(--muted)_22%,transparent)] p-3">
                <dl className="m-0 grid grid-cols-1 gap-1 sm:grid-cols-2">
                    {STAT_ROWS.map((row) => (
                        <div key={row.label} className="flex items-baseline justify-between gap-2 border-border/40 border-b py-1 last:border-b-0 sm:nth-last-[2]:border-b-0">
                            <dt className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{row.label}</dt>
                            <dd className="m-0 font-mono font-semibold text-[13px] text-foreground tabular-nums leading-none">{row.format(row.pick(phase.attributes))}</dd>
                        </div>
                    ))}
                </dl>
            </div>
        </div>
    );
}

function ImmunityIcon({ on }: { on: boolean }) {
    if (on) {
        return (
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" aria-hidden="true">
                <polyline points="20 6 9 17 4 12" />
            </svg>
        );
    }
    return (
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" aria-hidden="true">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
    );
}

function ImmunityCompareGrid({ phases, labels }: { phases: IEnemyLevelStats[]; labels: string[] }) {
    return (
        <div className="overflow-x-auto rounded-[10px] border border-border">
            <table className="w-full border-collapse text-left">
                <thead>
                    <tr className="border-border/60 border-b bg-[color-mix(in_oklch,var(--muted)_30%,transparent)]">
                        <th className="px-2.5 py-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em] sm:px-3 sm:py-2">Status</th>
                        {phases.map((p, i) => (
                            <th key={`im-h-${p.level}-${p.attributes.maxHp}-${p.attributes.atk}`} className="px-2.5 py-1.5 text-center font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em] sm:px-3 sm:py-2">
                                {phases.length > 1 ? labels[i] : "Value"}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {IMMUNITY_ROWS.map((row) => (
                        <tr key={row.label} className="border-border/40 border-b last:border-b-0">
                            <td className="px-2.5 py-1.5 font-medium font-sans text-[12px] text-foreground leading-none sm:px-3 sm:py-2">{row.label}</td>
                            {phases.map((p, i) => {
                                const on = row.pick(p.attributes);
                                return (
                                    <td key={`im-${row.label}-${p.level}-${p.attributes.maxHp}-${p.attributes.atk}`} className="px-2.5 py-1.5 text-center sm:px-3 sm:py-2">
                                        <span
                                            role="img"
                                            aria-label={`${row.label} ${on ? "immune" : "vulnerable"} in ${labels[i]}`}
                                            className={
                                                on
                                                    ? "inline-flex h-5 w-5 items-center justify-center rounded-full bg-[color-mix(in_oklch,var(--primary)_14%,transparent)] text-primary shadow-[inset_0_0_0_1px_color-mix(in_oklch,var(--primary)_45%,transparent)]"
                                                    : "inline-flex h-5 w-5 items-center justify-center rounded-full bg-[color-mix(in_oklch,var(--muted)_40%,transparent)] text-muted-foreground"
                                            }
                                        >
                                            <ImmunityIcon on={on} />
                                        </span>
                                    </td>
                                );
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

function NarrativeFormsNote({ narrativeFormCount, levelCount }: { narrativeFormCount: number; levelCount: number }) {
    return (
        <div className="mt-3 flex gap-2.5 rounded-md border border-border bg-[color-mix(in_oklch,var(--muted)_30%,transparent)] px-3 py-2.5">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" className="mt-px shrink-0 text-muted-foreground">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 16v-4M12 8h.01" />
            </svg>
            <p className="m-0 font-sans text-[12px] text-muted-foreground leading-relaxed">
                This enemy transforms across <strong className="text-foreground">{narrativeFormCount}</strong> forms in-fight, but Hypergryph's data ships {levelCount === 1 ? "a single stat block" : `${levelCount} stat blocks`}. Subsequent forms modify behavior through skill triggers rather than swapping the base
                HP/ATK/DEF - see <strong className="text-foreground">Overview · Traits</strong> for the form descriptions.
            </p>
        </div>
    );
}

function StatsTab({ enemy, phases }: { enemy: IEnemyView; phases: IEnemyLevelStats[] }) {
    if (phases.length === 0) {
        return (
            <div className="p-10 text-center">
                <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">No stats available for this enemy.</p>
            </div>
        );
    }
    const meta = getPhaseMeta(enemy, phases);
    const multi = phases.length > 1;
    const sectionLabel = multi ? (meta.fromNarrative ? `Combat · ${phases.length} forms` : `Combat · ${phases.length} levels`) : "Combat";
    const showNarrativeNote = meta.narrativeFormCount > phases.length;
    const gridClass = multi ? "grid grid-cols-1 gap-3 sm:grid-cols-2" : "grid grid-cols-1 gap-3";
    return (
        <div className="flex flex-col gap-5 p-4 sm:gap-5.5 sm:p-5">
            <section>
                <SectionHead>{sectionLabel}</SectionHead>
                <div className={gridClass}>
                    {meta.orderedPhases.map((phase, i) => (
                        <PhaseStatsBlock key={`phase-${phase.level}-${phase.attributes.maxHp}-${phase.attributes.atk}-${phase.attributes.def}`} phase={phase} label={meta.labels[i]} />
                    ))}
                </div>
                {showNarrativeNote && <NarrativeFormsNote narrativeFormCount={meta.narrativeFormCount} levelCount={phases.length} />}
            </section>
            <section>
                <SectionHead>Immunities</SectionHead>
                <ImmunityCompareGrid phases={meta.orderedPhases} labels={meta.labels} />
            </section>
        </div>
    );
}

function skillsAreEqual(a: IEnemySkill[], b: IEnemySkill[]): boolean {
    if (a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        const x = a[i];
        const y = b[i];
        if (x.prefabKey !== y.prefabKey || x.priority !== y.priority || x.cooldown !== y.cooldown || x.initCooldown !== y.initCooldown || x.spCost !== y.spCost) return false;
    }
    return true;
}

function SkillCard({ skill }: { skill: IEnemySkill }) {
    return (
        <div className="rounded-[10px] border border-border bg-[color-mix(in_oklch,var(--muted)_30%,transparent)] p-3 sm:p-3.5">
            <div className="flex items-center justify-between gap-2">
                <span className="truncate font-medium font-mono text-[12px] text-foreground leading-none">{skill.prefabKey}</span>
                <span className="font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]">P{skill.priority}</span>
            </div>
            <div className="mt-2 grid grid-cols-3 gap-1.5 sm:gap-2">
                <Meta label="Cooldown" value={`${skill.cooldown}s`} />
                <Meta label="Init CD" value={`${skill.initCooldown}s`} />
                <Meta label="SP Cost" value={String(skill.spCost)} />
            </div>
        </div>
    );
}

function SkillsTab({ enemy, phases }: { enemy: IEnemyView; phases: IEnemyLevelStats[] }) {
    if (phases.length === 0 || phases.every((p) => p.skills.length === 0)) {
        return (
            <div className="p-10 text-center">
                <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">No skill data available for this enemy.</p>
            </div>
        );
    }

    const base = phases[0].skills;
    const allSame = phases.every((p) => skillsAreEqual(p.skills, base));

    const meta = getPhaseMeta(enemy, phases);

    if (allSame) {
        return (
            <div className="flex flex-col gap-3 p-4 sm:gap-4 sm:p-5">
                {phases.length > 1 && (
                    <p className="m-0 font-sans text-[12px] text-muted-foreground leading-normal">
                        Skill kit is identical across all <strong className="text-foreground">{phases.length}</strong> {meta.fromNarrative ? "forms" : "levels"}.
                    </p>
                )}
                {base.map((sk) => (
                    <SkillCard key={`${sk.prefabKey}-${sk.priority}-${sk.cooldown}-${sk.initCooldown}-${sk.spCost}`} skill={sk} />
                ))}
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4 p-4 sm:gap-5 sm:p-5">
            {meta.orderedPhases.map((phase, i) => (
                <section key={`skills-phase-${phase.level}-${phase.attributes.maxHp}-${phase.attributes.atk}`} className="flex flex-col gap-2.5 sm:gap-3">
                    <PhaseHeader phase={phase} label={meta.labels[i]} />
                    {phase.skills.length === 0 ? (
                        <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">No skills active in this {meta.fromNarrative ? "form" : "level"}.</p>
                    ) : (
                        <div className="flex flex-col gap-3">
                            {phase.skills.map((sk) => (
                                <SkillCard key={`p${phase.level}-${sk.prefabKey}-${sk.priority}-${sk.cooldown}-${sk.initCooldown}-${sk.spCost}`} skill={sk} />
                            ))}
                        </div>
                    )}
                </section>
            ))}
        </div>
    );
}
