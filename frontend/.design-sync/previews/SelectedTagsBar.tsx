import { SelectedTagsBar } from "frontend";

// The summary strip above the recruitment results: each chosen tag is a
// removable chip, followed by the combination count and a reset. The component
// returns `null` when nothing is selected, so every story has at least one tag.
// Tag ids and names are the real `gachaTags` rows from /api/static/gacha.

const noop = () => {};

const tag = (id: number, name: string, type: string) => ({ id, name, type });

export const SingleTag = () => <SelectedTagsBar onRemove={noop} onReset={noop} resultCount={1} selectedTags={[tag(28, "Robot", "qualification")]} />;

export const TwoTags = () => <SelectedTagsBar onRemove={noop} onReset={noop} resultCount={3} selectedTags={[tag(3, "Defender", "class"), tag(22, "Defense", "affix")]} />;

// The in-game cap: five tags, which expands to 31 combinations.
export const MaxTags = () => <SelectedTagsBar onRemove={noop} onReset={noop} resultCount={31} selectedTags={[tag(11, "Top Operator", "qualification"), tag(14, "Senior Operator", "qualification"), tag(2, "Sniper", "class"), tag(10, "Ranged", "position"), tag(19, "DPS", "affix")]} />;
