import { Skull } from "lucide-react";
import { emphasizeTagsHtml } from "#/lib/gamedata/richtext";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IStage, IZone } from "#/types/stages";
import { DIFFICULTY_LABEL_KEY, STAGE_TYPE_LABEL_KEY } from "./constants";
import type { messages as detailConstantsMessages } from "./constants.messages";
import { zoneLabel } from "./helpers";
import { Pill } from "./primitives";
import type { messages } from "./StageHeader.messages";

export function StageHeader({ stage, zone }: { stage: IStage; zone: IZone | undefined }) {
    const t: TypedT<typeof messages> = useT("stages");
    const tConst: TypedT<typeof detailConstantsMessages> = useT("stages");
    const { title: zoneTitle, subtitle: zoneSub } = zoneLabel(zone, stage.zoneId);
    const stageTypeKey = STAGE_TYPE_LABEL_KEY[stage.stageType];
    return (
        <header>
            <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-md bg-primary px-2.5 py-1 font-bold font-mono text-[13px] text-primary-foreground leading-none tracking-wide">{stage.code}</span>
                {stage.bossMark && (
                    <Pill tone="danger">
                        <Skull className="h-3 w-3" /> {t("header.boss")}
                    </Pill>
                )}
                {stage.difficulty !== "NORMAL" && <Pill tone="warning">{tConst(DIFFICULTY_LABEL_KEY[stage.difficulty])}</Pill>}
            </div>

            <h1 className="mt-3 text-balance font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight">{stage.name ?? stage.code}</h1>

            <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                <span>{stageTypeKey ? tConst(stageTypeKey) : stage.stageType}</span>
                <span aria-hidden="true">·</span>
                <span className="text-foreground normal-case tracking-normal">{zoneTitle}</span>
                {zoneSub && (
                    <>
                        <span aria-hidden="true">·</span>
                        <span className="normal-case tracking-normal">{zoneSub}</span>
                    </>
                )}
            </div>

            {stage.description && (
                // biome-ignore lint/security/noDangerouslySetInnerHtml: description is trusted backend handbook data sanitized via emphasizeTagsHtml
                <p className="mt-3.5 max-w-[72ch] text-pretty font-sans text-[13px] text-muted-foreground leading-relaxed" dangerouslySetInnerHTML={{ __html: emphasizeTagsHtml(stage.description) }} />
            )}
        </header>
    );
}
