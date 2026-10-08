import { ExportSheet } from "frontend";
import { type ReactNode, useEffect } from "react";

// The story reader's export panel: scope (this story, this chapter, a range),
// format (EPUB / PDF / Markdown / Text), images, branches, typeface and paper,
// then a run with progress. The chapter comes in as `group` (the live
// `/api/story/index` entry for Evil Time Part 1); the wider library loads
// through a query, stubbed in the design bundle, so ranges past this chapter
// are unavailable.

const EVIL_TIME_1 = {"id":"main_0","name":"Evil Time Part 1","coverUrl":"/textures/avg/bg/avg_bkg_h1_bg_in_0/bg_indoor_1.png","bannerUrl":"/textures/spritepack/mixstory_kv_sprites_0/kv_evil_time_part1.png","titleImageUrl":"/textures/spritepack/mixstory_title_sprites_0/title_evil_time_part1.png","illustrationCount":17,"stories":[{"id":"main_0_0_welcome_to_guide","name":"Prologue, Part 1","sort":1,"avgTag":"Interlude","hasScript":true,"wordCount":768},{"id":"main_0_2_guide_to_home","name":"Prologue, Part 2","sort":2,"avgTag":"Interlude","hasScript":true,"wordCount":637},{"id":"main_0_level_main_00-01_beg","name":"Collapse","code":"0-1","sort":3,"avgTag":"Before Operation","hasScript":true,"wordCount":416},{"id":"main_0_level_main_00-01_end","name":"Collapse","code":"0-1","sort":4,"avgTag":"After Operation","hasScript":true,"wordCount":651},{"id":"main_0_level_main_00-02_beg","name":"Protection","code":"0-2","sort":5,"avgTag":"Before Operation","hasScript":true,"wordCount":523},{"id":"main_0_level_main_00-02_end","name":"Protection","code":"0-2","sort":6,"avgTag":"After Operation","hasScript":true,"wordCount":310},{"id":"main_0_level_main_00-04_beg","name":"Brawl","code":"0-4","sort":7,"avgTag":"Before Operation","hasScript":true,"wordCount":398},{"id":"main_0_level_main_00-06_end","name":"Strike","code":"0-6","sort":8,"avgTag":"After Operation","hasScript":true,"wordCount":206},{"id":"main_0_level_main_00-07_beg","name":"Infection","code":"0-7","sort":9,"avgTag":"Before Operation","hasScript":true,"wordCount":616},{"id":"main_0_level_main_00-07_end","name":"Infection","code":"0-7","sort":10,"avgTag":"After Operation","hasScript":true,"wordCount":252},{"id":"main_0_level_main_00-08_beg","name":"Hunting","code":"0-8","sort":11,"avgTag":"Before Operation","hasScript":true,"wordCount":666},{"id":"main_0_level_main_00-09_end","name":"Nearl","code":"0-9","sort":12,"avgTag":"After Operation","hasScript":true,"wordCount":370},{"id":"main_0_level_main_00-10_beg","name":"Dilemma","code":"0-10","sort":13,"avgTag":"Before Operation","hasScript":true,"wordCount":409},{"id":"main_0_level_main_00-11_end","name":"Blockade Running","code":"0-11","sort":14,"avgTag":"After Operation","hasScript":true,"wordCount":650}]};

const noop = () => {};

/** Full-viewport stage; blurs the popup's auto-focused control after the open transition. */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return <div className="min-h-dvh">{children}</div>;
};

/** Opened from inside a story: "This story" offered and selected. */
export const FromAStory = () => (
    <Stage>
        <ExportSheet open onOpenChange={noop} group={EVIL_TIME_1} storyId="main_0_level_main_00-01_beg" />
    </Stage>
);

/** Opened from the chapter page: the chapter is the default scope. */
export const FromTheChapter = () => (
    <Stage>
        <ExportSheet open onOpenChange={noop} group={EVIL_TIME_1} />
    </Stage>
);
