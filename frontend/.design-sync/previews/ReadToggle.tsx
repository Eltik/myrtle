import type React from "react";
import { ReadToggle } from "frontend";
import type { StoryProgress } from "../../src/lib/story/progress";

// ReadToggle is the tick beside every story: a control, not a verdict. An
// empty ring is unread; a primary check is read. A check with a small
// controller glyph is the Arknights client's own verdict (synced from the
// game, never written into the reader's document); a DASHED ring is a story
// the game says was played that the reader marked unread by hand. Each source
// carries its wording in a tooltip. Sizes: sm 32 px, md 36 px, lg 40 px on a
// fine pointer (all 44 px under 640).

const STORY = { id: "main_10_level_main_10-04_beg", name: "Breath of the City" };
const NONE: StoryProgress = { v: 2, read: {}, pos: {} };
const OWN: StoryProgress = { v: 2, read: { [STORY.id]: 1715600000000 }, pos: {} };
const CLEARED: StoryProgress = { v: 2, read: {}, unread: { [STORY.id]: 1715600000000 }, pos: {} };
const GAME = new Set([STORY.id]);
const NO_GAME = new Set<string>();

const Cell = ({ label, children }: { label: string; children: React.ReactNode }) => (
    <div className="flex flex-col items-center gap-1.5">
        {children}
        <span className="font-sans text-[11.5px] text-muted-foreground">{label}</span>
    </div>
);

/** The four read sources side by side: unread, read here, read in the game, and the game overridden. */
export const Sources = () => (
    <div className="flex items-start gap-8">
        <Cell label="Unread">
            <ReadToggle story={STORY} progress={NONE} gameRead={NO_GAME} />
        </Cell>
        <Cell label="Read here">
            <ReadToggle story={STORY} progress={OWN} gameRead={NO_GAME} />
        </Cell>
        <Cell label="Read in game">
            <ReadToggle story={STORY} progress={NONE} gameRead={GAME} />
        </Cell>
        <Cell label="Marked unread">
            <ReadToggle story={STORY} progress={CLEARED} gameRead={GAME} />
        </Cell>
    </div>
);

/** The three sizes, read: a list row's sm, md, and the chapter sheet's lg. */
export const Sizes = () => (
    <div className="flex items-end gap-8">
        <Cell label="sm">
            <ReadToggle story={STORY} progress={OWN} gameRead={NO_GAME} size="sm" className="border border-border" />
        </Cell>
        <Cell label="md">
            <ReadToggle story={STORY} progress={OWN} gameRead={NO_GAME} size="md" className="border border-border" />
        </Cell>
        <Cell label="lg">
            <ReadToggle story={STORY} progress={OWN} gameRead={NO_GAME} size="lg" className="border border-border" />
        </Cell>
    </div>
);
