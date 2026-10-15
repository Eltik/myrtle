import { ArtBrowser } from "frontend";
import { useState } from "react";

// ArtBrowser is the background editor's lower half. From md up: a source rail
// (Character art; then the Gallery sources Archives, Story CGs and Scenes with
// counts; then category chips and the story panel) beside the results. Below
// md the rail becomes a sticky filter bar. Its gallery and entity catalogues
// load through server functions the design bundle stubs to fail, so the
// honest renders here are the load-failure states: the character picker's,
// and the gallery's "Couldn't load the gallery" with Retry (the rail shows no
// counts and no filters until a catalogue loads).

type Picked = { kind: "skin" | "operator" | "archive_pic" | "story_cg" | "story_scene"; id: string } | null;

function Stage({ initial }: { initial: Picked }) {
    const [selected, setSelected] = useState<Picked>(initial);
    return (
        <div style={{ width: 880, minHeight: 560 }} className="bg-background">
            <ArtBrowser selected={selected as never} onPick={(kind, id) => setSelected({ kind, id })} />
        </div>
    );
}

/** Opens on Character art when the draft is an operator or outfit. */
export const CharacterSource = () => <Stage initial={{ kind: "operator", id: "char_291_aglina" }} />;

/** Opens on Story CGs when the draft is a story CG: the gallery's load-failure line. */
export const GallerySourceLoadFailed = () => <Stage initial={{ kind: "story_cg", id: "47_i01" }} />;
