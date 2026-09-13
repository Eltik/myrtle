import { BaseOptimizerProvider, PromotionToggle, TooltipProvider } from "frontend";
import { useState } from "react";
import type { IOptimizerAPI } from "../../src/lib/base/use-optimizer";

// PromotionToggle reads the base optimizer's context (`useBaseOptimizer()`):
// `ignorePromotion` and `setIgnorePromotion`. It never takes props - the
// state lives in the `IOptimizerAPI` object BaseOptimizer builds from
// `useOptimizer(uid)` and hands to `BaseOptimizerProvider`. BasePanel renders
// it in the controls row under the headline strip, inside a TooltipProvider
// (hovering the row explains what the switch does).

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

function Stage({ initial }: { initial: boolean }) {
    const [ignorePromotion, setIgnorePromotion] = useState(initial);
    const api = optimizerApi({ ignorePromotion, setIgnorePromotion });
    return (
        <TooltipProvider closeDelay={0} delay={350}>
            <BaseOptimizerProvider value={api}>
                <PromotionToggle />
            </BaseOptimizerProvider>
        </TooltipProvider>
    );
}

/** Default: plan only with the base skills each operator has actually unlocked. */
export const Off = () => <Stage initial={false} />;

/** On: every operator is planned at their highest base skills, promoted or not; locked skills show greyed on the board. */
export const On = () => <Stage initial />;
