import { FlatHead } from "frontend";

// FlatHead is the heading over the single flat list a sort produces in the
// Archives (any order other than "By storyline"): a title and a count over a
// thin rule, with no section glyph and no chapter range, because the list is
// not a shelf.

/** The sorted list's heading, as Browse prints it. */
export const SortedList = () => (
    <div style={{ width: 760 }}>
        <FlatHead title="All chapters" count="75 chapters" />
    </div>
);

/** Narrowed by a filter: the same heading with the filtered count. */
export const Filtered = () => (
    <div style={{ width: 760 }}>
        <FlatHead title="All chapters" count="17 chapters" />
    </div>
);
