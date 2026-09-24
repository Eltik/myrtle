import { useStore } from "@tanstack/react-store";
import { CloudIcon, CloudOffIcon, DownloadIcon, LoaderCircleIcon, RotateCcwIcon, TriangleAlertIcon, UploadIcon } from "lucide-react";
import type React from "react";
import { useId, useMemo, useRef, useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { authActions, authStore } from "#/lib/auth/store";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { emptyProgress, loadProgress, overriddenByHand, parseProgress, type StoryProgress, saveProgress, trustingGame } from "#/lib/story/progress";
import { type GameImport, pushStoryProgressNow, resolveStorySync, type SyncState, useStoryGameImport, useStoryProgressSync } from "#/lib/story/sync";
import { type LibIndex, progressSummary } from "./derive";
import type { messages } from "./ProgressTab.messages";
import { ReadingStats } from "./ReadingStats";
import { SyncConflict, SyncPolicySelect } from "./SyncConflict";

export interface IProgressTabProps {
    index: LibIndex;
    progress: StoryProgress;
    /** The game's own read verdict, weighed beside the document. */
    gameRead: ReadonlySet<string>;
    /** Re-reads localStorage after a restore or a reset, so every tab's fractions move together. */
    onProgressChanged: (next: StoryProgress) => void;
}

/**
 * The reading-progress summary, the account sync, and the backup, restore and
 * reset the reader's settings dialog already carries. All of them share
 * `lib/story/progress.ts`, so a backup taken here restores there and the other
 * way round, and every write reaches the sync through the same seam.
 *
 * This is the ONLY surface that reports the sync. The reader shows nothing: a
 * sync state on a story page is a distraction from reading, and the sync never
 * blocks it, so there is nothing there for the reader to act on.
 */
export function ProgressTab({ index, progress, gameRead, onProgressChanged }: IProgressTabProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const f = useFormatters();
    const isAuthenticated = useStore(authStore, (s) => s.user !== null);
    const { state: syncState, syncNow } = useStoryProgressSync();
    const gameImport = useStoryGameImport();
    const fileId = useId();
    const fileRef = useRef<HTMLInputElement>(null);
    const [notice, setNotice] = useState<string | null>(null);
    const [confirmReset, setConfirmReset] = useState(false);

    const summary = useMemo(() => progressSummary(index, progress, gameRead), [index, progress, gameRead]);
    // Hand clears the game disagrees with. A clear is pushed to the account
    // like any mark, so "Account wins" cannot undo it; only this can.
    const overridden = useMemo(() => overriddenByHand(progress, gameRead), [progress, gameRead]);

    const backup = () => {
        const blob = new Blob([JSON.stringify(loadProgress(), null, 2)], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `myrtle-story-progress-${new Date().toISOString().slice(0, 10)}.json`;
        a.click();
        URL.revokeObjectURL(url);
    };

    const restore = async (file: File | undefined) => {
        if (!file) return;
        const text = await file.text();
        const restored = parseProgress(text);
        const stories = Object.keys(restored.read).length;
        const positions = Object.keys(restored.pos).length;
        if (stories === 0 && positions === 0 && !text.includes('"read"')) {
            setNotice(t("progress.restoreFailed"));
            return;
        }
        saveProgress(restored);
        onProgressChanged(restored);
        setNotice(t("progress.restored", { stories, positions }));
    };

    return (
        <div className="flex flex-col gap-5 pt-4">
            <div className="grid grid-cols-2 gap-2.5 lg:grid-cols-4">
                <Stat label={t("progress.finished")} value={t("progress.finishedValue", { read: f.number(summary.read), total: f.number(summary.total) })} />
                <Stat label={t("progress.share")} value={f.percent(summary.fraction)} />
                <Stat label={t("progress.chapters")} value={t("progress.chaptersValue", { done: f.number(summary.groupsDone), total: f.number(summary.groups) })} />
                <Stat label={t("progress.words")} value={summary.words === null ? t("progress.wordsPending") : f.number(summary.words)} muted={summary.words === null} />
            </div>

            <div className="h-1.5 w-full overflow-hidden rounded-full bg-input">
                <div className="h-full rounded-full bg-primary transition-[width] duration-500" style={{ width: `${Math.round(summary.fraction * 100)}%` }} />
            </div>

            <ReadingStats index={index} progress={progress} gameRead={gameRead} />

            {/* A conflict REPLACES the status line rather than sitting under
                it: the row's own line would say "syncing" or a stale "synced",
                and neither is true while both documents are still waiting on
                the reader. */}
            {syncState.kind === "conflict" ? (
                <SyncConflict
                    summary={syncState.summary}
                    index={index}
                    onResolve={(choice) => {
                        resolveStorySync(choice);
                        // The engine writes localStorage for two of the three
                        // choices, and every tab's fractions read the document
                        // this component holds, so the tab re-reads it once the
                        // write has landed.
                        queueMicrotask(() => onProgressChanged(loadProgress()));
                    }}
                />
            ) : (
                <SyncRow state={syncState} signedIn={isAuthenticated} gameImport={gameImport} onSyncNow={syncNow} t={t} f={f} />
            )}

            {isAuthenticated && overridden.length > 0 ? (
                <div className="flex flex-wrap items-center gap-x-4 gap-y-2.5 rounded-[14px] border border-border bg-card p-4 sm:p-5">
                    <div className="min-w-0 flex-1">
                        <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("progress.overrides", { count: f.number(overridden.length) })}</h3>
                        <p className="mt-0.5 mb-0 font-sans text-[12.5px] text-muted-foreground">{t("progress.overrides.blurb")}</p>
                    </div>
                    <Button
                        variant="outline"
                        size="sm"
                        className="max-sm:h-11"
                        onClick={() => {
                            const next = trustingGame(progress, gameRead);
                            saveProgress(next);
                            onProgressChanged(next);
                        }}
                    >
                        {t("progress.overrides.trust")}
                    </Button>
                </div>
            ) : null}

            <div className="rounded-[14px] border border-border bg-card p-4 sm:p-5">
                <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("progress.manage")}</h3>
                <p className="mt-1 mb-3 max-w-prose font-sans text-[12.5px] text-muted-foreground">{isAuthenticated ? t("progress.manageBlurbSynced") : t("progress.manageBlurb")}</p>
                <div className="flex flex-wrap gap-2">
                    <Button variant="outline" size="sm" className="max-sm:h-11" onClick={backup}>
                        <DownloadIcon /> {t("progress.backup")}
                    </Button>
                    <Button variant="outline" size="sm" className="max-sm:h-11" onClick={() => fileRef.current?.click()}>
                        <UploadIcon /> {t("progress.restore")}
                    </Button>
                    <input id={fileId} ref={fileRef} type="file" accept="application/json,.json" className="sr-only" onChange={(e) => void restore(e.target.files?.[0])} />
                    <Button variant="destructive" size="sm" className="max-sm:h-11" onClick={() => setConfirmReset(true)}>
                        <RotateCcwIcon /> {t("progress.reset")}
                    </Button>
                </div>
                <output className="mt-2 block font-sans text-[12px] text-muted-foreground" aria-live="polite">
                    {notice}
                </output>
            </div>

            <AlertDialog open={confirmReset} onOpenChange={setConfirmReset}>
                <AlertDialogPopup>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{t("progress.reset.title")}</AlertDialogTitle>
                        <AlertDialogDescription>{isAuthenticated ? t("progress.reset.bodySynced") : t("progress.reset.body")}</AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogClose render={<Button variant="outline" />}>{t("progress.reset.cancel")}</AlertDialogClose>
                        <Button
                            variant="destructive"
                            onClick={() => {
                                const cleared = emptyProgress();
                                saveProgress(cleared);
                                // The local write alone would reach the account
                                // through the 2 s debounce. A reset is the one
                                // write worth sending at once: it is what the
                                // dialog just promised, on every device.
                                if (isAuthenticated) pushStoryProgressNow(cleared);
                                onProgressChanged(cleared);
                                setConfirmReset(false);
                                setNotice(t("progress.wasReset"));
                            }}
                        >
                            {t("progress.reset.confirm")}
                        </Button>
                    </AlertDialogFooter>
                </AlertDialogPopup>
            </AlertDialog>
        </div>
    );
}

