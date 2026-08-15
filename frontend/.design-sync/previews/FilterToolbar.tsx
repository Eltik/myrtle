import { FilterToolbar } from "frontend";

const FLAIRS = [
    { code: "meta", label: "Meta", color: "#5aa9d9", count: 34, displayOrder: 1 },
    { code: "cc", label: "Contingency Contract", color: "#e0834a", count: 18, displayOrder: 2 },
    { code: "is", label: "Integrated Strategies", color: "#9b73d4", count: 22, displayOrder: 3 },
    { code: "beginner", label: "Beginner", color: "#d8b54a", count: 11, displayOrder: 4 },
    { code: "class", label: "Class guide", color: "#5dbf86", count: 27, displayOrder: 5 },
    { code: "event", label: "Event", color: "#c069b4", count: 0, displayOrder: 6 },
];

const noop = () => {};

const handlers = {
    onTypeChange: noop,
    onSortChange: noop,
    onQueryChange: noop,
    onFlairToggle: noop,
    onClearFlairs: noop,
};

export const Default = () => <FilterToolbar type="all" sort="recent" query="" selectedFlairs={[]} flairOptions={FLAIRS} resultCount={182} totalCount={182} showFavoritesTab={false} {...handlers} />;

export const FiltersActive = () => <FilterToolbar type="community" sort="trending" query="risk 18" selectedFlairs={["cc", "meta"]} flairOptions={FLAIRS} resultCount={7} totalCount={182} showFavoritesTab={false} {...handlers} />;

export const SignedInWithFavorites = () => <FilterToolbar type="favorites" sort="favorites" query="" selectedFlairs={[]} flairOptions={FLAIRS} resultCount={9} totalCount={182} showFavoritesTab {...handlers} />;

export const NoFlairsPublished = () => <FilterToolbar type="official" sort="views" query="" selectedFlairs={[]} flairOptions={[]} resultCount={26} totalCount={182} showFavoritesTab={false} {...handlers} />;
