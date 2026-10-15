import { AutoTranslateProvider, CnName, OpRef } from "frontend";

// Context switch for the release planner's auto-translated names (the user's
// "Latin names" setting). On: names EN has not released yet show the planner's
// translation with its source tag. Off: they stay in Chinese.
const LOOKUP = new Map();

function Rows() {
    return (
        <div className="flex flex-col gap-3">
            <CnName cn="人偶的歌谣" auto={{ text: "Cantilena Puppae", source: "memory" }} primaryClassName="font-sans font-semibold text-[14px] text-foreground" />
            <CnName cn="相变临界" auto={{ text: "Critical Phase Transition", source: "appellation" }} primaryClassName="font-sans font-semibold text-[14px] text-foreground" />
            <span>
                <OpRef id="char_4219_yukari" lookup={LOOKUP} name={{ text: "Yukari Takeba", source: "appellation" }} />
            </span>
        </div>
    );
}

export const On = () => (
    <div className="w-full max-w-md p-4">
        <AutoTranslateProvider value={true}>
            <Rows />
        </AutoTranslateProvider>
    </div>
);

export const Off = () => (
    <div className="w-full max-w-md p-4">
        <AutoTranslateProvider value={false}>
            <Rows />
        </AutoTranslateProvider>
    </div>
);
