/**
 * The stories tab's lists (contract v1.1 section 5): the popular questions of
 * the reader's era (`GET /trevor/popular`) and the reader's own past questions
 * (`GET /trevor/me/jobs`, "My questions"), and how a past question becomes a
 * thread turn.
 */
import type { TrevorMyQuestion } from "#/types/generated/TrevorMyQuestion";
import type { TrevorPopular } from "#/types/generated/TrevorPopular";
import type { TrevorPopularQuestion } from "#/types/generated/TrevorPopularQuestion";
import type { TrevorSpoilerSafe } from "#/types/generated/TrevorSpoilerSafe";
import { parseAnswer } from "./answer";
import { foldQuestion } from "./chips";
import type { TurnState } from "./thread";

export interface IPopular {
    questions: TrevorPopularQuestion[];
    /** The era the list was drawn from; null signed out and before the first era. */
    spoiler: TrevorSpoilerSafe | null;
}

export interface IMyJob extends TrevorMyQuestion {
    /** The row's key in the list: the job, else the time and question. */
    key: string;
}

export interface IMyJobs {
    jobs: IMyJob[];
    /** The `before` cursor of the next page, or null on the last one. */
    nextBefore: string | null;
}

/** The popular list with each question trimmed and a question listed twice kept once. */
export function parsePopular(body: TrevorPopular): IPopular {
    const questions: TrevorPopularQuestion[] = [];
    const seen = new Set<string>();
    for (const item of body.items) {
        const question = item.question.trim();
        if (!question || seen.has(foldQuestion(question))) continue;
        seen.add(foldQuestion(question));
        questions.push({ ...item, question });
    }
    return { questions, spoiler: body.spoilerSafeUpTo };
}

/** One page of My questions. The backend sends no cursor: a full page continues before its last row's time. */
export function parseMyJobs(rows: TrevorMyQuestion[], limit: number): IMyJobs {
    const jobs = rows.map((row) => ({ ...row, key: row.jobId ?? `${row.createdAt}|${row.question}` }));
    const last = jobs[jobs.length - 1];
    return { jobs, nextBefore: jobs.length >= limit && last ? last.createdAt : null };
}

/** A past question as a thread turn: answered, failed, or open and to be followed again. */
export function toTurnState(job: IMyJob): TurnState {
    const question = job.question;
    const flag = job.spoilers === "all" ? { spoilers: true as const } : {};
    if (job.status === "done" && job.answer) return { kind: "done", question, answer: parseAnswer(job.answer), rung: job.answer.rung, spoiler: null, cached: true, ...flag };
    if (job.status === "done" || job.status === "failed" || !job.jobId) return { kind: "failed", question, message: "" };
    return { kind: "queued", question, jobId: job.jobId, rung: job.rung, spoiler: null, etaSecs: null, position: null, workerOnline: true, ...flag };
}

/** The job is still open (queued or running). */
export function jobOpen(job: IMyJob): boolean {
    return job.status === "queued" || job.status === "running";
}
