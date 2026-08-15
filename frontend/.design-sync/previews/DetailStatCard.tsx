import { Coins, Gauge, Heart, Timer, Users } from "lucide-react";
import { DetailStatCard, GameIcon, StagesDetailSectionHead } from "frontend";

const UI = (path: string) => `https://api.myrtle.moe/api/assets/textures/ui/${path}`;
const ICON = {
    team: UI("activity/teamquest/icon_tab_team.png"),
    cost: UI("%5Buc%5Dbattlefinish/icon_cost.png"),
    speed: UI("cooperate/battle/cooperate_battle_ui_plugin/speedUp_1x.png"),
};

/** The Overview row exactly as 4-10 — Extinguished Flames renders it. */
export const OverviewGrid = () => (
    <section className="max-w-4xl">
        <StagesDetailSectionHead>Overview</StagesDetailSectionHead>
        <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
            <DetailStatCard icon={<GameIcon src={ICON.cost} alt="Initial DP" />} label="Initial DP" value="10" accent="var(--info)" info="The amount of Deployment Points (DP) you start the battle with." />
            <DetailStatCard icon={<GameIcon src={ICON.cost} alt="DP per tick" />} label="DP / Tick" value="1s" accent="var(--muted-foreground)" info="How often you passively gain 1 DP - a lower number means DP regenerates faster." />
            <DetailStatCard icon={<GameIcon src={ICON.team} alt="Unit Limit" />} label="Unit Limit" value="10" accent="var(--primary)" info="The maximum number of operators you can have deployed on the field at once." />
            <DetailStatCard icon={<GameIcon src={ICON.speed} alt="Move Speed" />} label="Move Speed" value="×0.5" accent="var(--warning)" info="A global multiplier applied to every enemy's movement speed on this stage." />
        </div>
    </section>
);

/** The accent tints the icon well and its inner hairline; omitting it falls back to muted. */
export const AccentTones = () => (
    <div className="grid max-w-4xl grid-cols-2 gap-2.5 sm:grid-cols-4">
        <DetailStatCard icon={<Coins />} label="Sanity" value="21" accent="var(--info)" />
        <DetailStatCard icon={<Users />} label="Unit Limit" value="10" accent="var(--primary)" />
        <DetailStatCard icon={<Gauge />} label="Move Speed" value="×0.5" accent="var(--warning)" />
        <DetailStatCard icon={<Heart />} label="Life Points" value="3" accent="var(--destructive)" />
    </div>
);

export const NoAccent = () => (
    <div className="grid max-w-md grid-cols-2 gap-2.5">
        <DetailStatCard icon={<Timer />} label="Max Play Time" value="Unlimited" />
        <DetailStatCard icon={<Coins />} label="LMD Reward" value="1,140" />
    </div>
);
