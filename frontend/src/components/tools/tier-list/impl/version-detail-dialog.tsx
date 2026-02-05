"use client";

import { Calendar, FileText, Hash } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Badge } from "~/components/ui/shadcn/badge";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "~/components/ui/shadcn/dialog";
import { Separator } from "~/components/ui/shadcn/separator";
import type { TierListVersionDetail, TierListVersionSummary } from "~/types/api/impl/tier-list";

interface VersionDetailDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    version: TierListVersionSummary | null;
    tierListSlug: string;
}

export function VersionDetailDialog({ open, onOpenChange, version, tierListSlug }: VersionDetailDialogProps) {
    const [details, setDetails] = useState<TierListVersionDetail | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const fetchVersionDetails = useCallback(async () => {
        if (!version) return;

        setLoading(true);
        setError(null);

        try {
            const response = await fetch(`/api/tier-lists/${tierListSlug}/versions/${version.version}`);

            if (!response.ok) {
                throw new Error("Failed to fetch version details");
            }

            const data = (await response.json()) as TierListVersionDetail;
            setDetails(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : "An error occurred");
        } finally {
            setLoading(false);
        }
    }, [version, tierListSlug]);

    useEffect(() => {
        if (open && version) {
            fetchVersionDetails();
        } else {
            setDetails(null);
            setError(null);
        }
    }, [open, version, fetchVersionDetails]);

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString("en-US", {
            year: "numeric",
            month: "long",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    };

    return (
        <Dialog onOpenChange={onOpenChange} open={open}>
            <DialogContent className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <FileText className="h-5 w-5" />
                        Version Details
                        {version && <Badge variant="secondary">v{version.version}</Badge>}
                    </DialogTitle>
                    <DialogDescription>{version && `Published on ${formatDate(version.published_at)}`}</DialogDescription>
                </DialogHeader>

                {loading ? (
                    <div className="flex min-h-50 items-center justify-center">
                        <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                    </div>
                ) : error ? (
                    <div className="rounded-md border border-destructive/50 bg-destructive/10 p-4 text-center text-destructive">{error}</div>
                ) : details ? (
                    <div className="space-y-4">
                        {/* Version Info */}
                        <div className="flex flex-wrap gap-4 text-sm">
                            <div className="flex items-center gap-2 text-muted-foreground">
                                <Hash className="h-4 w-4" />
                                <span>Version {details.version}</span>
                            </div>
                            <div className="flex items-center gap-2 text-muted-foreground">
                                <Calendar className="h-4 w-4" />
                                <span>{formatDate(details.published_at)}</span>
                            </div>
                        </div>

                        {/* Change Summary */}
                        {details.change_summary && (
                            <>
                                <Separator />
                                <div className="space-y-2">
                                    <h4 className="font-medium text-foreground text-sm">Summary</h4>
                                    <p className="text-muted-foreground text-sm">{details.change_summary}</p>
                                </div>
                            </>
                        )}

                        {/* Full Changelog */}
                        <Separator />
                        <div className="space-y-2">
                            <h4 className="font-medium text-foreground text-sm">Changelog</h4>
                            <div className="max-h-75 overflow-y-auto rounded-md border border-border/50 bg-muted/30 p-4">
                                <pre className="whitespace-pre-wrap font-sans text-muted-foreground text-sm leading-relaxed">{details.changelog}</pre>
                            </div>
                        </div>
                    </div>
                ) : null}
            </DialogContent>
        </Dialog>
    );
}
