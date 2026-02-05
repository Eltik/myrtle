"use client";

import { useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "~/components/ui/shadcn/dialog";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Textarea } from "~/components/ui/shadcn/textarea";
import type { ReportReason } from "~/types/api/impl/tier-list";

interface ReportTierListDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    tierListSlug: string;
    tierListName: string;
    onReport: (data: { reason: ReportReason; description?: string }) => Promise<void>;
}

const REPORT_REASONS: { value: ReportReason; label: string; description: string }[] = [
    {
        value: "inappropriate_content",
        label: "Inappropriate Content",
        description: "Contains offensive, harmful, or inappropriate content",
    },
    {
        value: "spam",
        label: "Spam",
        description: "Low-quality content, advertisements, or repetitive submissions",
    },
    {
        value: "harassment",
        label: "Harassment",
        description: "Targets or harasses specific users or groups",
    },
    {
        value: "other",
        label: "Other",
        description: "Other violation not listed above",
    },
];

export function ReportTierListDialog({ open, onOpenChange, tierListName, onReport }: ReportTierListDialogProps) {
    const [reason, setReason] = useState<ReportReason | "">("");
    const [description, setDescription] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const selectedReason = REPORT_REASONS.find((r) => r.value === reason);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);

        if (!reason) {
            setError("Please select a reason for reporting");
            return;
        }

        setSubmitting(true);
        try {
            await onReport({
                reason: reason as ReportReason,
                description: description.trim() || undefined,
            });
            // Reset form and close
            setReason("");
            setDescription("");
            onOpenChange(false);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to submit report");
        } finally {
            setSubmitting(false);
        }
    };

    const handleClose = () => {
        setReason("");
        setDescription("");
        setError(null);
        onOpenChange(false);
    };

    return (
        <Dialog onOpenChange={handleClose} open={open}>
            <DialogContent className="sm:max-w-125">
                <DialogHeader>
                    <DialogTitle>Report Tier List</DialogTitle>
                    <DialogDescription>
                        Report <strong>{tierListName}</strong> for violating our community guidelines. Our moderation team will review your report.
                    </DialogDescription>
                </DialogHeader>

                <form className="space-y-4" onSubmit={handleSubmit}>
                    <div className="space-y-2">
                        <Label htmlFor="report-reason">Reason *</Label>
                        <Select onValueChange={(v) => setReason(v as ReportReason)} value={reason}>
                            <SelectTrigger id="report-reason">
                                <SelectValue placeholder="Select a reason" />
                            </SelectTrigger>
                            <SelectContent>
                                {REPORT_REASONS.map((r) => (
                                    <SelectItem key={r.value} value={r.value}>
                                        {r.label}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                        {selectedReason && <p className="text-muted-foreground text-xs">{selectedReason.description}</p>}
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="report-description">Additional Details (Optional)</Label>
                        <Textarea id="report-description" onChange={(e) => setDescription(e.target.value)} placeholder="Provide any additional context that might help our moderation team..." rows={4} value={description} />
                    </div>

                    {error && <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-destructive text-sm">{error}</div>}

                    <DialogFooter>
                        <Button disabled={submitting} onClick={handleClose} type="button" variant="outline">
                            Cancel
                        </Button>
                        <Button disabled={submitting || !reason} type="submit" variant="destructive">
                            {submitting ? "Submitting..." : "Submit Report"}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
}
