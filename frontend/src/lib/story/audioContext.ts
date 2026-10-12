/**
 * The AudioContext constructor, or undefined where the browser has none.
 * Safari before 14.1 exposes it only as `webkitAudioContext`. Shared by the
 * story audio and the music player, which each build their own context.
 */
export function audioContextConstructor(): typeof AudioContext | undefined {
    return window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
}
