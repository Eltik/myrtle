import { SustainabilityBadge, ToggleGroup, ToggleGroupItem } from "frontend";

// The verdict of the rotation's week-long morale simulation (the backend's
// `SustainabilityDto`): `verdict` is "holds_up" or "depletes", `horizon_hours`
// is the simulated window (168 h = 7 days), `depletedCount` is how many
// operators hit zero morale mid-shift, `dormOverflow` the peak number of
// resting operators the dorms could not hold. BasePanel shows it on the right
// of the controls row once a plan has been proposed.

/** The good outcome - a secondary badge with a check mark. */
export const HoldsUp = () => <SustainabilityBadge depletedCount={0} dormOverflow={0} horizonHours={168} verdict="holds_up" />;

/** Someone runs dry: the badge turns destructive and the copy counts who. */
export const Depletes = () => <SustainabilityBadge depletedCount={2} dormOverflow={0} horizonHours={168} verdict="depletes" />;

/** Depletion plus a dorm shortfall - both sentences render on one line. */
export const DepletesDormsShort = () => <SustainabilityBadge depletedCount={4} dormOverflow={3} horizonHours={168} verdict="depletes" />;

/** The singular forms: one operator, one day, one bed. */
export const OneDayHorizon = () => <SustainabilityBadge depletedCount={1} dormOverflow={1} horizonHours={24} verdict="depletes" />;

/** Where it lives: the right end of the controls row, opposite the shift strip (ShiftStrip's own ToggleGroup, drawn inline here). */
export const InControlsRow = () => (
    <div className="flex flex-wrap items-center justify-between gap-3">
        <ToggleGroup aria-label="Rotation shift" value={["now"]}>
            <ToggleGroupItem size="sm" value="now">
                Stationed now
            </ToggleGroupItem>
            <ToggleGroupItem size="sm" value="1">
                Shift 1
            </ToggleGroupItem>
            <ToggleGroupItem size="sm" value="2">
                Shift 2
            </ToggleGroupItem>
            <ToggleGroupItem size="sm" value="3">
                Shift 3
            </ToggleGroupItem>
        </ToggleGroup>
        <SustainabilityBadge depletedCount={0} dormOverflow={0} horizonHours={168} verdict="holds_up" />
    </div>
);
