import { ReleaseNotesList } from "frontend";

// The "Release notes" surface of /changelog: the curated archive, every entry
// from `src/content/changelog/entries.ts` rendered in full, newest first,
// including the ones filed with `announce: false` that never lit the bell.
// It takes no props and never fetches - the content is compiled in - so the
// one story is the archive as shipped. The dashed "No release notes written
// yet" empty state is unreachable while the entries file is non-empty.
export const Archive = () => (
    <div className="max-w-2xl">
        <ReleaseNotesList />
    </div>
);
