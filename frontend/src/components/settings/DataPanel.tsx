import { Link } from "@tanstack/react-router";
import { CheckIcon, KeyRoundIcon, LogOutIcon, MailIcon, RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import type { IUserProfile } from "#/types/user";
import { SettingRow } from "./SettingsShell";

interface IDataPanelProps {
    user: IUserProfile;
    onResync: () => void;
    syncing: boolean;
    onSignOut: () => void;
    signingOut: boolean;
}

export function DataPanel({ user, onResync, syncing, onSignOut, signingOut }: IDataPanelProps) {
    return (
        <div className="flex flex-col gap-4">
            <Card>
                <CardHeader>
                    <CardTitle>Sync &amp; refresh</CardTitle>
                    <CardDescription>Pull the latest from Yostar. Re-sync any time - we replace your stored snapshot.</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        title="Re-sync game data"
                        description="Operators, stage progress, IS/Sandbox, base, medals, inventory."
                        control={
                            <Button size="sm" onClick={onResync} disabled={syncing} loading={syncing} className="w-full sm:w-auto">
                                <RefreshCwIcon className="size-3.5" />
                                {syncing ? "Re-syncing…" : "Re-sync now"}
                            </Button>
                        }
                    />
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Your data on Myrtle</CardTitle>
                    <CardDescription>Everything we store is visible on your profile page. For a portable copy or a GDPR request, email privacy directly.</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        title="View on your profile"
                        description="Roster, scores, stage progress, gacha history (if stored), and medals - the data we have on you is what shows up here."
                        control={
                            <Button variant="outline" size="sm" render={<Link to="/user/$id" params={{ id: user.uid }} />}>
                                Open my profile
                            </Button>
                        }
                    />
                    <SettingRow
                        title="Request data export"
                        description="Email privacy@myrtle.moe to receive a portable copy of your data."
                        control={
                            // biome-ignore lint/a11y/useAnchorContent: anchor children are slotted in by Button via render prop
                            <Button variant="outline" size="sm" render={<a href="mailto:privacy@myrtle.moe?subject=Data%20export%20request" />}>
                                <MailIcon className="size-3.5" />
                                Email privacy@myrtle.moe
                            </Button>
                        }
                    />
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Linked Yostar account</CardTitle>
                    <CardDescription>OAuth session. We never store your password - only a short-lived session token.</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        layout="inline"
                        title={
                            <span className="inline-flex items-center gap-1.5">
                                <KeyRoundIcon className="size-3.5 text-muted-foreground" /> Yostar OAuth
                            </span>
                        }
                        description="Authenticated via your in-game email verification code."
                        control={
                            <Badge variant="success">
                                <CheckIcon className="size-3" /> Active
                            </Badge>
                        }
                    />
                    <SettingRow
                        title="Sign out of this browser"
                        description="Ends the current session. Your data stays - just sign back in to re-access it."
                        control={
                            <Button variant="outline" size="sm" onClick={onSignOut} disabled={signingOut} loading={signingOut}>
                                <LogOutIcon className="size-3.5" />
                                Sign out
                            </Button>
                        }
                    />
                </CardContent>
            </Card>
        </div>
    );
}
