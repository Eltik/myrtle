import { Skull } from "lucide-react";
import { ListStagePreview, PreviewFallback } from "frontend";

/** Backend-resolved art paths, exactly as `/static/stage-index` emits them. */
const BANNER_MAIN_4 = "textures/spritepack/ui_home_act_banner_zone_0/main_4.png";
const MAP_PREVIEW = (stageId: string) => `textures/arts/ui/stage_mappreview_h2_main_04_0/${stageId}.png`;

const CARDS = [
    { stageId: "main_04-08", badge: "4-8", title: "Acute Stress Disorder", ap: 21, boss: false },
    { stageId: "main_04-09", badge: "4-9", title: "Bad to the Bone", ap: 21, boss: false },
    { stageId: "main_04-10", badge: "4-10", title: "Extinguished Flames", ap: 21, boss: true },
];

export const StageCardArt = () => (
    <div className="grid max-w-2xl grid-cols-3 gap-3">
        {CARDS.map((c) => (
            <div key={c.stageId} className="flex w-full flex-col overflow-hidden rounded-lg border border-border bg-card/55 text-left">
                <div className="relative aspect-16/10 w-full flex-none overflow-hidden">
                    <PreviewFallback tone="var(--primary)" group="story" />
                    <ListStagePreview src={MAP_PREVIEW(c.stageId)} />
                    <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.9) 0%, rgba(7,9,12,0.15) 46%, rgba(7,9,12,0) 66%)" }} />
                    <span className="absolute bottom-1.75 left-1.75 z-2 inline-flex h-4.5 items-center rounded-[5px] border px-1.75 font-bold font-mono text-[10px] leading-none" style={{ borderColor: "color-mix(in oklch, var(--primary) 50%, transparent)", background: "rgba(0,0,0,0.6)", color: "var(--primary)" }}>
                        {c.badge}
                    </span>
                    {c.boss && (
                        <span className="absolute top-1.75 right-1.75 z-2 inline-flex h-4.25 items-center gap-1 rounded-full px-1.75 font-bold font-mono text-[8.5px] text-white uppercase tracking-[0.08em]" style={{ background: "color-mix(in oklch, var(--enemy-boss) 88%, black)" }}>
                            <Skull className="h-2.5 w-2.5" /> Boss
                        </span>
                    )}
                </div>
                <div className="flex flex-col gap-1.5 px-2.5 pt-2 pb-2.5">
                    <span className="line-clamp-2 min-h-8 font-sans font-semibold text-[12.5px] text-foreground leading-tight">{c.title}</span>
                    <span className="font-mono text-[9.5px] text-muted-foreground">{c.ap} ◆ Sanity</span>
                </div>
            </div>
        ))}
    </div>
);

/** Event banners load through the same component at hero scale. */
export const EventBanner = () => (
    <div className="relative h-60 max-w-2xl overflow-hidden rounded-2xl border border-border bg-card">
        <PreviewFallback tone="var(--primary)" group="story" iconClassName="h-8 w-8" />
        <ListStagePreview src={BANNER_MAIN_4} />
        <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.85) 0%, rgba(7,9,12,0.35) 45%, rgba(7,9,12,0) 70%)" }} />
        <div className="absolute" style={{ left: 28, right: 28, bottom: 28 }}>
            <div className="text-balance font-bold font-sans text-[34px] text-white leading-[1.02] tracking-[-0.03em]" style={{ textShadow: "0 2px 24px rgba(0,0,0,0.55)" }}>
                Burning Run
            </div>
            <div className="mt-3 font-mono text-[11px] text-white/75">4-1 ~ SW-EV-4 · 24 operations</div>
        </div>
    </div>
);

/** On a 404 the image hides itself and the group wash underneath shows through. */
export const MissingArt = () => (
    <div className="grid max-w-md grid-cols-2 gap-3">
        <div className="relative aspect-16/10 overflow-hidden rounded-lg border border-border">
            <PreviewFallback tone="var(--primary)" group="story" />
            <ListStagePreview src={MAP_PREVIEW("main_04-10")} />
        </div>
        <div className="relative aspect-16/10 overflow-hidden rounded-lg border border-border">
            <PreviewFallback tone="#b78fe6" group="is" />
            <ListStagePreview src="textures/arts/ui/stage_mappreview_h2_rogue_5_0/rogue_5_b-4.png" />
        </div>
    </div>
);
