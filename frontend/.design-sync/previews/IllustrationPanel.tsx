import { useEffect } from "react";
import { IllustrationPanel } from "frontend";
import type { IArtGroup } from "../../src/components/story/library/impl/illustrations";

// IllustrationPanel is the Illustrations tab's per-chapter dialog (a bottom
// sheet on a phone): the chapter's key visual and name, panes for the
// backgrounds, CGs and character sprites with their counts, and a grid of the
// pieces that opens each one full size. It fetches the chapter's art itself;
// the design bundle has no backend, so it renders its real load-failure
// branch inside the open dialog.

const noop = () => undefined;

/** Clears the modal's auto-focus ring once the open transition has placed it (60/180/400 ms, see NOTES). */
function useBlurAfterOpen() {
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => timers.forEach(clearTimeout);
    }, []);
}

const SHATTERPOINT: IArtGroup = {
    id: "main_10",
    name: "Shatterpoint",
    art: { kind: "banner", url: "/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png" },
    code: "EP10",
    kind: "main",
    chapter: 10,
    illustrationCount: 58,
    spriteCount: 66,
} as IArtGroup;

/** Opened from the Illustrations card: the header, the panes, and the failed load. */
export const Illustrations = () => {
    useBlurAfterOpen();
    return (
        <div style={{ minHeight: 560 }}>
            <IllustrationPanel group={SHATTERPOINT} kind="illustrations" onClose={noop} />
        </div>
    );
};

/** Opened on the sprites pane. */
export const Sprites = () => {
    useBlurAfterOpen();
    return (
        <div style={{ minHeight: 560 }}>
            <IllustrationPanel group={SHATTERPOINT} kind="sprites" onClose={noop} />
        </div>
    );
};
