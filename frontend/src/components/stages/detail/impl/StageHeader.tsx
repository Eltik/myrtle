import { Link } from "@tanstack/react-router";
import { Skull } from "lucide-react";
import type { StageVariantKind } from "#/lib/api/stages";
import { emphasizeTagsHtml } from "#/lib/gamedata/richtext";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IStage, IZone } from "#/types/stages";
import { DIFFICULTY_LABEL_KEY, STAGE_TYPE_LABEL_KEY } from "./constants";
import type { messages as detailConstantsMessages } from "./constants.messages";
import { zoneLabel } from "./helpers";
import { Pill } from "./primitives";
import type { messages } from "./StageHeader.messages";

export function StageHeader({ stage, zone, variants }: { stage: IStage; zone: IZone | undefined; variants: { stageId: string; kind: StageVariantKind | null }[] }) {
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
                {variants.length > 0 && (
                    <nav aria-label={t("header.variants")} className="inline-flex items-center rounded-full border border-border p-0.5">
                        {variants.map((v) => {
                            const current = v.stageId === stage.stageId;
                            return (
                                <Link
                                    key={v.stageId}
                                    to="/stages/$stageId"
                                    params={{ stageId: v.stageId }}
                                    aria-current={current ? "page" : undefined}
                                    className={cn("rounded-full px-2.5 py-1 font-medium font-sans text-[11.5px] leading-none transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60", current ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}
                                >
                                    {v.kind === "cm" ? t("header.variant.cm") : v.kind === "adverse" ? t("header.variant.adverse") : t("header.variant.standard")}
                                </Link>
                            );
                        })}
                    </nav>
                )}
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
                <p className="mt-3.5 max-w-[72ch] whitespace-pre-line text-pretty font-sans text-[13px] text-muted-foreground leading-relaxed" dangerouslySetInnerHTML={{ __html: emphasizeTagsHtml(stage.description) }} />
            )}
        </header>
    );
}
