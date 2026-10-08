import { SearchField } from "frontend";
import { useState } from "react";

// SearchField is the background editor's search box: a magnifier addon and a
// type="search" input. The gallery's search bar and both story searches
// (the rail's StoryPanel, the narrow StoryPicker) are this one component.

function Stateful({ initial, placeholder, label, className }: { initial: string; placeholder: string; label: string; className?: string }) {
    const [value, setValue] = useState(initial);
    return <SearchField value={value} onChange={setValue} placeholder={placeholder} label={label} className={className} />;
}

/** The gallery search, empty: the placeholder reads. */
export const GalleryEmpty = () => (
    <div className="w-96">
        <Stateful initial="" placeholder="Search gallery pictures" label="Search gallery pictures by title, event or story" />
    </div>
);

/** A query typed. */
export const GalleryQuery = () => (
    <div className="w-96">
        <Stateful initial="Kazimierz" placeholder="Search gallery pictures" label="Search gallery pictures by title, event or story" />
    </div>
);

/** The compact story search the rail's panel uses (h-8). */
export const StorySearchCompact = () => (
    <div className="w-60 rounded-lg border bg-card p-1.5">
        <Stateful initial="" placeholder="Find a story" label="Find a story by name" className="h-8" />
    </div>
);
