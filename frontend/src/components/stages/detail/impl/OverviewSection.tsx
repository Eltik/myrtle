import { asset } from "#/components/operators/detail/impl/assets";
import type { ILevel } from "#/lib/api/level";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./OverviewSection.messages";
import { GameIcon, SectionHead, StatCard } from "./primitives";

const UI = (path: string) => asset(`/textures/ui/${path}`);
const ICON = {
    team: UI("activity/teamquest/icon_tab_team.png"),
    cost: UI("%5Buc%5Dbattlefinish/icon_cost.png"),
    speed: UI("cooperate/battle/cooperate_battle_ui_plugin/speedUp_1x.png"),
};

export function OverviewSection({ level }: { level: ILevel | null }) {
    const t: TypedT<typeof messages> = useT("stages");
    const opts = level?.options;
    return (
        <section>
            <SectionHead>{t("overview.title")}</SectionHead>
            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
                {opts?.initialCost != null && <StatCard icon={<GameIcon src={ICON.cost} alt={t("overview.initialCost")} />} label={t("overview.initialCost")} value={String(opts.initialCost)} accent="var(--info)" info={t("overview.initialCost.info")} />}
                {opts?.costIncreaseTime != null && <StatCard icon={<GameIcon src={ICON.cost} alt={t("overview.dpTick.alt")} />} label={t("overview.dpTick")} value={`${opts.costIncreaseTime}s`} accent="var(--muted-foreground)" info={t("overview.dpTick.info")} />}
                {opts?.characterLimit != null && <StatCard icon={<GameIcon src={ICON.team} alt={t("overview.unitLimit")} />} label={t("overview.unitLimit")} value={String(opts.characterLimit)} accent="var(--primary)" info={t("overview.unitLimit.info")} />}
                {opts?.moveMultiplier != null && opts.moveMultiplier !== 1 && <StatCard icon={<GameIcon src={ICON.speed} alt={t("overview.moveSpeed")} />} label={t("overview.moveSpeed")} value={`×${opts.moveMultiplier}`} accent="var(--warning)" info={t("overview.moveSpeed.info")} />}
            </div>
        </section>
    );
}
