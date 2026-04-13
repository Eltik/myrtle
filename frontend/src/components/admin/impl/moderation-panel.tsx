"use client";

import { AlertTriangle, CheckCircle, Clock, ExternalLink, Eye, MoreHorizontal, RefreshCw, Shield, XCircle } from "lucide-react";
import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "~/components/ui/shadcn/dialog";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { Label } from "~/components/ui/shadcn/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "~/components/ui/shadcn/table";
import { Tabs, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { Textarea } from "~/components/ui/shadcn/textarea";
import type { ReportStatus, TierListReport } from "~/types/api/impl/tier-list";

type FilterStatus = "all" | "pending" | "reviewed" | "dismissed";

const STATUS_CONFIG: Record<ReportStatus, { label: string; icon: typeof Clock; className: string }> = {
    pending: {
        label: "Pending",
        icon: Clock,
        className: "bg-yellow-500/10 text-yellow-500 border-yellow-500/20",
    },
    reviewed: {
        label: "Reviewed",
        icon: CheckCircle,
        className: "bg-green-500/10 text-green-500 border-green-500/20",
    },
    dismissed: {
        label: "Dismissed",
        icon: XCircle,
        className: "bg-zinc-500/10 text-zinc-400 border-zinc-500/20",
    },
};

const REASON_LABELS: Record<string, string> = {
    inappropriate_content: "Inappropriate Content",
    spam: "Spam",
    harassment: "Harassment",
    other: "Other",
};

function StatusBadge({ status }: { status: ReportStatus }) {
    const config = STATUS_CONFIG[status];
    const Icon = config.icon;
    return (
        <Badge className={`${config.className} gap-1 border font-medium`} variant="outline">
            <Icon className="h-3 w-3" />
            {config.label}
        </Badge>
    );
}

export function ModerationPanel() {
    const [reports, setReports] = useState<TierListReport[]>([]);
    const [loading, setLoading] = useState(true);
    const [filter, setFilter] = useState<FilterStatus>("pending");
    const [moderateDialogOpen, setModerateDialogOpen] = useState(false);
    const [selectedReport, setSelectedReport] = useState<TierListReport | null>(null);
    const [moderationReason, setModerationReason] = useState("");
    const [processing, setProcessing] = useState(false);

    const fetchReports = useCallback(async () => {
        setLoading(true);
        try {
            const statusParam = filter !== "all" ? `?status=${filter}` : "";
            const response = await fetch(`/api/admin/tier-lists/reports${statusParam}`);
            const data = await response.json();

            if (data.success) {
                setReports(data.reports);
            } else {
                console.error("Failed to fetch reports:", data.error);
                toast.error("Failed to fetch reports");
            }
        } catch (error) {
            console.error("Error fetching reports:", error);
            toast.error("Failed to fetch reports");
        } finally {
            setLoading(false);
        }
    }, [filter]);

    useEffect(() => {
        fetchReports();
    }, [fetchReports]);

    const handleReviewReport = useCallback(
        async (reportId: string, action: "approve" | "dismiss", actionTaken?: string) => {
            setProcessing(true);
            try {
                const response = await fetch(`/api/admin/tier-lists/reports/${reportId}/review`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ action, action_taken: actionTaken }),
                });

                const data = await response.json();

                if (data.success) {
                    toast.success(`Report ${action === "approve" ? "approved" : "dismissed"}`);
                    fetchReports();
                } else {
                    toast.error(data.error || "Failed to review report");
                }
            } catch (error) {
                console.error("Error reviewing report:", error);
                toast.error("Failed to review report");
            } finally {
                setProcessing(false);
            }
        },
        [fetchReports],
    );

    const handleModerate = useCallback(async () => {
        if (!selectedReport?.tier_list_slug || !moderationReason.trim()) return;

        setProcessing(true);
        try {
            const response = await fetch(`/api/admin/tier-lists/${selectedReport.tier_list_slug}/moderate`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ reason: moderationReason.trim() }),
            });

            const data = await response.json();

            if (data.success) {
                toast.success("Tier list moderated successfully");
                setModerateDialogOpen(false);
                setModerationReason("");
                setSelectedReport(null);
                // Also mark the report as reviewed
                await handleReviewReport(selectedReport.id, "approve", `Tier list moderated: ${moderationReason.trim()}`);
            } else {
                toast.error(data.error || "Failed to moderate tier list");
            }
        } catch (error) {
            console.error("Error moderating tier list:", error);
            toast.error("Failed to moderate tier list");
        } finally {
            setProcessing(false);
        }
    }, [selectedReport, moderationReason, handleReviewReport]);

    const openModerateDialog = useCallback((report: TierListReport) => {
        setSelectedReport(report);
        setModerationReason("");
        setModerateDialogOpen(true);
    }, []);

    const pendingCount = reports.filter((r) => r.status === "pending").length;

    return (
        <Card>
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                        <Shield className="h-5 w-5 text-primary" />
                        <div>
                            <CardTitle>Content Moderation</CardTitle>
                            <CardDescription>Review and moderate reported community tier lists</CardDescription>
                        </div>
                    </div>
                    <div className="flex items-center gap-2">
                        {pendingCount > 0 && (
                            <Badge className="gap-1 bg-yellow-500/10 text-yellow-500" variant="outline">
                                <AlertTriangle className="h-3 w-3" />
                                {pendingCount} pending
                            </Badge>
                        )}
                        <Button disabled={loading} onClick={fetchReports} size="sm" variant="outline">
                            <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
                        </Button>
                    </div>
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* Filter Tabs */}
                <Tabs defaultValue="pending" onValueChange={(v) => setFilter(v as FilterStatus)} value={filter}>
                    <TabsList>
                        <TabsTrigger value="all">All</TabsTrigger>
                        <TabsTrigger value="pending">Pending</TabsTrigger>
                        <TabsTrigger value="reviewed">Reviewed</TabsTrigger>
                        <TabsTrigger value="dismissed">Dismissed</TabsTrigger>
                    </TabsList>
                </Tabs>

                {/* Reports Table */}
                <div className="rounded-lg border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Tier List</TableHead>
                                <TableHead>Reason</TableHead>
                                <TableHead>Status</TableHead>
                                <TableHead>Reported</TableHead>
                                <TableHead className="w-20">Actions</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {loading ? (
                                <TableRow>
                                    <TableCell className="h-24 text-center" colSpan={5}>
                                        <div className="flex items-center justify-center">
                                            <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ) : reports.length === 0 ? (
                                <TableRow>
                                    <TableCell className="h-24 text-center text-muted-foreground" colSpan={5}>
                                        No reports found.
                                    </TableCell>
                                </TableRow>
                            ) : (
                                reports.map((report) => (
                                    <TableRow key={report.id}>
                                        <TableCell>
                                            <div className="flex flex-col">
                                                <span className="font-medium">{report.tier_list_name || "Unknown"}</span>
                                                <span className="font-mono text-muted-foreground text-xs">{report.tier_list_slug || "-"}</span>
                                            </div>
                                        </TableCell>
                                        <TableCell>
                                            <div className="flex flex-col gap-1">
                                                <span className="text-sm">{REASON_LABELS[report.reason] || report.reason}</span>
                                                {report.description && <span className="line-clamp-1 text-muted-foreground text-xs">{report.description}</span>}
                                            </div>
                                        </TableCell>
                                        <TableCell>
                                            <StatusBadge status={report.status} />
                                        </TableCell>
                                        <TableCell>
                                            <span className="text-muted-foreground text-sm">{new Date(report.created_at).toLocaleDateString()}</span>
                                        </TableCell>
                                        <TableCell>
                                            <DropdownMenu>
                                                <DropdownMenuTrigger asChild>
                                                    <Button className="h-8 w-8" disabled={processing} size="icon" variant="ghost">
                                                        <MoreHorizontal className="h-4 w-4" />
                                                    </Button>
                                                </DropdownMenuTrigger>
                                                <DropdownMenuContent align="end">
                                                    {report.tier_list_slug && (
                                                        <DropdownMenuItem asChild>
                                                            <Link href={`/tier-list?slug=${report.tier_list_slug}`} target="_blank">
                                                                <Eye className="mr-2 h-4 w-4" />
                                                                View Tier List
                                                                <ExternalLink className="ml-auto h-3 w-3" />
                                                            </Link>
                                                        </DropdownMenuItem>
                                                    )}
                                                    {report.status === "pending" && (
                                                        <>
                                                            <DropdownMenuSeparator />
                                                            <DropdownMenuItem onClick={() => handleReviewReport(report.id, "dismiss")}>
                                                                <XCircle className="mr-2 h-4 w-4" />
                                                                Dismiss Report
                                                            </DropdownMenuItem>
                                                            <DropdownMenuItem className="text-destructive focus:text-destructive" onClick={() => openModerateDialog(report)}>
                                                                <Shield className="mr-2 h-4 w-4" />
                                                                Moderate Tier List
                                                            </DropdownMenuItem>
                                                        </>
                                                    )}
                                                </DropdownMenuContent>
                                            </DropdownMenu>
                                        </TableCell>
                                    </TableRow>
                                ))
                            )}
                        </TableBody>
                    </Table>
                </div>
            </CardContent>

            {/* Moderate Dialog */}
            <Dialog onOpenChange={setModerateDialogOpen} open={moderateDialogOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>Moderate Tier List</DialogTitle>
                        <DialogDescription>
                            This will hide <strong>{selectedReport?.tier_list_name}</strong> from public view. Please provide a reason for this action.
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="moderation-reason">Reason *</Label>
                            <Textarea id="moderation-reason" onChange={(e) => setModerationReason(e.target.value)} placeholder="Explain why this tier list is being moderated..." rows={4} value={moderationReason} />
                        </div>
                    </div>
                    <DialogFooter>
                        <Button disabled={processing} onClick={() => setModerateDialogOpen(false)} variant="outline">
                            Cancel
                        </Button>
                        <Button disabled={processing || !moderationReason.trim()} onClick={handleModerate} variant="destructive">
                            {processing ? "Processing..." : "Moderate Tier List"}
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </Card>
    );
}
