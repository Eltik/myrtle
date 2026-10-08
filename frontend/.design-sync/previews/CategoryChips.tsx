import { CategoryChips } from "frontend";
import { useState } from "react";

// CategoryChips is the gallery's category filter: one wrapping row of toggle
// pills, each with its muted facet count (what turning it on would add).
// Several can be on; an "on" chip is tinted and checked. A category the other
// filters leave empty dims to half opacity. ArtBrowser shows it in the rail
// (w-64) on wide screens and under the search on narrow ones. Counts are the
// live EN catalogues (`/api/story/art-gallery/scene`, `/api/story/gallery`).

type Category = "main" | "side" | "vignette" | "is" | "reclamation" | "sideContent" | "record";
type Option = { id: Category; count: number };

/** Scenes, unfiltered: 916 plates across four categories. */
const SCENES: readonly Option[] = [
    { id: "main", count: 298 },
    { id: "side", count: 591 },
    { id: "vignette", count: 17 },
    { id: "record", count: 10 },
];

function Stateful({ initial, options, width = 256 }: { initial: Category[]; options: readonly Option[]; width?: number }) {
    const [chosen, setChosen] = useState<Category[]>(initial);
    return (
        <div style={{ width }}>
            <CategoryChips options={options} chosen={chosen} onToggle={(id) => setChosen((c) => (c.includes(id) ? c.filter((x) => x !== id) : [...c, id]))} />
        </div>
    );
}

/** Scenes, nothing chosen: every category counted. */
export const NoneChosen = () => <Stateful initial={[]} options={SCENES} />;

/** Two categories on: tinted, checked. */
export const TwoChosen = () => <Stateful initial={["main", "side"]} options={SCENES} />;

/** A search for "Kazimierz" narrows the counts; the categories it empties dim. */
export const SearchNarrowed = () => (
    <Stateful
        initial={[]}
        options={[
            { id: "main", count: 0 },
            { id: "side", count: 24 },
            { id: "vignette", count: 0 },
            { id: "record", count: 2 },
        ]}
    />
);

/** Archives across a wide filter bar: Integrated Strategies themes and side-story archives. */
export const WideArchives = () => (
    <Stateful
        initial={["is"]}
        width={640}
        options={[
            { id: "side", count: 57 },
            { id: "is", count: 267 },
        ]}
    />
);
