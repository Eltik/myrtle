"use client";

import { BarChart3, ChevronRight, Database, RefreshCw, SettingsIcon, UserX } from "lucide-react";
import { motion } from "motion/react";
import type { GetServerSideProps } from "next";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import ProtectedPageLayout from "~/components/layout/protected-page-layout";
import { SEO } from "~/components/seo";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Separator } from "~/components/ui/shadcn/separator";
import { Switch } from "~/components/ui/shadcn/switch";
import { useAuth } from "~/hooks/use-auth";

interface GachaSettings {
    store_records: boolean;
    share_anonymous_stats: boolean;
    total_pulls: number;
    six_star_count: number;
    five_star_count: number;
}

function SettingsPageContent() {
    const { user, refreshProfile } = useAuth();
    const [publicProfile, setPublicProfile] = useState(true);
    const [isRefreshing, setIsRefreshing] = useState(false);
    const [isSaving, setIsSaving] = useState(false);
    const [showPrivacy, setShowPrivacy] = useState(true);
    const [showProfile, setShowProfile] = useState(true);
    const [showGacha, setShowGacha] = useState(true);

    // Gacha settings state
    const [gachaSettings, setGachaSettings] = useState<GachaSettings | null>(null);
    const [isLoadingGacha, setIsLoadingGacha] = useState(true);
    const [isSavingGacha, setIsSavingGacha] = useState(false);

    // Fetch gacha settings on mount
    const fetchGachaSettings = useCallback(async () => {
        try {
            const response = await fetch("/api/gacha/settings");
            const data = await response.json();

            if (data.success) {
                setGachaSettings(data.settings);
            }
        } catch (error) {
            console.error("Error fetching gacha settings:", error);
        } finally {
            setIsLoadingGacha(false);
        }
    }, []);

    useEffect(() => {
        if (user?.status) {
            fetchGachaSettings();
        }
    }, [user?.status, fetchGachaSettings]);

    // Handle gacha storage toggle
    const handleGachaStorageToggle = async (checked: boolean) => {
        setIsSavingGacha(true);
        try {
            const response = await fetch("/api/gacha/settings", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ storeRecords: checked }),
            });

            const data = await response.json();

            if (data.success) {
                setGachaSettings(data.settings);
                toast.success(checked ? "Gacha records will now be stored" : "Gacha record storage disabled");
            } else {
                toast.error(data.error || "Failed to update gacha settings");
            }
        } catch (error) {
            console.error("Error updating gacha settings:", error);
            toast.error("Failed to update gacha settings");
        } finally {
            setIsSavingGacha(false);
        }
    };

    // Handle anonymous stats toggle
    const handleAnonymousStatsToggle = async (checked: boolean) => {
        setIsSavingGacha(true);
        try {
            const response = await fetch("/api/gacha/settings", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ shareAnonymousStats: checked }),
            });

            const data = await response.json();

            if (data.success) {
                setGachaSettings(data.settings);
                toast.success(checked ? "Your data will be included in global statistics" : "Your data will be excluded from global statistics");
            } else {
                toast.error(data.error || "Failed to update gacha settings");
            }
        } catch (error) {
            console.error("Error updating gacha settings:", error);
            toast.error("Failed to update gacha settings");
        } finally {
            setIsSavingGacha(false);
        }
    };

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
    return (
        <>
            <SEO description="Manage your myrtle.moe profile settings, privacy preferences, and account options." noIndex path="/my/settings" title="Settings" />

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

                <Card>
                    <CardHeader>
                        <CardTitle>Gacha Settings</CardTitle>
                        <CardDescription>Control how your gacha pull data is stored and used</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        <Disclosure onOpenChange={setShowGacha} open={showGacha} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                            <DisclosureTrigger>
                                <div className="flex cursor-pointer items-center justify-between rounded-lg border border-border bg-card p-4 transition-colors hover:bg-secondary/50">
                                    <div className="flex items-center gap-3">
                                        <Database className="h-5 w-5 text-primary" />
                                        <div className="text-left">
                                            <p className="font-medium text-sm">Gacha Data Storage</p>
                                            <p className="text-muted-foreground text-xs">{isLoadingGacha ? "Loading settings..." : gachaSettings?.store_records ? "Your gacha records are being stored" : "Gacha record storage is disabled"}</p>
                                        </div>
                                    </div>
                                    <motion.div animate={{ rotate: showGacha ? 90 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                                        <ChevronRight className="h-4 w-4 text-muted-foreground" />
                                    </motion.div>
                                </div>
                            </DisclosureTrigger>
                            <DisclosureContent>
                                <div className="mt-4 space-y-4 rounded-lg border border-border bg-muted/30 p-4">
                                    {isLoadingGacha ? (
                                        <div className="flex items-center justify-center py-4">
                                            <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                                        </div>
                                    ) : (
                                        <>
                                            <div className="flex items-center justify-between">
                                                <div className="space-y-0.5">
                                                    <label className="font-medium text-sm" htmlFor="store-gacha">
                                                        Store Gacha Records
                                                    </label>
                                                    <p className="text-muted-foreground text-xs">Save your pull history when viewing gacha data</p>
                                                </div>
                                                <Switch checked={gachaSettings?.store_records ?? true} disabled={isSavingGacha} id="store-gacha" onCheckedChange={handleGachaStorageToggle} />
                                            </div>

                                            <Separator />

                                            <div className="flex items-center justify-between">
                                                <div className="space-y-0.5">
                                                    <label className="font-medium text-sm" htmlFor="share-stats">
                                                        Share Anonymous Statistics
                                                    </label>
                                                    <p className="text-muted-foreground text-xs">Include your data in global pull rate statistics</p>
                                                </div>
                                                <Switch checked={gachaSettings?.share_anonymous_stats ?? true} disabled={isSavingGacha} id="share-stats" onCheckedChange={handleAnonymousStatsToggle} />
                                            </div>

                                            <Separator />

                                            {gachaSettings && gachaSettings.total_pulls > 0 && (
                                                <>
                                                    <div className="flex items-center gap-2 text-muted-foreground text-xs">
                                                        <BarChart3 className="h-4 w-4" />
                                                        <span>Your stored statistics:</span>
                                                    </div>
                                                    <div className="grid grid-cols-3 gap-4 text-center">
                                                        <div className="rounded-lg bg-background p-3">
                                                            <p className="font-bold text-lg">{gachaSettings.total_pulls.toLocaleString()}</p>
                                                            <p className="text-muted-foreground text-xs">Total Pulls</p>
                                                        </div>
                                                        <div className="rounded-lg bg-background p-3">
                                                            <p className="font-bold text-lg text-orange-500">{gachaSettings.six_star_count.toLocaleString()}</p>
                                                            <p className="text-muted-foreground text-xs">6★ Operators</p>
                                                        </div>
                                                        <div className="rounded-lg bg-background p-3">
                                                            <p className="font-bold text-lg text-yellow-500">{gachaSettings.five_star_count.toLocaleString()}</p>
                                                            <p className="text-muted-foreground text-xs">5★ Operators</p>
                                                        </div>
                                                    </div>
                                                    <Separator />
                                                </>
                                            )}

                                            <div className="text-muted-foreground text-xs">
                                                <p className="mb-2 font-medium">About gacha data storage:</p>
                                                <ul className="ml-4 list-disc space-y-1">
                                                    <li>When enabled, your pull history is automatically saved when you view the Gacha page</li>
                                                    <li>Stored data helps track your personal pull statistics over time</li>
                                                    <li>Anonymous statistics contribute to global pull rate calculations</li>
                                                    <li>You can disable storage at any time without losing previously stored data</li>
                                                </ul>
                                            </div>
                                        </>
                                    )}
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

export default function SettingsPage() {
    return (
        <ProtectedPageLayout>
            <SettingsPageContent />
        </ProtectedPageLayout>
    );
}
// Server-side props - optional, can be used for future enhancements
export const getServerSideProps: GetServerSideProps = async () => {
    return {
        props: {},
    };
};
