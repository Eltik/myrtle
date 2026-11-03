import { Link } from "@tanstack/react-router";
import { Skull } from "lucide-react";
import { Badge } from "#/components/ui/badge";
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
    const hasMarks = stage.bossMark || stage.difficulty !== "NORMAL" || variants.length > 0;
    return (
        <header>
            {hasMarks && (
                <div className="mb-3 flex flex-wrap items-center gap-2">
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
                                        className={cn(
                                            "pointer-coarse:after:-translate-1/2 relative rounded-full px-2.5 py-1 font-medium font-sans text-[11.5px] leading-none transition-colors pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:h-11 pointer-coarse:after:w-full pointer-coarse:after:min-w-11 focus-visible:ring-2 focus-visible:ring-ring",
                                            current ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground",
                                        )}
                                    >
                                        {v.kind === "cm" ? t("header.variant.cm") : v.kind === "adverse" ? t("header.variant.adverse") : t("header.variant.standard")}
                                    </Link>
                                );
                            })}
                        </nav>
                    )}
                </div>
            )}

            <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
                {stage.name && (
                    <Badge variant="outline" size="lg" className="border-transparent bg-primary font-mono font-semibold text-primary-foreground tabular-nums">
                        {stage.code}
                    </Badge>
                )}
                <h1 className="m-0 text-balance font-bold font-sans text-2xl text-foreground leading-[1.1] tracking-tight sm:text-3xl">{stage.name ?? stage.code}</h1>
            </div>

            <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 font-medium font-mono text-[10.5px] text-muted-foreground">
                <span>{stageTypeKey ? tConst(stageTypeKey) : stage.stageType}</span>
                <span aria-hidden="true">·</span>
                <span className="text-foreground">{zoneTitle}</span>
                {zoneSub && (
                    <>
                        <span aria-hidden="true">·</span>
                        <span>{zoneSub}</span>
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
