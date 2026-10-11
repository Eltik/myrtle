import { createContext, useContext } from "react";
import type { LibrarySelection } from "./route";

/**
 * How a view deep in the page moves the page: open an album, show a Library
 * view. The page provides it; the URL is what actually changes.
 */
export interface IMusicNav {
    /** Open the album modal for an album id (`siren:<cid>`). */
    openAlbum: (albumId: string) => void;
    /** Switch to the Library on a view, a playlist or an artist. */
    openLibrary: (selection: LibrarySelection) => void;
}

const NOOP: IMusicNav = { openAlbum: () => {}, openLibrary: () => {} };

export const MusicNavContext = createContext<IMusicNav>(NOOP);

export function useMusicNav(): IMusicNav {
    return useContext(MusicNavContext);
}
