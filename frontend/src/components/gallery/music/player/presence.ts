import { useStore } from "@tanstack/react-store";
import { Store } from "@tanstack/store";

/**
 * WHETHER THE SITE HAS MUSIC, without the player.
 *
 * The root layout reads this to decide whether to load the player dock at
 * all, so it must not import `./engine`: a visitor who never opens the music
 * page never downloads the engine. The engine writes it (`setMusicLive`) when
 * a track is loaded AND has been asked to sound in this page load, so a
 * session restored paused on the music page and left untouched does not
 * follow the visitor around the site.
 *
 * It also carries the calls that keep the site to one music at a time: a
 * story's own music pauses the player rather than playing over it, and the
 * player starting pauses (or stops) whatever story music registered here.
 */
const presence = new Store<boolean>(false);

let pauser: (() => void) | null = null;

export function setMusicLive(live: boolean): void {
    if (presence.state !== live) presence.setState(() => live);
}

export function useMusicLive(): boolean {
    return useStore(presence, (s) => s);
}

/** The engine's pause, registered when the engine module loads. */
export function registerMusicPauser(fn: (() => void) | null): void {
    pauser = fn;
}

/** Pause the music player if it is loaded. A no-op on a page load that never loaded it. */
export function pauseSiteMusic(): void {
    if (presence.state) pauser?.();
}

/** Story music that is live on this page: the story library's channel, a reader's cue. */
const otherMusic = new Set<() => void>();

/**
 * Register a way to silence music the player does not own. The returned call
 * takes it off; a page registers on mount and takes it off on unmount.
 */
export function registerOtherMusic(silence: () => void): () => void {
    otherMusic.add(silence);
    return () => {
        otherMusic.delete(silence);
    };
}

/** The player is about to sound: silence every story music that registered. */
export function silenceOtherMusic(): void {
    for (const silence of otherMusic) silence();
}
