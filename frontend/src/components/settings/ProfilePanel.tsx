import { CheckIcon, RefreshCwIcon, UserRoundIcon } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { formatServerWithPublisher } from "#/lib/auth/login";
import { formatRelativeShort } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { SettingRow } from "./SettingsShell";

interface IProfilePanelProps {
    user: IUserProfile;
    onResync: () => void;
    syncing: boolean;
}

export function ProfilePanel({ user, onResync, syncing }: IProfilePanelProps) {
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
                                    UID {user.uid}
                                </Badge>
                                <Badge variant="outline" size="sm">
                                    {formatServerWithPublisher(user.server)}
                                </Badge>
                                <Badge variant="success" size="sm">
                                    <CheckIcon className="size-3" />
                                    Synced {formatRelativeShort(user.updated_at)}
                                </Badge>
                            </div>
                        </div>
                    </div>
                    <Button variant="outline" size="sm" onClick={onResync} disabled={syncing} loading={syncing} className="w-full sm:w-auto">
                        <RefreshCwIcon className="size-3.5" />
                        {syncing ? "Re-syncing…" : "Re-sync now"}
                    </Button>
                </div>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Game-synced info</CardTitle>
                    <CardDescription>Read-only. Pulled from Yostar when you sync - change it in-game.</CardDescription>
                </CardHeader>
                <CardContent className="pt-0">
                    <SettingRow title="Arknights nickname" description="Shown on your profile, leaderboard, and tier lists you publish." control={<Input value={`${display}${nickNum}`} readOnly className="w-full sm:w-65" />} />
                    <SettingRow title="Account level" description="Doctor level from the in-game profile." control={<Input value={user.level != null ? `Lv. ${user.level}` : "-"} readOnly className="w-full sm:w-30" />} />
                    <SettingRow title="Game server" description="Which Arknights server you're synced from. Re-link your account to change this." control={<Input value={formatServerWithPublisher(user.server)} readOnly className="w-full sm:w-55" />} />
                    <SettingRow
                        title="Assistant operator"
                        description="The operator displayed on your in-game and Myrtle profile."
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
        </div>
    );
}
