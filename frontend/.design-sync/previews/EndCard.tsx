import { EndCard } from "frontend";
import type { ReactNode } from "react";

// The sheet after the last line: "End of story", its neighbours in chapter
// order, a replay and the way back to the library. It is `absolute inset-0`
// at 85% black, so it sits over the last scene in the reader's 16:9 box.

const BG = "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_bg_ce_0/bg_ceo.png";

function StageBox({ children }: { children: ReactNode }) {
    return (
        <div className="relative aspect-video w-full overflow-hidden rounded-lg bg-black">
            <img src={BG} alt="" className="absolute inset-0 size-full object-cover" />
            {children}
        </div>
    );
}

const noop = () => {};
// Only `id` is read off a neighbour (it builds the `/stories/$storyId` link).
type Entry = Parameters<typeof EndCard>[0]["previous"];
const prev = { id: "main_15_level_main_15-08_beg", name: "Decompression Syndrome" } as unknown as Entry;
const next = { id: "main_15_level_main_15-09_beg", name: "Reboot" } as unknown as Entry;

// Mid-chapter: both neighbours exist, so Previous / Read again / Next.
export const MidChapter = () => (
    <StageBox>
        <EndCard title="Decompression Syndrome" previous={prev} next={next} onRestart={noop} />
    </StageBox>
);

// The chapter's first story: no previous neighbour, the row starts at Read again.
export const ChapterStart = () => (
    <StageBox>
        <EndCard title="Decompression Syndrome" previous={null as unknown as Entry} next={next} onRestart={noop} />
    </StageBox>
);

// The last story of the chapter: no Next, only Previous and Read again.
export const ChapterEnd = () => (
    <StageBox>
        <EndCard title="Decompression Syndrome" previous={prev} next={null as unknown as Entry} onRestart={noop} />
    </StageBox>
);
