import { OperatorPanel } from "frontend";
import type { ReactNode } from "react";

const OPERATOR_ACCENT = "oklch(0.74 0.17 75)";

/** The expanded Operator subscore card the panel always lives inside. */
const PanelFrame = ({ pct, children }: { pct: string; children: ReactNode }) => (
    <div className="mx-auto max-w-2xl overflow-hidden rounded-xl border border-border bg-card sm:rounded-2xl">
        <div className="flex items-center justify-between p-4 sm:p-5">
            <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Operator · Roster depth &amp; investment</span>
            <span className="font-bold text-2xl tabular-nums" style={{ color: OPERATOR_ACCENT }}>
                {pct}
            </span>
        </div>
        <div className="border-border/30 border-t">{children}</div>
    </div>
);

const scoreBreakdown = [
    { kind: "elite", weight_share: 0.24, completion: 0.91, contribution: 0.2184 },
    { kind: "level", weight_share: 0.17, completion: 0.88, contribution: 0.1496 },
    { kind: "mastery", weight_share: 0.28, completion: 0.62, contribution: 0.1736 },
    { kind: "module", weight_share: 0.19, completion: 0.55, contribution: 0.1045 },
    { kind: "potential", weight_share: 0.08, completion: 0.71, contribution: 0.0568 },
    { kind: "trust", weight_share: 0.04, completion: 0.96, contribution: 0.0384 },
];

const delta = (tag: string, grade: number) => ({ tag, operator_score_delta: grade * 12, operator_grade_delta: grade, total_score_delta: grade * 0.283 });

const gap = (operator_id: string, name: string, rarity: number, elite: number, level: number, mastery: number, moduleLevel: number, trust: number, missing: string[], subscore: number) => ({
    operator_id,
    name,
    rarity,
    current_elite: elite,
    current_level: level,
    current_skill_level: 7,
    max_mastery: mastery,
    max_module_level: moduleLevel,
    current_trust: trust,
    is_support: false,
    missing,
    deltas: missing.map((t, i) => delta(t, subscore / (missing.length + i * 0.4))),
    subscore_potential_gain: subscore / 100,
    total_potential_gain: (subscore / 100) * 0.283,
});

const withGaps = {
    operators: {
        score_breakdown: scoreBreakdown,
        below_milestone: [
            gap("char_2015_dusk", "Dusk", 6, 2, 74, 1, 0, 96, ["MAX_LEVEL", "M3", "MOD3", "POT6"], 0.94),
            gap("char_1013_chen2", "Ch'en the Holungday", 6, 2, 90, 2, 2, 200, ["M3", "MOD3"], 0.61),
            gap("char_248_mgllan", "Magallan", 6, 1, 68, 0, 0, 42, ["ELITE", "M3", "MOD3", "POT6", "TRUST"], 1.38),
            gap("char_400_weedy", "Weedy", 6, 2, 90, 3, 1, 200, ["MOD3"], 0.22),
            gap("char_134_ifrit", "Ifrit", 6, 2, 82, 2, 3, 178, ["MAX_LEVEL", "M3"], 0.47),
            gap("char_112_siege", "Siege", 6, 2, 90, 3, 3, 112, ["POT6"], 0.09),
            gap("char_140_whitew", "Lappland", 5, 2, 80, 1, 0, 88, ["M3", "MOD3", "POT6"], 0.51),
            gap("char_143_ghost", "Specter", 5, 2, 71, 0, 0, 64, ["MAX_LEVEL", "M3", "MOD3"], 0.44),
            gap("char_128_plosis", "Ptilopsis", 5, 2, 80, 3, 1, 200, ["MOD3"], 0.13),
            gap("char_215_mantic", "Manticore", 5, 1, 55, 0, 0, 31, ["ELITE", "M3", "TRUST"], 0.36),
            gap("char_378_asbest", "Asbestos", 5, 2, 80, 2, 0, 148, ["M3", "MOD3"], 0.18),
        ],
    },
};

const allDone = {
    operators: { score_breakdown: scoreBreakdown, below_milestone: [] },
};

export const RosterGaps = () => (
    <PanelFrame pct="91.3%">
        <OperatorPanel improvements={withGaps} accent={OPERATOR_ACCENT} />
    </PanelFrame>
);

// Nothing left below a milestone: the score breakdown still renders, followed
// by the "everything is maxed" hint instead of the rarity buckets.
export const EveryOperatorMaxed = () => (
    <PanelFrame pct="100%">
        <OperatorPanel improvements={allDone} accent={OPERATOR_ACCENT} />
    </PanelFrame>
);
