import { Link } from "@tanstack/react-router";
import { CheckIcon, KeyRoundIcon, Link2OffIcon, LogOutIcon, MailIcon, RefreshCwIcon, UserRoundIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { formatServerWithPublisher } from "#/lib/auth/login";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { formatRelativeShort } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import type { messages as dataMessages } from "./DataPanel.messages";
import type { messages as profileMessages } from "./ProfilePanel.messages";
import { SettingRow } from "./SettingsShell";

interface IAccountPanelProps {
    user: IUserProfile;
    onResync: () => void;
    syncing: boolean;
    onSignOut: () => void;
    signingOut: boolean;
    onDisconnect: () => void;
    disconnecting: boolean;
}

/**
 * The single account section, replacing the old "Profile" and "Account & data"
 * pair.
 *
 * Both of those carried a re-sync button wired to the same mutation, and
 * "Profile" restated in read-only text boxes what the identity header one card
 * above it already showed - nickname, level and server, with server appearing
 * twice on the SAME panel. None of those boxes was editable; they were `<Input
 * readOnly>` with no `onChange`, which reads as "type here" and does nothing.
 *
 * What survives is the identity header (with level now a badge rather than a
 * text box), ONE re-sync, the assistant operator (the one game-synced fact the
 * public profile does not show), and the data and linked-account cards.
 */
export function AccountPanel({ user, onResync, syncing, onSignOut, signingOut, onDisconnect, disconnecting }: IAccountPanelProps) {
    // The re-sync labels and the data/linked-account copy are declared by the
    // two panels this one replaces; the keys are unchanged so no translation is
    // invalidated by the merge.
    const t: TypedT<typeof profileMessages & typeof dataMessages> = useT("settings");
    const display = user.nickname ?? "Doctor";
    const nickNum = user.nick_number ? `#${user.nick_number}` : "";

    return (
        <div className="flex flex-col gap-4">
            <Card>
                <div className="flex flex-col gap-4 p-4 sm:grid sm:grid-cols-[auto_1fr_auto] sm:items-center sm:gap-5 sm:p-6">
                    <div className="flex items-center gap-4 sm:contents">
                        <div className="relative size-14 shrink-0 overflow-hidden rounded-full border-2 border-primary/30 bg-linear-to-br from-primary to-primary/60 text-primary-foreground sm:size-16">
                            <span className="absolute inset-0 flex items-center justify-center font-bold text-[20px] sm:text-[22px]">
                                <OperatorAvatar charId={user.secretary_skin_id ?? user.secretary} name={display} />
                            </span>
                        </div>
                        <div className="flex min-w-0 flex-col gap-1.5">
                            <div className="flex min-w-0 items-baseline gap-1.5">
                                <span className="truncate font-semibold text-[17px] text-foreground leading-tight tracking-[-0.01em] sm:text-[18px]">{display}</span>
                                {nickNum ? <span className="font-mono text-[12px] text-muted-foreground">{nickNum}</span> : null}
                            </div>
                            <div className="flex flex-wrap items-center gap-1.5">
                                <Badge variant="outline" size="sm">
                                    {t("profile.uid", { uid: user.uid })}
                                </Badge>
                                {user.level != null ? (
                                    <Badge variant="outline" size="sm">
                                        {t("profile.level.value", { level: user.level })}
                                    </Badge>
                                ) : null}
                                <Badge variant="outline" size="sm">
                                    {formatServerWithPublisher(user.server)}
                                </Badge>
                                <Badge variant="success" size="sm">
                                    <CheckIcon className="size-3" />
                                    {t("profile.synced", { when: formatRelativeShort(user.updated_at) })}
                                </Badge>
                            </div>
                        </div>
                    </div>
                </div>
            </Card>

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
                    <SettingRow
                        title={t("profile.assistant.title")}
                        description={t("profile.assistant.desc")}
                        control={
                            <div className="flex items-center gap-2.5">
                                <div className="inline-flex size-9 items-center justify-center overflow-hidden rounded-lg border border-border bg-[color-mix(in_srgb,var(--primary)_14%,var(--card))] text-muted-foreground">
                                    {user.secretary ? <OperatorAvatar charId={user.secretary_skin_id ?? user.secretary} name={display} /> : <UserRoundIcon className="size-4" />}
                                </div>
                                <span className="font-mono text-[13px] text-muted-foreground">{user.secretary ?? "-"}</span>
                            </div>
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
