import { RailLabel, StoryPanel } from "frontend";
import { useState } from "react";

// StoryPanel is the art browser rail's story filter: a bordered panel with its
// own compact search on an opaque bar at the top (outside the scroll), then
// "All stories" with the total while the search is blank, then each story
// with its muted count. The list scrolls inside the panel and fades at both
// edges; it fills whatever height the rail leaves. Choosing the chosen story
// again clears it. Fixture: the live EN Story CG catalogue
// (`/api/story/art-gallery/cg`): 80 stories, 1,256 pictures.

const GROUPS = [
    {id: "main_0", name: "Evil Time Part 1", category: "main", count: 11},
    {id: "main_1", name: "Evil Time Part 2", category: "main", count: 7},
    {id: "main_2", name: "Separated Hearts", category: "main", count: 2},
    {id: "main_3", name: "Stinging Shock", category: "main", count: 7},
    {id: "main_4", name: "Burning Run", category: "main", count: 10},
    {id: "main_5", name: "Necessary Solutions", category: "main", count: 19},
    {id: "main_6", name: "Partial Necrosis", category: "main", count: 20},
    {id: "main_7", name: "The Birth of Tragedy", category: "main", count: 22},
    {id: "main_8", name: "Roaring Flare", category: "main", count: 47},
    {id: "main_9", name: "Stormwatch", category: "main", count: 13},
    {id: "main_10", name: "Shatterpoint", category: "main", count: 28},
    {id: "main_11", name: "Return To Mist", category: "main", count: 22},
    {id: "main_12", name: "All Quiet Under the Thunder", category: "main", count: 15},
    {id: "main_13", name: "The Whirlpool that is Passion", category: "main", count: 18},
    {id: "main_14", name: "Absolved Will Be the Seekers", category: "main", count: 46},
    {id: "main_15", name: "Dissociative Recombination", category: "main", count: 45},
    {id: "main_16", name: "Abnormal Spectrum", category: "main", count: 26},
    {id: "1stact", name: "Grani and the Knights' Treasure", category: "side", count: 8},
    {id: "act3d0", name: "Heart of Surging Flame", category: "side", count: 6},
    {id: "act5d0", name: "Code of Brawl", category: "side", count: 11},
    {id: "act11d0", name: "Twilight of Wolumonde", category: "side", count: 5},
    {id: "act9d0", name: "Darknights Memoir", category: "side", count: 6},
    {id: "act12d0", name: "The Great Chief Returns", category: "side", count: 10},
    {id: "act13d5", name: "Maria Nearl", category: "side", count: 8},
    {id: "act15d0", name: "Mansfield Break", category: "side", count: 17},
    {id: "act16d5", name: "Who Is Real", category: "side", count: 7},
    {id: "act17d0", name: "Operation Originium Dust", category: "side", count: 14},
    {id: "act18d0", name: "A Walk in the Dust", category: "side", count: 5},
    {id: "act18d3", name: "Under Tides", category: "side", count: 19},
    {id: "act12side", name: "Dossoles Holiday", category: "side", count: 10},
    {id: "act13side", name: "Near Light", category: "side", count: 16},
    {id: "act14side", name: "Break the Ice", category: "side", count: 14},
    {id: "act15side", name: "Invitation to Wine", category: "side", count: 8},
    {id: "act16side", name: "Guide Ahead", category: "side", count: 13},
    {id: "act17side", name: "Stultifera Navis", category: "side", count: 16},
    {id: "act18side", name: "Lingering Echoes", category: "side", count: 12},
    {id: "act20side", name: "Ideal City: Endless Carnival", category: "side", count: 13},
    {id: "act19side", name: "Dorothy's Vision", category: "side", count: 12},
    {id: "act21side", name: "IL Siracusano", category: "side", count: 18},
    {id: "act22side", name: "What the Firelight Casts", category: "side", count: 12},
    {id: "act23side", name: "Where Vernal Winds Will Never Blow", category: "side", count: 16},
    {id: "act25side", name: "Lone Trail", category: "side", count: 17},
    {id: "act26side", name: "Hortus de Escapismo", category: "side", count: 15},
    {id: "act27side", name: "So Long, Adele: Home Away From Home", category: "side", count: 16},
    {id: "act28side", name: "Come Catastrophes or Wakes of Vultures", category: "side", count: 14},
    {id: "act29side", name: "Zwillingstürme im Herbst", category: "side", count: 31},
    {id: "act30side", name: "The Rides to Lake Silberneherze", category: "side", count: 12},
    {id: "act31side", name: "Here A People Sows", category: "side", count: 17},
    {id: "act32side", name: "Operation Lucent Arrowhead", category: "side", count: 11},
    {id: "act33side", name: "Babel", category: "side", count: 21},
    {id: "act34side", name: "Path of Life", category: "side", count: 17},
    {id: "act35side", name: "Adventure That Cannot Wait for the Sun", category: "side", count: 26},
    {id: "act36side", name: "Delicious On Terra", category: "side", count: 19},
    {id: "act37side", name: "Ending a Grand Overture", category: "side", count: 29},
    {id: "act38side", name: "I Portatori dei Velluti", category: "side", count: 39},
    {id: "act39side", name: "Exodus from the Pale Sea", category: "side", count: 21},
    {id: "act40side", name: "Such is the Joy of Our Reunion", category: "side", count: 25},
    {id: "act41side", name: "When Elegies Are Ashes", category: "side", count: 13},
    {id: "act42side", name: "The Masses' Travels", category: "side", count: 40},
    {id: "act43side", name: "Act or Die", category: "side", count: 24},
    {id: "act44side", name: "Ato", category: "side", count: 25},
    {id: "act45side", name: "Somniloquium Serenum", category: "side", count: 18},
    {id: "act46side", name: "Retracing Our Steps", category: "side", count: 25},
    {id: "act47side", name: "Unrealized Realities", category: "side", count: 28},
    {id: "act48side", name: "Medjehtiqedti Bound", category: "side", count: 15},
    {id: "act49side", name: "First of A Thousand Autumns", category: "side", count: 31},
    {id: "act51side", name: "People, A People", category: "side", count: 30},
    {id: "act4d0", name: "Operational Intelligence", category: "vignette", count: 1},
    {id: "act6d5", name: "Ancient Forge", category: "vignette", count: 4},
    {id: "act13d0", name: "Rewinding Breeze", category: "vignette", count: 2},
    {id: "act8mini", name: "Vigilo", category: "vignette", count: 1},
    {id: "act10mini", name: "A Light Spark in Darkness", category: "vignette", count: 1},
    {id: "act12mini", name: "An Obscure Wanderer", category: "vignette", count: 3},
    {id: "act14mini", name: "A Death in Chunfen", category: "vignette", count: 3},
    {id: "act15mini", name: "The Black Forest Wills A Dream", category: "vignette", count: 6},
    {id: "act16mini", name: "To the Grinning Valley", category: "vignette", count: 3},
    {id: "act17mini", name: "A Kazdelian Rescue", category: "vignette", count: 3},
    {id: "act18mini", name: "See You Soon", category: "vignette", count: 8},
    {id: "act19mini", name: "Fantasy in the Mirage", category: "vignette", count: 6},
    {id: "act20mini", name: "Crossing", category: "vignette", count: 2},
] as const;

function Rail({ initial }: { initial: string | null }) {
    const [chosen, setChosen] = useState<string | null>(initial);
    return (
        <div style={{ height: 420 }} className="flex w-64 flex-col">
            <RailLabel>Story</RailLabel>
            <StoryPanel groups={GROUPS} chosen={chosen} total={1256} onChoose={setChosen} />
        </div>
    );
}

/** Nothing chosen: "All stories" tinted, carrying the total. */
export const AllStories = () => <Rail initial={null} />;

/** One story chosen: its row tinted. */
export const StoryChosen = () => <Rail initial="main_8" />;

/** A catalogue with no stories to list (a source with no matching tiles). */
export const Empty = () => (
    <div style={{ height: 260 }} className="flex w-64 flex-col">
        <RailLabel>Story</RailLabel>
        <StoryPanel groups={[]} chosen={null} total={0} onChoose={() => undefined} />
    </div>
);
