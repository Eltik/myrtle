"use client";

import { useState } from "react";
import { Button } from "../../ui/shadcn/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "../../ui/shadcn/dialog";
import { Label } from "../../ui/shadcn/label";
import { Textarea } from "../../ui/shadcn/textarea";

interface PublishVersionDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onPublish: (data: { changelog: string; change_summary: string | null }) => Promise<void>;
    tierListName: string;
}

export function PublishVersionDialog({ open, onOpenChange, onPublish, tierListName }: PublishVersionDialogProps) {
    const [changelog, setChangelog] = useState("");
    const [changeSummary, setChangeSummary] = useState("");
    const [publishing, setPublishing] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);

        if (!changelog.trim()) {
            setError("Changelog is required");
            return;
        }

        setPublishing(true);
        try {
            await onPublish({
                changelog: changelog.trim(),
                change_summary: changeSummary.trim() || null,
            });
            // Reset form
            setChangelog("");
            setChangeSummary("");
            onOpenChange(false);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to publish version");
        } finally {
            setPublishing(false);
        }
    };

    const handleClose = () => {
        setChangelog("");
        setChangeSummary("");
        setError(null);
        onOpenChange(false);
    };

    return (
        <Dialog onOpenChange={handleClose} open={open}>
            <DialogContent className="sm:max-w-137.5">
                <DialogHeader>
                    <DialogTitle>Publish New Version</DialogTitle>
                    <DialogDescription>
                        Create a versioned snapshot of <strong>{tierListName}</strong>. This will record the current state of the tier list and make the changelog visible to users.
                    </DialogDescription>
                </DialogHeader>

                <form className="space-y-4" onSubmit={handleSubmit}>
                    <div className="space-y-2">
                        <Label htmlFor="publish-summary">Change Summary</Label>
                        <Textarea id="publish-summary" onChange={(e) => setChangeSummary(e.target.value)} placeholder="Brief summary shown in the changelog header (e.g., 'Balance adjustments for patch 2.3')" rows={2} value={changeSummary} />
                        <p className="text-muted-foreground text-xs">Optional but recommended. This is the short summary users see first.</p>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="publish-changelog">Changelog *</Label>
                        <Textarea
                            id="publish-changelog"
                            onChange={(e) => setChangelog(e.target.value)}
                            placeholder="Detailed changelog describing what changed in this version...

Example:
- Moved Exusiai to S+ tier due to module buff
- Added Ptilopsis to A tier (support category)
- Adjusted Amiya placement after balance changes"
                            rows={6}
                            value={changelog}
                        />
                        <p className="text-muted-foreground text-xs">Required. Describe all changes made since the last version.</p>
                    </div>

                    {error && <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-destructive text-sm">{error}</div>}

                    <DialogFooter>
                        <Button disabled={publishing} onClick={handleClose} type="button" variant="outline">
                            Cancel
                        </Button>
                        <Button disabled={publishing || !changelog.trim()} type="submit">
                            {publishing ? "Publishing..." : "Publish Version"}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
}
