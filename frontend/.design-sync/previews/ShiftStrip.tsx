import { AccountFacts, BaseOptimizerProvider, PromotionToggle, ShiftStrip, TooltipProvider } from "frontend";
import { useState } from "react";
import type { IOptimizerAPI } from "../../src/lib/base/use-optimizer";

// ShiftStrip reads the base optimizer's context (`useBaseOptimizer()`) and
// takes no props: `shiftCount` (3 - the rotation planner's SHIFT_COUNT, or
// the catalog's before a plan exists) draws one toggle per shift after
// "Stationed now", and `viewShift` (null = the live base, 1..n = that shift's
// crews) is the selected one. It renders nothing while `shiftCount` is 0,
// i.e. before the catalog has loaded. BasePanel leads the controls row with
// it, followed by PromotionToggle and AccountFacts.

const noop = () => undefined;

/** An `IOptimizerAPI` at rest: a scored 2-4-3 layout, no proposal, no shift selected. */
function optimizerApi(overrides: Partial<IOptimizerAPI>): IOptimizerAPI {
    return {
        layout: [],
        dirty: false,
        catalog: new Map(),
        slots: [],
        formulas: [],
        presets: [],
        shiftCount: 3,
        boardRooms: [],
        catalogLoading: false,
        layoutLoading: false,
        evaluation: undefined,
        evaluating: false,
        evaluationError: null,
        rotation: null,
        rotationLoading: false,
        ignorePromotion: false,
        setIgnorePromotion: noop,
        openRecruitSlots: 0,
        setOpenRecruitSlots: noop,
        factsSaved: null,
        trainingClass: null,
        setTrainingClass: noop,
        claimIntervalHours: undefined,
        setClaimIntervalHours: noop,
        viewShift: null,
        setViewShift: noop,
        shiftRoom: () => undefined,
        proposal: null,
        optimizing: false,
        optimizeError: null,
        runOptimize: noop,
        reset: noop,
        ...overrides,
    };
}

function Stage({ initialShift, shiftCount = 3 }: { initialShift: number | null; shiftCount?: number }) {
    const [viewShift, setViewShift] = useState<number | null>(initialShift);
    const api = optimizerApi({ shiftCount, viewShift, setViewShift });
    return (
        <BaseOptimizerProvider value={api}>
            <ShiftStrip />
        </BaseOptimizerProvider>
    );
}

/** Default: "Stationed now" - the board shows who is actually in each room. */
export const StationedNow = () => <Stage initialShift={null} />;

/** Viewing the second shift: the board swaps every room to that shift's crew. */
export const ViewingShift2 = () => <Stage initialShift={2} />;

/** The controls row as BasePanel lays it out: shift strip, promotion toggle, account facts. */
export const ControlsRow = () => {
    const [viewShift, setViewShift] = useState<number | null>(1);
    const [ignorePromotion, setIgnorePromotion] = useState(false);
    const [openRecruitSlots, setOpenRecruitSlots] = useState(2);
    const [trainingClass, setTrainingClass] = useState<string | null>("Caster");
    const api = optimizerApi({ viewShift, setViewShift, ignorePromotion, setIgnorePromotion, openRecruitSlots, setOpenRecruitSlots, trainingClass, setTrainingClass, factsSaved: true });
    return (
        <TooltipProvider closeDelay={0} delay={350}>
            <BaseOptimizerProvider value={api}>
                <div className="flex flex-wrap items-center gap-4">
                    <ShiftStrip />
                    <PromotionToggle />
                    <AccountFacts />
                </div>
            </BaseOptimizerProvider>
        </TooltipProvider>
    );
};
