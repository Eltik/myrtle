import { asset } from "#/components/operators/detail/impl/assets";
import type { ILevel } from "#/lib/api/level";
import { GameIcon, SectionHead, StatCard } from "./primitives";

const UI = (path: string) => asset(`/textures/ui/${path}`);
const ICON = {
    team: UI("activity/teamquest/icon_tab_team.png"),
    cost: UI("%5Buc%5Dbattlefinish/icon_cost.png"),
    speed: UI("cooperate/battle/cooperate_battle_ui_plugin/speedUp_1x.png"),
};

export function OverviewSection({ level }: { level: ILevel | null }) {
    const opts = level?.options;
    return (
        <section>
            <SectionHead>Overview</SectionHead>
            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
                {opts?.initialCost != null && <StatCard icon={<GameIcon src={ICON.cost} alt="Initial DP" />} label="Initial DP" value={String(opts.initialCost)} accent="var(--info)" info="The amount of Deployment Points (DP) you start the battle with." />}
                {opts?.costIncreaseTime != null && <StatCard icon={<GameIcon src={ICON.cost} alt="DP per tick" />} label="DP / Tick" value={`${opts.costIncreaseTime}s`} accent="var(--muted-foreground)" info="How often you passively gain 1 DP - a lower number means DP regenerates faster." />}
                {opts?.characterLimit != null && <StatCard icon={<GameIcon src={ICON.team} alt="Unit Limit" />} label="Unit Limit" value={String(opts.characterLimit)} accent="var(--primary)" info="The maximum number of operators you can have deployed on the field at once." />}
                {opts?.moveMultiplier != null && opts.moveMultiplier !== 1 && (
                    <StatCard icon={<GameIcon src={ICON.speed} alt="Move Speed" />} label="Move Speed" value={`×${opts.moveMultiplier}`} accent="var(--warning)" info="A global multiplier applied to every enemy's movement speed on this stage. ×0.5 means enemies move at half their normal speed." />
                )}
            </div>
        </section>
    );
}
