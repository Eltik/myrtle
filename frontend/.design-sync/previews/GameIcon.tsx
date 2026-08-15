import { DetailStatCard, GameIcon, StagesDetailSectionHead } from "frontend";

/** The in-game UI sprites OverviewSection pulls off the asset server. */
const UI = (path: string) => `https://api.myrtle.moe/api/assets/textures/ui/${path}`;
const ICON = {
    team: UI("activity/teamquest/icon_tab_team.png"),
    cost: UI("%5Buc%5Dbattlefinish/icon_cost.png"),
    speed: UI("cooperate/battle/cooperate_battle_ui_plugin/speedUp_1x.png"),
};

/** The sprites are white line art with a drop shadow — they are authored for a dark well. */
export const StageSprites = () => (
    <div className="flex w-fit flex-wrap gap-8 rounded-[10px] border border-border p-6" style={{ background: "linear-gradient(155deg, #1e2026, #111317)" }}>
        {[
            { src: ICON.cost, alt: "Deployment Points", label: "Initial DP" },
            { src: ICON.team, alt: "Unit Limit", label: "Unit Limit" },
            { src: ICON.speed, alt: "Move Speed", label: "Move Speed" },
        ].map((i) => (
            <div key={i.label} className="flex flex-col items-center gap-2">
                <GameIcon src={i.src} alt={i.alt} />
                <span className="font-medium font-mono text-[10px] uppercase leading-none tracking-[0.12em]" style={{ color: "hsla(0,0%,78%,0.92)" }}>
                    {i.label}
                </span>
            </div>
        ))}
    </div>
);

/** Where they actually ship: inside DetailStatCard's tinted icon well. */
export const InStatCards = () => (
    <section className="max-w-4xl">
        <StagesDetailSectionHead>Overview</StagesDetailSectionHead>
        <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
            <DetailStatCard icon={<GameIcon src={ICON.cost} alt="Initial DP" />} label="Initial DP" value="10" accent="var(--info)" />
            <DetailStatCard icon={<GameIcon src={ICON.cost} alt="DP per tick" />} label="DP / Tick" value="1s" accent="var(--muted-foreground)" />
            <DetailStatCard icon={<GameIcon src={ICON.team} alt="Unit Limit" />} label="Unit Limit" value="10" accent="var(--primary)" />
            <DetailStatCard icon={<GameIcon src={ICON.speed} alt="Move Speed" />} label="Move Speed" value="×0.5" accent="var(--warning)" />
        </div>
    </section>
);
