import { useQuery } from "@tanstack/react-query";
import { type IPlanProgressLabels, PlanProgressRows } from "#/components/tools/planner/PlanProgressRows";
import { planProgress } from "#/components/tools/planner/planProgress";
import { MAX_SKILL_LEVEL, masteryOf } from "#/components/tools/planner/planTargets";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { useOperatorName } from "#/hooks/use-operator-name";
import { type IOperatorPlanResponse, publicPlansQueryOptions } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { messages } from "./PlansTab.messages";

interface IPlanCardProps {
    p: IOperatorPlanResponse;
    roster: IRosterEntry[];
}

function PlanCard({ p, roster }: IPlanCardProps) {
    const operatorName = useOperatorName();
    const t: TypedT<typeof messages> = useT("user");
    const op = p.operator;
    if (!op) return null;

    const rosterEntry = roster.find((re) => re.operator_id === p.operator_id);
    const labels: IPlanProgressLabels = {
        level: t("profile.plans.level"),
        skills: t("profile.plans.skills"),
        modules: t("profile.plans.modules"),
        eliteAlt: (elite) => t("profile.plans.eliteAlt", { elite }),
        levelValue: (level) => t("profile.plans.skillLevel", { level }),
        skillFallback: (n) => t("profile.plans.skillFallback", { n }),
        skillTarget: (value) => (value <= MAX_SKILL_LEVEL ? String(value) : `M${masteryOf(value)}`),
        moduleStage: (stage) => (stage === 0 ? "X" : String(stage)),
    };

    return (
        <div className="relative flex flex-col gap-4 rounded-xl border border-border bg-card p-4 transition-all hover:shadow-md">
            <div className="flex items-start gap-3">
                <span aria-hidden="true" className="relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-xl border border-border bg-muted/70">
                    <OperatorAvatar charId={op.id} name={op.name} className="block h-full w-full object-cover" server={op.server} />
                </span>
                <div className="min-w-0 flex-1">
                    <h3 className="truncate font-bold text-foreground text-sm leading-tight">{operatorName(op)}</h3>
                    <p className="mt-0.5 truncate text-muted-foreground text-xs leading-normal">
                        {rarityToNumber(op.rarity)}★ {formatSubProfession(op.subProfessionId)}
                    </p>
                </div>
            </div>

            <PlanProgressRows progress={planProgress(op, p, rosterEntry)} server={op.server} labels={labels} className="border-border/40 border-t pt-3" />
        </div>
    );
}

function PlanGrid({ plans, roster }: { plans: IOperatorPlanResponse[]; roster: IRosterEntry[] }) {
    return (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,320px),1fr))] gap-4">
            {plans.map((p) => (
                <PlanCard key={p.id} p={p} roster={roster} />
            ))}
        </div>
    );
}

interface IPlanSection {
    /** Null for the trailing section of plans in no group. */
    group: string | null;
    plans: IOperatorPlanResponse[];
}

/** Section order. Name only for now; a pinned-first rule goes here as the leading comparison. */
function compareGroupNames(a: string, b: string): number {
    return a.localeCompare(b);
}

/** One section per group, a plan repeated under each of its groups, the groupless last. Empty when no plan has a group. */
function planSections(plans: IOperatorPlanResponse[]): IPlanSection[] {
    const byGroup = new Map<string, IOperatorPlanResponse[]>();
    const ungrouped: IOperatorPlanResponse[] = [];
    for (const p of plans) {
        if (p.groups.length === 0) ungrouped.push(p);
        for (const g of p.groups) {
            const list = byGroup.get(g);
            if (list) list.push(p);
            else byGroup.set(g, [p]);
        }
    }
    if (byGroup.size === 0) return [];
    const sections: IPlanSection[] = [...byGroup.keys()].sort(compareGroupNames).map((group) => ({ group, plans: byGroup.get(group) ?? [] }));
    if (ungrouped.length > 0) sections.push({ group: null, plans: ungrouped });
    return sections;
}

interface IPlansTabProps {
    uid: string;
    roster: IRosterEntry[];
}

export function PlansTab({ uid, roster }: IPlansTabProps) {
    const t: TypedT<typeof messages> = useT("user");
    const { data: plans = [], isLoading } = useQuery(publicPlansQueryOptions(uid));
    const sections = planSections(plans);

    if (isLoading) {
        return (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,320px),1fr))] gap-4">
                {["sk-p-1", "sk-p-2", "sk-p-3"].map((key) => (
                    <div key={key} className="h-24 animate-pulse rounded-xl border border-border/40 bg-card p-4" />
                ))}
            </div>
        );
    }

    if (plans.length === 0) {
        return (
            <div className="flex flex-col items-center justify-center gap-3 rounded-3xl border border-border bg-card px-8 py-16 text-center">
                <span className="font-mono text-[11px] text-muted-foreground uppercase tracking-widest">{t("profile.plans.kicker")}</span>
                <h3 className="font-semibold text-lg tracking-tight">{t("profile.plans.empty.title")}</h3>
                <p className="max-w-sm text-muted-foreground text-sm">{t("profile.plans.empty.desc")}</p>
            </div>
        );
    }

    return (
        <section aria-label={t("profile.plans.aria")} className="flex flex-col gap-6">
            {sections.length === 0 ? (
                <PlanGrid plans={plans} roster={roster} />
            ) : (
                sections.map((section) => {
                    const title = section.group ?? t("profile.plans.ungrouped");
                    return (
                        <section key={section.group ?? "\u0000ungrouped"} aria-label={title} className="flex flex-col gap-3">
                            <h2 className="flex items-baseline gap-2 font-mono text-[11px] text-muted-foreground uppercase tracking-widest">
                                <span>{title}</span>
                                <span className="tabular-nums">{section.plans.length}</span>
                            </h2>
                            <PlanGrid plans={section.plans} roster={roster} />
                        </section>
                    );
                })
            )}
        </section>
    );
}
