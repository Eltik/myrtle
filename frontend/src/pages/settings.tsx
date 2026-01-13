"use client";

import { ChevronRight, RefreshCw, SettingsIcon, UserX } from "lucide-react";
import { motion } from "motion/react";
import type { GetServerSideProps } from "next";
import { useState } from "react";
import { toast } from "sonner";
import { SEO } from "~/components/seo";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Separator } from "~/components/ui/shadcn/separator";
import { Switch } from "~/components/ui/shadcn/switch";
import { useAuth } from "~/hooks/use-auth";

export default function SettingsPage() {
    const { user, loading, refreshProfile } = useAuth();
    const [publicProfile, setPublicProfile] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [isSaving, setIsSaving] = useState(false);
    const [showPrivacy, setShowPrivacy] = useState(true);
    const [showProfile, setShowProfile] = useState(true);

    const handleVisibilityToggle = async (checked: boolean) => {
        setIsSaving(true);
        try {
            const response = await fetch("/api/settings/update-visibility", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ publicProfile: checked }),
            });

            const data = await response.json();

            if (data.success) {
                setPublicProfile(checked);
                toast.success(checked ? "Profile is now visible on leaderboards" : "Profile hidden from leaderboards");
            } else {
                toast.error(data.message || "Failed to update visibility");
            }
        } catch (error) {
            console.error("Error updating visibility:", error);
            toast.error("Failed to update profile visibility");
        } finally {
            setIsSaving(false);
        }
    };

    const handleRefreshProfile = async () => {
        setIsRefreshing(true);
        try {
            const result = await refreshProfile();

            if (result.success) {
                toast.success("Profile refreshed successfully!");
            } else {
                toast.error(result.message || "Failed to refresh profile");
            }
        } catch (error) {
            console.error("Error refreshing profile:", error);
            toast.error("Failed to refresh profile");
        } finally {
            setIsRefreshing(false);
        }
    };

    if (loading) {
        return (
            <>
                <SEO description="Manage your myrtle.moe profile settings, privacy preferences, and account options." noIndex path="/settings" title="Settings" />
                <div className="flex min-h-[50vh] items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            </>
        );
    }

    if (!user?.status) {
        return (
            <>
                <SEO description="Manage your myrtle.moe profile settings, privacy preferences, and account options." noIndex path="/settings" title="Settings" />
                <div className="container mx-auto flex min-h-[50vh] items-center justify-center p-4">
                    <Card className="max-w-md">
                        <CardHeader>
                            <CardTitle>Authentication Required</CardTitle>
                            <CardDescription>Please log in to access your settings.</CardDescription>
                        </CardHeader>
                    </Card>
                </div>
            </>
        );
    }

    return (
        <>
            <SEO description="Manage your myrtle.moe profile settings, privacy preferences, and account options." noIndex path="/settings" title="Settings" />

            <div className="mx-auto max-w-3xl space-y-6">
                <div className="space-y-2">
                    <div className="flex items-center gap-3">
                        <SettingsIcon className="h-8 w-8 text-primary" />
                        <h1 className="font-bold text-3xl">Settings</h1>
                    </div>
                    <p className="text-muted-foreground">Manage your profile and preferences</p>
                </div>

                <Card>
                    <CardHeader>
                        <CardTitle>Privacy Settings</CardTitle>
                        <CardDescription>Control how your profile appears to other users</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        <Disclosure onOpenChange={setShowPrivacy} open={showPrivacy} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                            <DisclosureTrigger>
                                <div className="flex cursor-pointer items-center justify-between rounded-lg border border-border bg-card p-4 transition-colors hover:bg-secondary/50">
                                    <div className="flex items-center gap-3">
                                        <UserX className="h-5 w-5 text-primary" />
                                        <div className="text-left">
                                            <p className="font-medium text-sm">Leaderboard Visibility</p>
                                            <p className="text-muted-foreground text-xs">{publicProfile ? "Your profile is visible on public leaderboards" : "Your profile is hidden from public leaderboards"}</p>
                                        </div>
                                    </div>
                                    <motion.div animate={{ rotate: showPrivacy ? 90 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                    </motion.div>
                                </div>
                            </DisclosureTrigger>
                            <DisclosureContent>
                                <div className="mt-4 space-y-4 rounded-lg border border-border bg-muted/30 p-4">
                                    <div className="flex items-center justify-between">
                                        <div className="space-y-0.5">
                                            <label className="font-medium text-sm" htmlFor="public-profile">
                                                Public Profile
                                            </label>
                                            <p className="text-muted-foreground text-xs">Allow your profile to appear on leaderboards</p>
                                        </div>
                                        <Switch checked={publicProfile} disabled={isSaving} id="public-profile" onCheckedChange={handleVisibilityToggle} />
                                    </div>

                                    <Separator />

                                    <div className="text-muted-foreground text-xs">
                                        <p className="mb-2 font-medium">What this affects:</p>
                                        <ul className="ml-4 list-disc space-y-1">
                                            <li>Visibility on global and server leaderboards</li>
                                            <li>Appearance in player search results</li>
                                            <li>Profile discoverability by other users</li>
                                        </ul>
                                        <p className="mt-2">Your profile will still be accessible via direct link even when hidden from leaderboards.</p>
                                    </div>
                                </div>
                            </DisclosureContent>
                        </Disclosure>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>Profile Management</CardTitle>
                        <CardDescription>Update your profile data from the game servers</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        <Disclosure onOpenChange={setShowProfile} open={showProfile} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                            <DisclosureTrigger>
                                <div className="flex cursor-pointer items-center justify-between rounded-lg border border-border bg-card p-4 transition-colors hover:bg-secondary/50">
                                    <div className="flex items-center gap-3">
                                        <RefreshCw className="h-5 w-5 text-primary" />
                                        <div className="text-left">
                                            <p className="font-medium text-sm">Refresh Profile Data</p>
                                            <p className="text-muted-foreground text-xs">Sync your latest game progress and statistics</p>
                                        </div>
                                    </div>
                                    <motion.div animate={{ rotate: showProfile ? 90 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                    </motion.div>
                                </div>
                            </DisclosureTrigger>
                            <DisclosureContent>
                                <div className="mt-4 space-y-4 rounded-lg border border-border bg-muted/30 p-4">
                                    <div className="text-muted-foreground text-sm">
                                        <p className="mb-3">Refreshing your profile will fetch the latest data from the game servers, including:</p>
                                        <ul className="ml-4 list-disc space-y-1">
                                            <li>Character roster and levels</li>
                                            <li>Inventory and materials</li>
                                            <li>Base progress and upgrades</li>
                                            <li>Account statistics and achievements</li>
                                        </ul>
                                        <p className="mt-3 text-xs">This process may take a few moments to complete.</p>
                                    </div>

                                    <Separator />

                                    <Button className="w-full" disabled={isRefreshing} onClick={handleRefreshProfile} size="lg" variant="default">
                                        {isRefreshing ? (
                                            <>
                                                <div className="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
                                                Refreshing Profile...
                                            </>
                                        ) : (
                                            <>
                                                <RefreshCw className="mr-2 h-4 w-4" />
                                                Refresh Profile Now
                                            </>
                                        )}
                                    </Button>
                                </div>
                            </DisclosureContent>
                        </Disclosure>
                    </CardContent>
                </Card>

                <Card className="border-dashed">
                    <CardHeader>
                        <CardTitle className="text-muted-foreground">Coming Soon</CardTitle>
                        <CardDescription>More settings and features will be added in future updates</CardDescription>
                    </CardHeader>
                </Card>
            </div>
        </>
    );
}

// Server-side props - optional, can be used for future enhancements
export const getServerSideProps: GetServerSideProps = async () => {
    return {
        props: {},
    };
};
