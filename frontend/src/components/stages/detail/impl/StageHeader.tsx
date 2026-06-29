import { Skull } from "lucide-react";
import type { IStage, IZone } from "#/types/stages";
import { DIFFICULTY_LABEL, STAGE_TYPE_LABEL } from "./constants";
import { descToHtml, zoneLabel } from "./helpers";
import { Pill } from "./primitives";

export function StageHeader({ stage, zone }: { stage: IStage; zone: IZone | undefined }) {
    const { title: zoneTitle, subtitle: zoneSub } = zoneLabel(zone, stage.zoneId);
    return (
        <header>
            <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-md bg-primary px-2.5 py-1 font-bold font-mono text-[13px] text-primary-foreground leading-none tracking-wide">{stage.code}</span>
                {stage.bossMark && (
                    <Pill tone="danger">
                        <Skull className="h-3 w-3" /> Boss
                    </Pill>
                )}
                {stage.difficulty !== "NORMAL" && <Pill tone="warning">{DIFFICULTY_LABEL[stage.difficulty]}</Pill>}
            </div>

            <h1 className="mt-3 text-balance font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight">{stage.name ?? stage.code}</h1>

            <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 font-medium font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">
                <span>{STAGE_TYPE_LABEL[stage.stageType] ?? stage.stageType}</span>
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
                // biome-ignore lint/security/noDangerouslySetInnerHtml: description is trusted backend handbook data sanitized via descToHtml
                <p className="mt-3.5 max-w-[72ch] text-pretty font-sans text-[13px] text-muted-foreground leading-relaxed" dangerouslySetInnerHTML={{ __html: descToHtml(stage.description) }} />
            )}
        </header>
    );
}
