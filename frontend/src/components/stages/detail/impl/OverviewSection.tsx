import type { ILevel } from "#/lib/api/level";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./OverviewSection.messages";
import { SectionHead, StatCard } from "./primitives";

export function OverviewSection({ level }: { level: ILevel | null }) {
    const t: TypedT<typeof messages> = useT("stages");
    const opts = level?.options;
    return (
        <section>
            <SectionHead>{t("overview.title")}</SectionHead>
            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
                {opts?.initialCost != null && <StatCard label={t("overview.initialCost")} value={String(opts.initialCost)} info={t("overview.initialCost.info")} />}
                {opts?.costIncreaseTime != null && <StatCard label={t("overview.dpTick")} value={`${opts.costIncreaseTime}s`} info={t("overview.dpTick.info")} />}
                {opts?.characterLimit != null && <StatCard label={t("overview.unitLimit")} value={String(opts.characterLimit)} info={t("overview.unitLimit.info")} />}
                {opts?.moveMultiplier != null && opts.moveMultiplier !== 1 && <StatCard label={t("overview.moveSpeed")} value={`×${opts.moveMultiplier}`} info={t("overview.moveSpeed.info")} />}
            </div>
        </section>
    );
}
