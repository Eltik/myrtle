import { RailLabel, RailRow } from "frontend";
import { BookImageIcon, ClapperboardIcon, MountainIcon, UserRoundIcon } from "lucide-react";
import { useState } from "react";

// RailRow is one row of the background editor's source rail (the left column
// of the art browser at md and up): a lucide icon, the source's name and, for
// gallery sources, a muted monospaced count of the pictures it holds. The
// chosen row is tinted with the primary; the rest stay muted until hovered.
// ArtBrowser lays them out as below: "Character art" alone, a hairline, then
// the "Gallery" label over the three gallery sources.

type Source = "characters" | "archive_pic" | "story_cg" | "story_scene";

function SourceRail({ initial }: { initial: Source }) {
    const [source, setSource] = useState<Source>(initial);
    return (
        <aside className="flex w-64 flex-col gap-0.5 border-e bg-background px-3 py-4">
            <RailRow active={source === "characters"} onClick={() => setSource("characters")} name="Character art" icon={<UserRoundIcon aria-hidden="true" />} />
            <div className="my-1.5 border-t" />
            <RailLabel>Gallery</RailLabel>
            <RailRow active={source === "archive_pic"} onClick={() => setSource("archive_pic")} name="Archives" icon={<BookImageIcon aria-hidden="true" />} count={324} />
            <RailRow active={source === "story_cg"} onClick={() => setSource("story_cg")} name="Story CGs" icon={<ClapperboardIcon aria-hidden="true" />} count={1256} />
            <RailRow active={source === "story_scene"} onClick={() => setSource("story_scene")} name="Scenes" icon={<MountainIcon aria-hidden="true" />} count={916} />
        </aside>
    );
}

/** The source rail as the editor opens it: character art chosen. */
export const SourceRailCharacters = () => <SourceRail initial="characters" />;

/** A gallery source chosen: its row tinted, its count kept. */
export const SourceRailStoryCgs = () => <SourceRail initial="story_cg" />;

/** One row each way, side by side: active, idle with a count, idle without one. */
export const States = () => (
    <div className="flex w-64 flex-col gap-0.5">
        <RailRow active onClick={() => undefined} name="Story CGs" icon={<ClapperboardIcon aria-hidden="true" />} count={1256} />
        <RailRow active={false} onClick={() => undefined} name="Scenes" icon={<MountainIcon aria-hidden="true" />} count={916} />
        <RailRow active={false} onClick={() => undefined} name="Character art" icon={<UserRoundIcon aria-hidden="true" />} />
    </div>
);

/** While the gallery catalogue loads, the rows carry no count yet. */
export const CountsLoading = () => (
    <div className="flex w-64 flex-col gap-0.5">
        <RailLabel>Gallery</RailLabel>
        <RailRow active={false} onClick={() => undefined} name="Archives" icon={<BookImageIcon aria-hidden="true" />} count={null} />
        <RailRow active={false} onClick={() => undefined} name="Story CGs" icon={<ClapperboardIcon aria-hidden="true" />} count={null} />
        <RailRow active={false} onClick={() => undefined} name="Scenes" icon={<MountainIcon aria-hidden="true" />} count={null} />
    </div>
);
