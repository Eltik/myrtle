import { Link } from "@tanstack/react-router";
import { CheckIcon, KeyRoundIcon, Link2OffIcon, LogOutIcon, MailIcon, RefreshCwIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IUserProfile } from "#/types/user";
import type { messages } from "./DataPanel.messages";
import type { messages as profileMessages } from "./ProfilePanel.messages";
import { SettingRow } from "./SettingsShell";

interface IDataPanelProps {
    user: IUserProfile;
    onResync: () => void;
    syncing: boolean;
    onSignOut: () => void;
    signingOut: boolean;
    onDisconnect: () => void;
    disconnecting: boolean;
}

export function DataPanel({ user, onResync, syncing, onSignOut, signingOut, onDisconnect, disconnecting }: IDataPanelProps) {
    // The re-sync button is shared with ProfilePanel, which declares its two labels.
    const t: TypedT<typeof messages & typeof profileMessages> = useT("settings");

    return (
        <div className="flex flex-col gap-4">
            <Card>
                <CardHeader>
                    <CardTitle>{t("data.sync.title")}</CardTitle>
                    <CardDescription>{t("data.sync.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        title={t("data.sync.rowTitle")}
                        description={t("data.sync.rowDesc")}
                        control={
                            <Button size="sm" onClick={onResync} disabled={syncing} loading={syncing} className="w-full sm:w-auto">
                                <RefreshCwIcon className="size-3.5" />
                                {syncing ? t("resync.pending") : t("resync.now")}
                            </Button>
                        }
                    />
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>{t("data.yourData.title")}</CardTitle>
                    <CardDescription>{t("data.yourData.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        title={t("data.viewProfile.title")}
                        description={t("data.viewProfile.desc")}
                        control={
                            <Button variant="outline" size="sm" render={<Link to="/user/$id" params={{ id: user.uid }} />}>
                                {t("data.viewProfile.action")}
                            </Button>
                        }
                    />
                    <SettingRow
                        title={t("data.export.title")}
                        description={t("data.export.desc")}
                        control={
                            // biome-ignore lint/a11y/useAnchorContent: anchor children are slotted in by Button via render prop
                            <Button variant="outline" size="sm" render={<a href="mailto:privacy@myrtle.moe?subject=Data%20export%20request" />}>
                                <MailIcon className="size-3.5" />
                                {t("data.export.action")}
                            </Button>
                        }
                    />
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>{t("data.linked.title")}</CardTitle>
                    <CardDescription>{t("data.linked.desc")}</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow
                        layout="inline"
                        title={
                            <span className="inline-flex items-center gap-1.5">
                                <KeyRoundIcon className="size-3.5 text-muted-foreground" /> {t("data.oauth.title")}
                            </span>
                        }
                        description={t("data.oauth.desc")}
                        control={
                            <Badge variant="success">
                                <CheckIcon className="size-3" /> {t("data.oauth.active")}
                            </Badge>
                        }
                    />
                    <SettingRow
                        title={t("data.signOut.title")}
                        description={t("data.signOut.desc")}
                        control={
                            <Button variant="outline" size="sm" onClick={onSignOut} disabled={signingOut} loading={signingOut}>
                                <LogOutIcon className="size-3.5" />
                                {t("data.signOut.action")}
                            </Button>
                        }
                    />
                    <SettingRow
                        title={t("data.disconnect.title")}
                        description={t("data.disconnect.desc")}
                        control={
                            <Button variant="destructive-outline" size="sm" onClick={onDisconnect} disabled={disconnecting} loading={disconnecting}>
                                <Link2OffIcon className="size-3.5" />
                                {t("data.disconnect.action")}
                            </Button>
                        }
                    />
                </CardContent>
            </Card>
        </div>
    );
}
