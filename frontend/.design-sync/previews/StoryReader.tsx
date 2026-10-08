import { StoryReader } from "frontend";
import { useEffect } from "react";
import script from "../../src/lib/story/__fixtures__/main_15_level_main_15-08_end.json";

// The whole reader for one story: the stage (scene, sprites, overlays), the
// text box, the two toolbar pills, the progress hairline, and the title and
// end cards, driven by the script engine. The script is the repo's own
// fixture of 15-8 "Decompression Syndrome" (Hortus de Escapismo), the same
// payload the route hands it. The reader sizes itself to the viewport minus
// the site header. Reading progress and settings live in localStorage, so each
// story clears them during render to start from the same place.

type Props = Parameters<typeof StoryReader>[0];

const ENTRY = { id: script.id, name: "Decompression Syndrome", code: "15-8", sort: 15, avgTag: "After Operation", groupId: "main_15", hasScript: true, wordCount: script.wordCount, hasVideo: false, requiredStages: [] } as unknown as Props["entry"];
const PREV = { ...ENTRY, id: "main_15_level_main_15-08_beg", avgTag: "Before Operation" } as Props["entry"];

function fresh() {
    try {
        window.localStorage.removeItem("myrtle.story.progress");
        window.localStorage.removeItem("myrtle.story.settings");
    } catch {
        // Storage blocked: the reader starts fresh anyway.
    }
}

const Reader = (over: Partial<Props>) => {
    fresh();
    return <StoryReader script={script as unknown as Props["script"]} entry={ENTRY} groupName="Hortus de Escapismo" category="main" previous={PREV} next={null as unknown as Props["next"]} {...over} />;
};

// Opened from the library: the title card over the black stage.
export const TitleCard = () => <Reader />;

// Mid-scene: Amiya speaking, toolbar and progress up. A click on the stage
// during the typewriter completes the line, the way a reader would, so the
// card shows the whole line rather than wherever the reveal had reached.
function CompleteLine() {
    useEffect(() => {
        const id = window.setTimeout(() => document.querySelector<HTMLElement>("[data-story-stage]")?.click(), 300);
        return () => window.clearTimeout(id);
    }, []);
    return null;
}
export const MidScene = () => (
    <>
        <Reader initialLine={49} />
        <CompleteLine />
    </>
);

// A choice halt: the decision options over the dimmed scene.
export const Decision = () => <Reader initialLine={46} />;
