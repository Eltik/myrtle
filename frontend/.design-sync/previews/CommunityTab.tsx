import { CommunityTab } from "frontend";
import type { LibIndex } from "../../src/components/story/library/impl/derive";

// CommunityTab ranks the Archives by what Myrtle's synced players have read
// (most-read chapters and operator records, with the player count and when it
// was computed). It fetches the community figures itself; the design bundle
// has no backend, so it renders its real "no community data yet" branch.

const INDEX: LibIndex = { groups: [], records: [] };
const noop = () => undefined;

/** No community figures to rank: the empty card. */
export const Unavailable = () => (
    <div style={{ width: 860 }}>
        <CommunityTab index={INDEX} onViewChapter={noop} />
    </div>
);
