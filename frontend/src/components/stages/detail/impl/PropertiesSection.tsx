import type { ILevel } from "#/lib/api/level";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IStage } from "#/types/stages";
import type { messages } from "./PropertiesSection.messages";
import { FlagRow, Meta, SectionHead } from "./primitives";

export function PropertiesSection({ stage, level }: { stage: IStage; level: ILevel | null }) {
    const t: TypedT<typeof messages> = useT("stages");
    const opts = level?.options;
    return (
        <>
            <section>
                <SectionHead>{t("props.title")}</SectionHead>
                <div className="flex flex-col gap-2.5">
                    <div className="flex flex-col rounded-[10px] border border-border bg-card px-3.5 py-1.5">
                        <FlagRow label={t("props.canPractice")} on={stage.canPractice} />
                        <FlagRow label={t("props.battleReplay")} on={stage.canBattleReplay} />
                        <FlagRow label={t("props.autoDeploy")} on={stage.canMultipleBattle} />
                        <FlagRow label={t("props.storyOnly")} on={stage.isStoryOnly} />
                        <FlagRow label={t("props.predefinedSquad")} on={stage.isPredefined} hint={t("props.predefinedSquad.hint")} />
                        {opts?.isTrainingLevel != null && <FlagRow label={t("props.trainingLevel")} on={opts.isTrainingLevel} hint={t("props.trainingLevel.hint")} />}
                        {opts?.steeringEnabled != null && <FlagRow label={t("props.steeringEnabled")} on={opts.steeringEnabled} hint={t("props.steeringEnabled.hint")} />}
                        {opts?.isPredefinedCardsSelectable != null && <FlagRow label={t("props.cardsSelectable")} on={opts.isPredefinedCardsSelectable} hint={t("props.cardsSelectable.hint")} />}
                    </div>
                </div>
            </section>

            <section>
                <SectionHead>{t("props.ids.title")}</SectionHead>
                <div className="grid grid-cols-2 gap-2">
                    <Meta label={t("props.ids.stage")} value={stage.stageId} />
                    <Meta label={t("props.ids.level")} value={stage.levelId ?? "-"} />
                    <Meta label={t("props.ids.zone")} value={stage.zoneId} />
                    {stage.hardStagedId && <Meta label={t("props.ids.challenge")} value={stage.hardStagedId} />}
                </div>
            </section>
        </>
    );
}
