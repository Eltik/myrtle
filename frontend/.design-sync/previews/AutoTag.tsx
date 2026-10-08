import { AutoTag } from "frontend";

// Marks a name the planner filled in itself because EN has no official one yet.
// The three sources: a name EN used before (memory), an operator's own
// appellation, or a hand-maintained override. Each carries an explanatory title.
export const Sources = () => (
    <div className="flex flex-col gap-2 p-4 font-sans text-[13px] text-foreground">
        <span className="flex items-center gap-2">
            Cantilena Puppae <AutoTag source="memory" />
        </span>
        <span className="flex items-center gap-2">
            Yukari Takeba <AutoTag source="appellation" />
        </span>
        <span className="flex items-center gap-2">
            Critical Phase Transition <AutoTag source="override" />
        </span>
    </div>
);

export const Memory = () => (
    <div className="p-4">
        <AutoTag source="memory" />
    </div>
);
