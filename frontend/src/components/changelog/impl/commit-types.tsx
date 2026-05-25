import { Bug, FileText, GitCommitHorizontal, type LucideIcon, Paintbrush, RefreshCw, RotateCcw, Sparkles, TestTube, Wrench, Zap } from "lucide-react";
import type { CommitType } from "#/lib/api/changelog";

export interface ICommitTypeStyle {
    label: string;
    Icon: LucideIcon;
    /** Solid color for the timeline dot. */
    dotClass: string;
    /** Tinted pill used for the inline type badge. */
    pillClass: string;
}

export const COMMIT_TYPE_STYLES: Record<CommitType, ICommitTypeStyle> = {
    feature: { label: "Feature", Icon: Sparkles, dotClass: "bg-success", pillClass: "bg-success/10 text-success-foreground ring-success/25" },
    fix: { label: "Fix", Icon: Bug, dotClass: "bg-destructive", pillClass: "bg-destructive/10 text-destructive-foreground ring-destructive/25" },
    perf: { label: "Perf", Icon: Zap, dotClass: "bg-info", pillClass: "bg-info/10 text-info-foreground ring-info/25" },
    refactor: { label: "Refactor", Icon: RefreshCw, dotClass: "bg-warning", pillClass: "bg-warning/10 text-warning-foreground ring-warning/25" },
    docs: { label: "Docs", Icon: FileText, dotClass: "bg-info", pillClass: "bg-info/10 text-info-foreground ring-info/25" },
    style: { label: "Style", Icon: Paintbrush, dotClass: "bg-muted-foreground/60", pillClass: "bg-muted text-muted-foreground ring-border" },
    test: { label: "Test", Icon: TestTube, dotClass: "bg-muted-foreground/60", pillClass: "bg-muted text-muted-foreground ring-border" },
    chore: { label: "Chore", Icon: Wrench, dotClass: "bg-muted-foreground/60", pillClass: "bg-muted text-muted-foreground ring-border" },
    revert: { label: "Revert", Icon: RotateCcw, dotClass: "bg-warning", pillClass: "bg-warning/10 text-warning-foreground ring-warning/25" },
    other: { label: "Update", Icon: GitCommitHorizontal, dotClass: "bg-primary", pillClass: "bg-primary/10 text-primary ring-primary/25" },
};

export function commitTypeStyle(type: CommitType): ICommitTypeStyle {
    return COMMIT_TYPE_STYLES[type] ?? COMMIT_TYPE_STYLES.other;
}
