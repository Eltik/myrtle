import { ChevronRight, Skull } from "lucide-react";
import { useState } from "react";
import { Dialog, DialogDescription, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { STAGE_GROUP_DESCRIPTION, STAGE_GROUP_LABEL } from "#/lib/registry/stage-groups";
import { coverImage, type IEventVM } from "./derive";
import { PreviewFallback, STAGE_GROUP_ICON } from "./PreviewFallback";
import { StagePreview } from "./StagePreview";

function Stat({ label, value, tone }: { label: string; value: string; tone?: string }) {
    return (
        <div className="rounded-[10px] border border-border bg-secondary/40 px-3 py-2.5">
            <div className="font-mono text-[9px] text-muted-foreground uppercase tracking-[0.14em]">{label}</div>
            <div className="mt-1 truncate font-mono font-semibold text-[13px]" style={{ color: tone ?? "var(--foreground)" }}>
                {value}
            </div>
        </div>
    );
}

/**
 * Enlarged-banner detail view for an event / chapter / mode season. Opened from
 * a row thumbnail or the featured hero; "Browse stages" expands the row in the
 * list behind it.
 */
export function EventDetailDialog({ event, onClose, onBrowse }: { event: IEventVM | null; onClose: () => void; onBrowse: (event: IEventVM) => void }) {
    // Keep rendering the last event through the close animation.
    const [last, setLast] = useState<IEventVM | null>(event);
    if (event && event !== last) setLast(event);
    const ev = event ?? last;
    if (!ev) return null;

    const cover = coverImage(ev);
    const Icon = STAGE_GROUP_ICON[ev.group];
    const totalSanity = ev.stages.reduce((n, s) => n + Math.max(0, s.apCost), 0);
    const previews = ev.stages.filter((s) => s.preview).slice(0, 6);

    return (
        <Dialog
            open={event !== null}
            onOpenChange={(o) => {
                if (!o) onClose();
            }}
        >
            <DialogPopup className="max-w-2xl gap-0 overflow-hidden p-0" closeProps={{ className: "absolute end-2 top-2 text-white/85 hover:bg-white/15 hover:text-white" }}>
                {/* Enlarged banner */}
                <div className="relative h-40 flex-none overflow-hidden sm:h-64">
                    <PreviewFallback tone={ev.tone} group={ev.group} iconClassName="h-9 w-9" />
                    {cover && <StagePreview src={cover} />}
                    <div className="pointer-events-none absolute inset-0" style={{ background: "linear-gradient(0deg, rgba(7,9,12,0.88) 0%, rgba(7,9,12,0.35) 45%, rgba(7,9,12,0.12) 100%)" }} />
                    <div className="absolute right-4.5 bottom-4 left-4.5 sm:right-6 sm:bottom-4.5 sm:left-6">
                        <div className="mb-2 flex flex-wrap items-center gap-2">
                            <span
                                className="inline-flex h-5.5 items-center gap-1.5 rounded-full border px-2.5 font-mono font-semibold text-[9.5px] uppercase tracking-[0.09em]"
                                style={{ borderColor: `color-mix(in oklch, ${ev.tone} 55%, transparent)`, background: `color-mix(in oklch, ${ev.tone} 16%, rgba(0,0,0,0.4))`, color: ev.tone }}
                            >
                                <Icon className="h-3 w-3" aria-hidden="true" />
                                {STAGE_GROUP_LABEL[ev.group]}
                            </span>
                            {ev.kicker && <span className="font-mono text-[10px] text-white/60 uppercase tracking-[0.12em]">{ev.kicker}</span>}
                        </div>
                        <DialogTitle className="text-balance font-bold font-sans text-[21px] text-white leading-[1.05] tracking-tight sm:text-[26px]" style={{ textShadow: "0 2px 20px rgba(0,0,0,0.55)" }}>
                            {ev.title}
                        </DialogTitle>
                    </div>
                </div>

                {/* Category details */}
                <div className="flex flex-col gap-4 p-4.5 pt-4 sm:gap-4.5 sm:p-6 sm:pt-5">
                    <DialogDescription className="text-pretty font-sans text-[13px] leading-[1.6]">{STAGE_GROUP_DESCRIPTION[ev.group]}</DialogDescription>

                    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                        <Stat label="Operations" value={String(ev.stageCount)} />
                        <Stat label="Bosses" value={ev.bossCount > 0 ? String(ev.bossCount) : "-"} tone={ev.bossCount > 0 ? "var(--enemy-boss)" : undefined} />
                        <Stat label="Codes" value={ev.codeRange || "-"} />
                        <Stat label="Sanity total" value={totalSanity > 0 ? `${totalSanity} ◆` : "Free"} />
                    </div>

                    {previews.length > 0 && (
                        <div>
                            <div className="mb-2 font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em]">Field previews</div>
                            <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
                                {previews.map((s) => (
                                    <div key={s.stageId} className="relative aspect-16/10 overflow-hidden rounded-md border border-border">
                                        <PreviewFallback tone={ev.tone} group={ev.group} iconClassName="h-3 w-3" />
                                        {s.preview && <StagePreview src={s.preview} />}
                                    </div>
                                ))}
                            </div>
                        </div>
                    )}

                    <div className="flex flex-col gap-3 border-border border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
                        <span className="font-mono text-[10.5px] text-muted-foreground">
                            {ev.bossCount > 0 && (
                                <span className="mr-3 inline-flex items-center gap-1" style={{ color: "var(--enemy-boss)" }}>
                                    <Skull className="h-3 w-3" aria-hidden="true" /> Boss operation{ev.bossCount === 1 ? "" : "s"} inside
                                </span>
                            )}
                            {ev.statusLabel}
                        </span>
                        <button
                            type="button"
                            onClick={() => onBrowse(ev)}
                            className="inline-flex h-10 w-full flex-none cursor-pointer items-center justify-center gap-1.5 whitespace-nowrap rounded-lg bg-primary px-4 font-sans font-semibold text-[12.5px] text-primary-foreground shadow-sm transition-transform hover:-translate-y-px sm:h-8.5 sm:w-auto"
                        >
                            Browse stages
                            <ChevronRight className="h-3.25 w-3.25" aria-hidden="true" />
                        </button>
                    </div>
                </div>
            </DialogPopup>
        </Dialog>
    );
}
