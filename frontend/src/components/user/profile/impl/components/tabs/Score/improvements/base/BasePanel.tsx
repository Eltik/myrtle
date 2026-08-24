import { Download } from "lucide-react";
import { type CSSProperties, type RefObject, useRef, useState } from "react";
import { Dialog, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { toastManager } from "#/components/ui/toast";
import type { IBaseImprovements, IImprovementsResponse } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { CollapsibleSection, EmptyHint, PANEL_PADDING, Pill, SectionHeader, TEXT_KICKER } from "../shared";
import { ComparisonSection } from "./ComparisonSection";
import { ExportPlanContent } from "./ExportPlanContent";
import { exportPlanAsImage } from "./exportPlan";
import { PeakGrid } from "./PeakSection";
import { PerceptionSection } from "./PerceptionSection";
import { AccentKicker, LayoutChips, YieldHeadline } from "./parts";
import { RotationSection } from "./RotationSection";
import { ShiftPoster } from "./ShiftSection";
import { compactNum } from "./yield";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

/** The panel accent as a CSS variable, so `--imp-accent` reaches every base
 *  component without threading a prop through the whole tree. The dialog body
 *  re-sets it because `DialogPopup` renders in a portal outside this subtree. */
function accentVar(accent: string): CSSProperties {
    return { "--imp-accent": accent } as CSSProperties;
}

/**
 * The Base subscore's inline panel: the current layout, the current → optimal
 * yield headline, and the entry point to the full plan dialog.
 */
export function BasePanel({ improvements, accent }: IProps) {
    const { base } = improvements;
    if (!base.optimal && !base.rotation) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>No base layout synced yet. Hit refresh once you're in your base.</EmptyHint>
            </div>
        );
    }

    // Difference between the player's live base and the optimal peak.
    const curVal = base.current?.yield_total_value ?? 0;
    const optVal = base.optimal?.yield_total_value ?? 0;
    const gain = optVal - curVal;

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-5`} style={accentVar(accent)}>
            {base.layout.length > 0 && (
                <div className="flex flex-col gap-2">
                    <SectionHeader title="Current layout" accent={accent} />
                    <LayoutChips layout={base.layout} detailed />
                </div>
            )}

            {base.current && base.optimal && (
                <div className="flex flex-col gap-2 rounded-md border border-border/40 bg-muted/10 p-3">
                    <div className="flex flex-wrap items-center justify-between gap-2">
                        <AccentKicker>Current vs optimal</AccentKicker>
                        {gain > 0 && <Pill color={accent}>+{compactNum(gain)} value/day</Pill>}
                    </div>
                    <YieldHeadline current={base.current} optimal={base.optimal} />
                    {base.claim && (
                        <p className="text-[11px] text-muted-foreground">
                            Log in every <span className="font-mono tabular-nums">{Math.floor(base.claim.next_full_hours)}h</span> to lose nothing
                            {(() => {
                                const daily = base.claim.intervals.find((i) => i.hours === 24);
                                return daily && daily.lost_lmd_per_day >= 1 ? (
                                    <>
                                        {" "}
                                        - once a day costs <span className="font-mono text-destructive/90 tabular-nums">−{Math.round(daily.lost_lmd_per_day).toLocaleString()} LMD/day</span>
                                    </>
                                ) : null;
                            })()}.
                            {base.unrotated && base.unrotated.depleted.length > 0 && (
                                <>
                                    {" "}
                                    Without swapping, <span className="font-mono tabular-nums">{base.unrotated.depleted.length}</span> operators run dry.
                                </>
                            )}
                        </p>
                    )}

                    {/* Dialog opens the full plan as a focused full-screen view on every
                        screen size, desktop included. */}
                    <FullPlanDialog base={base} accent={accent} />
                </div>
            )}
        </div>
    );
}

/** The full-plan dialog plus its "Export plan" action. The exported image captures a
 *  dedicated wide poster (`ExportPlanContent`) kept off-screen, NOT the tall dialog
 *  body, so the PNG is well-proportioned rather than a long vertical ribbon. */
function FullPlanDialog({ base, accent }: { base: IBaseImprovements; accent: string }) {
    const captureRef = useRef<HTMLDivElement>(null);
    return (
        <Dialog>
            <DialogTrigger className={cn("self-start rounded-md border border-border/45 bg-background/60 px-2 py-1 transition-colors hover:border-foreground/25", TEXT_KICKER, "text-muted-foreground hover:text-foreground")}>View full base plan →</DialogTrigger>
            <DialogPopup className="max-w-4xl">
                <div className="flex items-center justify-between gap-3 px-6 pt-6">
                    <DialogTitle className="text-base">Base optimization plan</DialogTitle>
                    <ExportPlanButton targetRef={captureRef} />
                </div>
                <DialogPanel>
                    <BasePlanBody base={base} accent={accent} />
                </DialogPanel>
            </DialogPopup>
            {/* Off-screen export layout - present in the DOM so its ref is ready, but
                visually hidden and laid out at its own fixed width. */}
            <div aria-hidden className="pointer-events-none fixed top-0 -left-24999.75 opacity-0">
                <div ref={captureRef}>
                    <ExportPlanContent base={base} />
                </div>
            </div>
        </Dialog>
    );
}

/** Renders the off-screen poster (`targetRef`) to a downloadable PNG. */
function ExportPlanButton({ targetRef }: { targetRef: RefObject<HTMLDivElement | null> }) {
    const [busy, setBusy] = useState(false);
    const handleExport = async () => {
        if (busy || !targetRef.current) return;
        setBusy(true);
        try {
            await exportPlanAsImage(targetRef.current, "base-plan.png");
        } catch {
            toastManager.add({
                id: `base-export-${Date.now()}`,
                title: "Couldn't export plan",
                description: "Something went wrong rendering the image. Try again in a moment.",
                type: "error",
            });
        } finally {
            setBusy(false);
        }
    };
    return (
        <button type="button" onClick={handleExport} disabled={busy} className={cn("flex shrink-0 items-center gap-1.5 rounded-md border border-border/45 bg-background/60 px-2 py-1 transition-colors hover:border-foreground/25 disabled:opacity-60", TEXT_KICKER, "text-muted-foreground hover:text-foreground")}>
            <Download className="size-3.5" />
            {busy ? "Exporting…" : "Export plan"}
        </button>
    );
}

/**
 * The consolidated full base plan, laid out like the downloadable poster: a compact
 * header (layout + yield), then the 3-shift rotation as a side-by-side grid of
 * colour-accented room blocks. The sustained 24/7 rotation, peak single-shift, resource
 * economy, and full comparison are tucked into collapsible sections so the dialog opens
 * focused instead of as one long wall.
 */
function BasePlanBody({ base, accent }: { base: IBaseImprovements; accent: string }) {
    const hasShifts = Boolean(base.shift_rotation && base.shift_rotation.shifts.length > 0);
    return (
        <div className="flex flex-col gap-4" style={accentVar(accent)}>
            <PlanHeader base={base} />

            {hasShifts && base.shift_rotation ? <ShiftPoster rotation={base.shift_rotation} /> : base.optimal && <PeakGrid optimal={base.optimal} />}

            {base.rotation && base.rotation.rooms.length > 0 && (
                <CollapsibleSection title="Sustained 24/7 rotation" count={`+${base.rotation.sustained_efficiency.toFixed(1)}% sustained`} accent={accent} defaultOpen={!hasShifts}>
                    <RotationSection rotation={base.rotation} />
                </CollapsibleSection>
            )}

            {hasShifts && base.optimal && (
                <CollapsibleSection title="Peak single-shift" count={`+${base.optimal.total_production_efficiency.toFixed(1)}%`} accent={accent}>
                    <PeakGrid optimal={base.optimal} />
                </CollapsibleSection>
            )}

            {base.perception && base.perception.consumers.length > 0 && (
                <CollapsibleSection title="Resource economy (max ceiling)" count={`+${Math.max(...base.perception.consumers.map((c) => c.bonus_pct)).toFixed(0)}% peak`} accent={accent}>
                    <PerceptionSection plan={base.perception} />
                </CollapsibleSection>
            )}

            {base.current && base.optimal && (
                <CollapsibleSection title="Current vs optimal detail" accent={accent}>
                    <ComparisonSection current={base.current} optimal={base.optimal} />
                </CollapsibleSection>
            )}
        </div>
    );
}

/** Poster-style header: the base layout as colour chips (doubling as the room-colour
 *  legend) and the current → optimal yield headline. */
function PlanHeader({ base }: { base: IBaseImprovements }) {
    return (
        <div className="flex flex-col gap-2.5 border-border/40 border-b pb-3">
            <LayoutChips layout={base.layout} />
            {base.current && base.optimal && <YieldHeadline current={base.current} optimal={base.optimal} />}
        </div>
    );
}
