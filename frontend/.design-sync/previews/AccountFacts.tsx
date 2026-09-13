import { AccountFacts, BaseOptimizerProvider, TooltipProvider } from "frontend";
import { useState } from "react";
import type { IOptimizerAPI } from "../../src/lib/base/use-optimizer";

// AccountFacts reads the base optimizer's context (`useBaseOptimizer()`) and
// takes no props. It declares the two account facts the roster sync cannot
// read: recruit slots purchased beyond the first (+0..+3, which prices
// per-slot HR skills like Lin's Meritocracy) and the class currently
// training (ranks trainer hints). `factsSaved` is null until the Doctor edits
// something, then true ("saved" - the profile owner's token wrote it) or
// false ("what-if" - a viewer's session-only override). BasePanel renders it
// at the end of the controls row inside a TooltipProvider.

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

interface IStageProps {
    slots: number;
    trainingClass: string | null;
    /** What an edit reports back: null = untouched, true = saved to the profile, false = session what-if. */
    saved: boolean | null;
}

function Stage({ slots, trainingClass, saved }: IStageProps) {
    const [openRecruitSlots, setOpenRecruitSlots] = useState(slots);
    const [training, setTraining] = useState<string | null>(trainingClass);
    const api = optimizerApi({ openRecruitSlots, setOpenRecruitSlots, trainingClass: training, setTrainingClass: setTraining, factsSaved: saved });
    return (
        <TooltipProvider closeDelay={0} delay={350}>
            <BaseOptimizerProvider value={api}>
                <AccountFacts />
            </BaseOptimizerProvider>
        </TooltipProvider>
    );
}

/** A profile with nothing declared: +0 slots, nobody training, no save marker yet. */
export const Untouched = () => <Stage saved={null} slots={0} trainingClass={null} />;

/** The owner has declared two extra recruit slots and a Caster in training; the edit wrote through, so "saved" shows. */
export const SavedToProfile = () => <Stage saved slots={2} trainingClass="Caster" />;

/** A viewer's session-only override on someone else's profile: the same controls, tagged "what-if". */
export const ViewerWhatIf = () => <Stage saved={false} slots={3} trainingClass="Guard" />;
