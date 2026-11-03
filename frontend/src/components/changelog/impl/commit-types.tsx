import { Bug, FileText, GitCommitHorizontal, type LucideIcon, Paintbrush, RefreshCw, RotateCcw, Sparkles, TestTube, Wrench, Zap } from "lucide-react";
import type { CommitType } from "#/lib/api/changelog";
import type { messages } from "./commit-types.messages";

export type CommitTypeMessageKey = keyof typeof messages & string;

export interface ICommitTypeStyle {
    labelKey: CommitTypeMessageKey;
    Icon: LucideIcon;
    /** Solid color for the timeline dot. */
    dotClass: string;
    /** CSS colour for the tint badge's `--tone` (fill and border; the label stays foreground). */
    tone: string;
}

export const COMMIT_TYPE_STYLES: Record<CommitType, ICommitTypeStyle> = {
    feature: { labelKey: "commitType.feature", Icon: Sparkles, dotClass: "bg-success", tone: "var(--success)" },
    fix: { labelKey: "commitType.fix", Icon: Bug, dotClass: "bg-destructive", tone: "var(--destructive)" },
    perf: { labelKey: "commitType.perf", Icon: Zap, dotClass: "bg-info", tone: "var(--info)" },
    refactor: { labelKey: "commitType.refactor", Icon: RefreshCw, dotClass: "bg-warning", tone: "var(--warning)" },
    docs: { labelKey: "commitType.docs", Icon: FileText, dotClass: "bg-info", tone: "var(--info)" },
    style: { labelKey: "commitType.style", Icon: Paintbrush, dotClass: "bg-muted-foreground/60", tone: "var(--muted-foreground)" },
    test: { labelKey: "commitType.test", Icon: TestTube, dotClass: "bg-muted-foreground/60", tone: "var(--muted-foreground)" },
    chore: { labelKey: "commitType.chore", Icon: Wrench, dotClass: "bg-muted-foreground/60", tone: "var(--muted-foreground)" },
    revert: { labelKey: "commitType.revert", Icon: RotateCcw, dotClass: "bg-warning", tone: "var(--warning)" },
    other: { labelKey: "commitType.other", Icon: GitCommitHorizontal, dotClass: "bg-primary", tone: "var(--primary)" },
};

export function commitTypeStyle(type: CommitType): ICommitTypeStyle {
    return COMMIT_TYPE_STYLES[type] ?? COMMIT_TYPE_STYLES.other;
}
