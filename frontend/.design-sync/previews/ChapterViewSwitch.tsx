import { useState } from "react";
import { ChapterViewSwitch } from "frontend";
import type { ChapterView } from "../../src/components/story/library/impl/ArchivePanel";
import type { StoryArchiveSection } from "../../src/types/generated/StoryArchiveSection";

// ChapterViewSwitch is the chapter sheet's Entries / Archive toggle under the
// meta row. It renders only for a chapter that HAS an archive (about one in
// twenty EN chapters), so most sheets never show it.

const SECTIONS = [{ kind: "music", count: 4, tracks: [] }] as unknown as StoryArchiveSection[];

/** Entries selected, as a sheet opens. */
export const EntriesSelected = () => {
    const [view, setView] = useState<ChapterView>("entries");
    return (
        <div className="bg-popover pt-3" style={{ width: 520 }}>
            <ChapterViewSwitch view={view} onView={setView} sections={SECTIONS} />
        </div>
    );
};

/** Archive selected. */
export const ArchiveSelected = () => {
    const [view, setView] = useState<ChapterView>("archive");
    return (
        <div className="bg-popover pt-3" style={{ width: 520 }}>
            <ChapterViewSwitch view={view} onView={setView} sections={SECTIONS} />
        </div>
    );
};
