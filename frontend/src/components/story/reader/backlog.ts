/**
 * THE LOG'S REACH: which rows the backlog keeps when the reader moves.
 *
 * The log holds every halt on the CURRENT choice path up to the furthest halt
 * reached, not only the halts behind the reader. Read to line 85, jump back to
 * line 16, and rows 17..85 stay, marked as ahead, each one a jump back there.
 * The rule that decides it is one comparison per halt:
 *
 * - entries before the halt are always kept;
 * - an entry already logged at the halt with the same kind is KNOWN GROUND, and
 *   everything beyond it stays too;
 * - anything else (a first visit, or a branch that went another way) drops the
 *   tail, because those rows belong to a path the reader is no longer on.
 *
 * A choice counts as the same ground only when the SAME option was taken; a
 * different option is a divergence and the rows after it go.
 *
 * Pure, so the rule is tested without a script or a DOM; `useStoryPlayer`
 * calls these at each place the log used to be filtered.
 */

export interface BacklogEntry {
    haltIndex: number;
    /** `cutscene` is a video halt, which has no line of its own to log. */
    kind: "line" | "choice" | "cutscene";
    speaker?: string;
    text: string;
    isNarration: boolean;
    /** A choice row's option VALUE, which is what decides whether a re-choice diverged. */
    value?: string;
}

function before(log: readonly BacklogEntry[], haltIndex: number): BacklogEntry[] {
    return log.filter((e) => e.haltIndex < haltIndex);
}

function after(log: readonly BacklogEntry[], haltIndex: number): BacklogEntry[] {
    return log.filter((e) => e.haltIndex > haltIndex);
}

/**
 * A line or cutscene halt was applied. Known ground keeps the tail and takes
 * the new entry in place of the old one (its text is the one the box drew, so
 * a nickname changed since is the one the log now reads); anything else
 * truncates at the halt and appends.
 */
export function withHalt(log: readonly BacklogEntry[], entry: BacklogEntry): BacklogEntry[] {
    const known = log.find((e) => e.haltIndex === entry.haltIndex);
    if (known && known.kind === entry.kind) return [...before(log, entry.haltIndex), entry, ...after(log, entry.haltIndex)];
    return [...before(log, entry.haltIndex), entry];
}

/**
 * A decision halt was applied: it logs nothing until an option is taken. A
 * choice already logged at it is known ground and the log is unchanged; with
 * none, the tail goes.
 */
export function withDecision(log: readonly BacklogEntry[], haltIndex: number): BacklogEntry[] {
    const known = log.find((e) => e.haltIndex === haltIndex);
    if (known?.kind === "choice") return [...log];
    return before(log, haltIndex);
}

/** An option was taken. The same option keeps the tail; a different one drops it. */
export function withChoice(log: readonly BacklogEntry[], entry: BacklogEntry): BacklogEntry[] {
    const known = log.find((e) => e.haltIndex === entry.haltIndex);
    if (known?.kind === "choice" && known.value !== undefined && known.value === entry.value) return [...before(log, entry.haltIndex), entry, ...after(log, entry.haltIndex)];
    return [...before(log, entry.haltIndex), entry];
}

function sameRow(a: BacklogEntry | undefined, b: BacklogEntry): boolean {
    return a !== undefined && a.kind === b.kind && (a.kind !== "choice" || a.value === b.value);
}

/**
 * The log after a replay to `target` (Back, a backlog or scrubber jump,
 * resume). `probe` is a fresh walk of the path, at least up to `target`;
 * `existing` is the log the reader had.
 *
 * When the existing log walked the SAME path up to the target (every probe
 * row at or before it is in the log, same kind, same option), the rows before
 * the target are the probe's, the rows from it on are the existing ones, and
 * any probe row past the existing reach is appended: that is what keeps
 * 17..85 through a jump back to 16. Otherwise the log IS the probe, which is
 * how a resume rebuilds it out to the saved reach from an empty one.
 */
export function rebuiltTo(probe: readonly BacklogEntry[], existing: readonly BacklogEntry[], target: number): BacklogEntry[] {
    const byHalt = new Map(existing.map((e) => [e.haltIndex, e]));
    const samePath = existing.length > 0 && probe.filter((e) => e.haltIndex <= target).every((e) => sameRow(byHalt.get(e.haltIndex), e));
    if (!samePath) return [...probe];
    const reach = reachOf(existing, target);
    return [...before(probe, target), ...existing.filter((e) => e.haltIndex >= target), ...probe.filter((e) => e.haltIndex > reach)];
}

/** The furthest halt reached on this path: the last logged row, or the current halt when that is further. */
export function reachOf(log: readonly BacklogEntry[], haltIndex: number): number {
    let reach = haltIndex;
    for (const e of log) if (e.haltIndex > reach) reach = e.haltIndex;
    return reach;
}

/**
 * The row the dialog scrolls to when it opens: the current halt's own row, or
 * on a decision (which has no row until it is answered) the last row before
 * it. -1 on an empty log.
 */
export function anchorRow(log: readonly BacklogEntry[], haltIndex: number): number {
    let at = -1;
    for (let i = 0; i < log.length; i++) {
        if (log[i].haltIndex > haltIndex) break;
        at = i;
    }
    return at < 0 && log.length > 0 ? 0 : at;
}
