import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { storyArtGalleryQueryOptions, storyGalleryQueryOptions } from "#/lib/api/story";
import type { GalleryKind } from "../background";
import { archiveTiles, type IGalleryTile, storyArtTiles } from "../gallery";

/** One gallery source's catalogue: its tiles once loaded, and how its load went. */
export interface IGallerySource {
    tiles: readonly IGalleryTile[];
    status: "pending" | "error" | "success";
    retry: () => void;
}

/**
 * The three gallery catalogues (the Archives gallery, the story CGs, the story scene
 * plates) flattened to tiles. All three load as soon as the editor opens, so the source
 * rows can print their counts. The catalogues are the default server's: the picture
 * routes fall back to it from any other, and the header asks for it the same way.
 */
export function useGalleryCatalog(): Record<GalleryKind, IGallerySource> {
    const archive = useQuery(storyGalleryQueryOptions());
    const cgs = useQuery(storyArtGalleryQueryOptions("cg"));
    const scenes = useQuery(storyArtGalleryQueryOptions("scene"));
    const archiveList = useMemo(() => (archive.data ? archiveTiles(archive.data) : []), [archive.data]);
    const cgList = useMemo(() => (cgs.data ? storyArtTiles(cgs.data) : []), [cgs.data]);
    const sceneList = useMemo(() => (scenes.data ? storyArtTiles(scenes.data) : []), [scenes.data]);
    // A query that answered with no catalogue (`null`) is an error to the picker, as a failed one is.
    const status = (q: { status: "pending" | "error" | "success"; data: unknown }) => (q.status === "success" && q.data == null ? "error" : q.status);
    return {
        archive_pic: { tiles: archiveList, status: status(archive), retry: () => void archive.refetch() },
        story_cg: { tiles: cgList, status: status(cgs), retry: () => void cgs.refetch() },
        story_scene: { tiles: sceneList, status: status(scenes), retry: () => void scenes.refetch() },
    };
}
