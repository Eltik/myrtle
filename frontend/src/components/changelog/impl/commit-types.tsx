import { Bug, FileText, GitCommitHorizontal, type LucideIcon, Paintbrush, RefreshCw, RotateCcw, Sparkles, TestTube, Wrench, Zap } from "lucide-react";
import type { CommitType } from "#/lib/api/changelog";
import type { messages } from "./commit-types.messages";

export type CommitTypeMessageKey = keyof typeof messages & string;

export interface ICommitTypeStyle {
    labelKey: CommitTypeMessageKey;
    Icon: LucideIcon;
    /** Solid color for the timeline dot. */
    dotClass: string;
    /** Tinted pill used for the inline type badge. */
    pillClass: string;
}

export const COMMIT_TYPE_STYLES: Record<CommitType, ICommitTypeStyle> = {
    feature: { labelKey: "commitType.feature", Icon: Sparkles, dotClass: "bg-success", pillClass: "bg-success/10 text-success-foreground ring-success/25" },
    fix: { labelKey: "commitType.fix", Icon: Bug, dotClass: "bg-destructive", pillClass: "bg-destructive/10 text-destructive-foreground ring-destructive/25" },
    perf: { labelKey: "commitType.perf", Icon: Zap, dotClass: "bg-info", pillClass: "bg-info/10 text-info-foreground ring-info/25" },
    refactor: { labelKey: "commitType.refactor", Icon: RefreshCw, dotClass: "bg-warning", pillClass: "bg-warning/10 text-warning-foreground ring-warning/25" },
    docs: { labelKey: "commitType.docs", Icon: FileText, dotClass: "bg-info", pillClass: "bg-info/10 text-info-foreground ring-info/25" },
    style: { labelKey: "commitType.style", Icon: Paintbrush, dotClass: "bg-muted-foreground/60", pillClass: "bg-muted text-muted-foreground ring-border" },
    test: { labelKey: "commitType.test", Icon: TestTube, dotClass: "bg-muted-foreground/60", pillClass: "bg-muted text-muted-foreground ring-border" },
    chore: { labelKey: "commitType.chore", Icon: Wrench, dotClass: "bg-muted-foreground/60", pillClass: "bg-muted text-muted-foreground ring-border" },
    revert: { labelKey: "commitType.revert", Icon: RotateCcw, dotClass: "bg-warning", pillClass: "bg-warning/10 text-warning-foreground ring-warning/25" },
    other: { labelKey: "commitType.other", Icon: GitCommitHorizontal, dotClass: "bg-primary", pillClass: "bg-primary/10 text-primary ring-primary/25" },
};

export function commitTypeStyle(type: CommitType): ICommitTypeStyle {
    return COMMIT_TYPE_STYLES[type] ?? COMMIT_TYPE_STYLES.other;
}
