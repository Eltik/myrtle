import { DetailKicker } from "frontend";

/** The section labels the stage detail page actually prints, top to bottom. */
const STAGE_SECTIONS = ["Overview", "Spawn Schedule", "Drops", "Waves · 5", "Enemies · 7", "Properties", "Identifiers"];

export const SectionLabels = () => (
    <div className="flex flex-col gap-3">
        {STAGE_SECTIONS.map((label) => (
            <DetailKicker key={label}>{label}</DetailKicker>
        ))}
    </div>
);

export const InPanelHeader = () => (
    <section className="max-w-xl overflow-hidden rounded-[14px] border border-border bg-card">
        <div className="flex flex-wrap items-center gap-2.5 border-border border-b px-4 py-3">
            <DetailKicker>Spawn Schedule</DetailKicker>
            <span className="font-mono text-[11px] text-muted-foreground tabular-nums">20 groups</span>
            <span className="h-px flex-1 bg-border" />
        </div>
        <p className="m-0 px-4 py-3.5 font-sans text-[12.5px] text-muted-foreground leading-relaxed">Every SPAWN action in 4-10 — Extinguished Flames, ordered by the second it fires.</p>
    </section>
);

/** How MapSettings heads its panel: a bare kicker, then the controls. */
export const AsPanelTitle = () => (
    <div className="flex max-w-xl flex-col rounded-[10px] border border-border bg-card px-3.5 py-2.5">
        <div className="mb-1 flex items-center">
            <DetailKicker>Settings</DetailKicker>
        </div>
        <p className="m-0 font-sans text-[12px] text-muted-foreground leading-relaxed">Route lines, enemy icons, wait timers and walking chibis, toggled per map.</p>
    </div>
);