/**
 * The account-sync row: what the sync is doing, what the GAME contributed, and
 * the one button that starts it by hand.
 *
 * Signed out it is an invitation, not an error, so it carries the site's own
 * login dialog rather than a link away from the page the reader is on.
 *
 * The second line reports the import from the game, which is the source the
 * reader never sees working: marks arrive from a profile refresh that happens
 * on another page entirely, so without this line a story turning read has no
 * explanation. An account that has never refreshed game data is told what to do
 * rather than shown a zero.
 */
function SyncRow({ state, signedIn, gameImport, onSyncNow, t, f }: { state: Exclude<SyncState, { kind: "conflict" }>; signedIn: boolean; gameImport: GameImport; onSyncNow: () => void; t: TypedT<typeof messages>; f: ReturnType<typeof useFormatters> }): React.ReactElement {
    const busy = state.kind === "syncing";
    const failed = state.kind === "failed";

    let line: string;
    if (!signedIn || state.kind === "off") line = t("progress.sync.signedOut");
    else if (state.kind === "syncing") line = t("progress.sync.syncing");
    else if (state.kind === "idle") line = state.at === null ? t("progress.sync.pending") : t("progress.sync.synced", { when: f.relativeLong(new Date(state.at).toISOString()) });
    else if (state.reason === "offline") line = t("progress.sync.offline");
    else if (state.reason === "unauthorized") line = t("progress.sync.unauthorized");
    else line = t("progress.sync.failed");

    // Only after a pull has answered: before that the count is unknown, and a
    // "nothing imported" line on a sync that has not run yet would be a claim
    // about the account rather than about the request.
    let gameLine: string | null = null;
    if (signedIn && gameImport !== null) {
        if (gameImport.at === null) gameLine = t("progress.sync.gameNever");
        else {
            const when = f.relativeLong(new Date(gameImport.at * 1000).toISOString());
            gameLine = gameImport.count === 0 ? t("progress.sync.gameNone", { when }) : t("progress.sync.gameImported", { count: f.number(gameImport.count), archived: f.number(gameImport.archived), when });
        }
    }

    const Icon = failed ? (state.reason === "offline" ? CloudOffIcon : TriangleAlertIcon) : busy ? LoaderCircleIcon : CloudIcon;

    return (
        <div className="rounded-[14px] border border-border bg-card p-4 sm:p-5">
            <div className="flex flex-wrap items-center gap-x-4 gap-y-2.5">
                <Icon className={`size-4 shrink-0 ${failed ? "text-destructive" : "text-muted-foreground"} ${busy ? "animate-spin" : ""}`} aria-hidden="true" />
                <div className="min-w-0 flex-1">
                    <h3 className="m-0 font-sans font-semibold text-[14px] text-foreground">{t("progress.sync")}</h3>
                    <p className={`mt-0.5 mb-0 font-sans text-[12.5px] ${failed ? "text-destructive" : "text-muted-foreground"}`} aria-live="polite">
                        {line}
                    </p>
                    {gameLine === null ? null : (
                        <p className="mt-0.5 mb-0 font-sans text-[12.5px] text-muted-foreground" aria-live="polite">
                            {gameLine}
                        </p>
                    )}
                </div>
                {signedIn ? (
                    <Button variant="outline" size="sm" className="max-sm:h-11" disabled={busy} onClick={onSyncNow}>
                        {failed ? t("progress.sync.retry") : t("progress.sync.now")}
                    </Button>
                ) : (
                    <Button variant="outline" size="sm" className="max-sm:h-11" onClick={() => authActions.openLoginDialog()}>
                        {t("progress.sync.signIn")}
                    </Button>
                )}
            </div>
            {/* The standing answer, where a reader can change it without
                waiting for the two sides to disagree again. Signed out there is
                nothing to disagree with, so the line is not there at all. */}
            {signedIn && state.kind === "idle" ? (
                <div className="mt-3 border-border border-t pt-3">
                    <SyncPolicySelect />
                </div>
            ) : null}
        </div>
    );
}

function Stat({ label, value, muted }: { label: string; value: string; muted?: boolean }): React.ReactElement {
    return (
        <div className="rounded-xl border border-border bg-card p-3 sm:p-3.5">
            <div className="font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">{label}</div>
            <div className={muted ? "mt-1 font-sans text-[14px] text-muted-foreground" : "mt-1 font-light font-mono text-[20px] text-foreground tabular-nums tracking-[-0.02em] sm:text-[24px]"}>{value}</div>
        </div>
    );
}
