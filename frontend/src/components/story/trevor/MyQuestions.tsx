/** "My questions" on the stories page's Ask tab: the reader's past questions, newest first, each with its status. */
import { HistoryIcon } from "lucide-react";
import type React from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Spinner } from "#/components/ui/spinner";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { type IMyJob, jobOpen } from "#/lib/trevor/library";
import { cn } from "#/lib/utils";
import type { JobStatus } from "#/types/generated/JobStatus";
import { SpoilersBadge } from "./SpoilerSwitch";
import type { messages } from "./trevor.messages";

type TrevorT = TypedT<typeof messages>;

/** The reader's past questions, newest first; each opens into the thread. */
export function MyQuestions({ jobs, loading, failed, hasMore, onMore, onOpen }: { jobs: IMyJob[]; loading: boolean; failed: boolean; hasMore: boolean; onMore: () => void; onOpen: (job: IMyJob) => void }): React.ReactElement {
    const t: TrevorT = useT("story");
    const f = useFormatters();
    return (
        <section className="flex flex-col gap-2" aria-labelledby="trevor-mine" data-trevor-mine>
            <h3 id="trevor-mine" className="flex items-center gap-1.5 font-semibold text-muted-foreground text-xs uppercase tracking-wide">
                <HistoryIcon aria-hidden className="size-3.5" />
                {t("trevor.library.mine")}
            </h3>
            {loading ? (
                <div className="flex justify-center py-4">
                    <Spinner className="size-4" />
                </div>
            ) : failed ? (
                <p className="text-muted-foreground text-sm">{t("trevor.library.mineError")}</p>
            ) : jobs.length === 0 ? (
                <p className="text-muted-foreground text-sm">{t("trevor.library.mineEmpty")}</p>
            ) : (
                <ul className="flex flex-col gap-1.5">
                    {jobs.map((job) => (
                        <li key={job.key}>
                            <button
                                type="button"
                                onClick={() => onOpen(job)}
                                className="flex w-full flex-col gap-1 rounded-lg border border-border bg-background px-3 py-2 text-start transition-colors hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring max-sm:min-h-11"
                                data-trevor-mine-job={job.jobId ?? job.key}
                                data-trevor-mine-status={job.status}
                            >
                                <span className="line-clamp-2 text-sm">{job.question}</span>
                                <span className="flex flex-wrap items-center gap-x-2 gap-y-1 text-muted-foreground text-xs">
                                    <JobStatusBadge job={job} t={t} />
                                    {job.spoilers === "all" ? <SpoilersBadge /> : null}
                                    {Number.isNaN(Date.parse(job.createdAt)) ? null : <span className="tabular-nums">{f.relative(new Date(job.createdAt).toISOString())}</span>}
                                </span>
                            </button>
                        </li>
                    ))}
                </ul>
            )}
            {hasMore && !loading && !failed ? (
                <Button variant="ghost" size="sm" className="self-start max-sm:h-11" onClick={onMore}>
                    {t("trevor.library.mineMore")}
                </Button>
            ) : null}
        </section>
    );
}

function JobStatusBadge({ job, t }: { job: IMyJob; t: TrevorT }): React.ReactElement {
    const open = jobOpen(job);
    return (
        <Badge variant="outline" className={cn("h-auto py-0 font-normal", open && "border-primary/40 text-foreground", job.status === "failed" && "border-warning/40")}>
            {open ? <Spinner className="size-3" /> : null}
            {statusLabel(job.status, t)}
        </Badge>
    );
}

function statusLabel(status: JobStatus, t: TrevorT): string {
    switch (status) {
        case "queued":
            return t("trevor.library.status.queued");
        case "running":
            return t("trevor.library.status.running");
        case "done":
            return t("trevor.library.status.done");
        case "failed":
            return t("trevor.library.status.failed");
    }
}
