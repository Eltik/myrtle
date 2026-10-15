/**
 * Listening time the way the stats tile prints it: seconds under a minute,
 * whole minutes under an hour, else hours and minutes. Returned as parts so
 * the message catalogue owns the units.
 */
export type ListeningTime = { unit: "seconds"; seconds: number } | { unit: "minutes"; minutes: number } | { unit: "hours"; hours: number; minutes: number };

export function listeningTime(totalSeconds: number): ListeningTime {
    const s = Math.max(0, Math.floor(Number.isFinite(totalSeconds) ? totalSeconds : 0));
    if (s < 60) return { unit: "seconds", seconds: s };
    if (s < 3600) return { unit: "minutes", minutes: Math.floor(s / 60) };
    return { unit: "hours", hours: Math.floor(s / 3600), minutes: Math.floor((s % 3600) / 60) };
}
