import { Gauge } from "lucide-react";
import { DetailStatCard, FlagRow, InfoHint, StagesDetailSectionHead } from "frontend";

/**
 * The hint reveals its tooltip on hover/focus only, so a static card shows the
 * affordance in place — which is how it always ships: welded to a label.
 */
export const InStatCardLabel = () => (
    <div className="grid max-w-md grid-cols-2 gap-2.5">
        <DetailStatCard icon={<Gauge />} label="Move Speed" value="×0.5" accent="var(--warning)" info="A global multiplier applied to every enemy's movement speed on this stage. ×0.5 means enemies move at half their normal speed." />
        <DetailStatCard icon={<Gauge />} label="DP / Tick" value="1s" accent="var(--muted-foreground)" info="How often you passively gain 1 DP - a lower number means DP regenerates faster." />
    </div>
);

export const InPropertyRows = () => (
    <section className="max-w-md">
        <StagesDetailSectionHead>Properties</StagesDetailSectionHead>
        <div className="flex flex-col rounded-[10px] border border-border bg-card px-3.5 py-1.5">
            <FlagRow label="Predefined Squad" on={false} hint="The stage hands you a fixed roster of operators to clear it with, instead of letting you bring your own." />
            <FlagRow label="Steering Enabled" on={true} hint="A pathfinding flag: when on, enemies steer smoothly around each other and corners rather than snapping tile-to-tile. It rarely affects strategy." />
            <FlagRow label="Cards Selectable" on={false} hint="In a predefined-squad stage, whether you may pick which of the provided operators to deploy. When off, the loadout is locked." />
        </div>
    </section>
);

export const NextToHeading = () => (
    <div className="flex max-w-md items-center gap-1.5">
        <span className="font-medium font-sans text-[13px] text-foreground">Danger Level · Elite 1 Lv. 70</span>
        <InfoHint label="Danger Level">The recommended squad strength for a comfortable clear. Below it, expect to lose life points.</InfoHint>
    </div>
);
