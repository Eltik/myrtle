import { GRANT_LEVELS, levelVariant } from "#/components/admin/shell/model";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import type { TierListPermissionLevel } from "#/lib/api/admin";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./Users.messages";

type UsersT = TypedT<typeof messages>;

interface IGrantRowProps {
    label: string;
    by: string;
    level: string;
    levelLabel: string;
    onRevoke: (() => void) | undefined;
    revoking: boolean;
    revokeLabel: string;
}

export function GrantRow({ label, by, level, levelLabel, onRevoke, revoking, revokeLabel }: IGrantRowProps): React.ReactElement {
    return (
        <div className="flex items-center gap-2.5 rounded-lg border border-border py-[7px] pr-2 pl-3">
            <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                <span className="truncate text-[13.5px]" title={label}>
                    {label}
                </span>
                <span className="text-[12px] text-muted-foreground">{by}</span>
            </div>
            <Badge variant={levelVariant(level)}>{levelLabel}</Badge>
            {onRevoke ? (
                <Button size="xs" variant="ghost" disabled={revoking} onClick={onRevoke}>
                    {revokeLabel}
                </Button>
            ) : null}
        </div>
    );
}

interface IGrantPickerProps {
    level: TierListPermissionLevel;
    onLevel: (level: TierListPermissionLevel) => void;
    levelLabel: (level: string) => string;
    options: { id: string; label: string }[];
    loading: boolean;
    emptyLabel: string;
    busy: boolean;
    onPick: (id: string, label: string) => void;
}

/**
 * The "+ Grant" box. The design grants Edit only; a level row (default Edit)
 * keeps the old Permissions screen's ability to grant View, Publish or Admin.
 */
export function GrantPicker({ level, onLevel, levelLabel, options, loading, emptyLabel, busy, onPick }: IGrantPickerProps): React.ReactElement {
    const t: UsersT = useT("admin");
    return (
        <div className="flex flex-col gap-2 rounded-lg bg-muted p-2.5">
            <div className="flex flex-wrap items-center gap-1.5">
                <span className="mr-1 text-[12.5px] text-muted-foreground">{t("users.sheet.picker.level")}</span>
                {GRANT_LEVELS.map((l) => (
                    <Button key={l} size="xs" variant={l === level ? "secondary" : "ghost"} aria-pressed={l === level} className={cn(l === level && "bg-background")} onClick={() => onLevel(l)}>
                        {levelLabel(l)}
                    </Button>
                ))}
            </div>
            <span className="text-[12.5px] text-muted-foreground">{t("users.sheet.picker.on", { level: levelLabel(level) })}</span>
            {loading ? (
                <span className="text-[12.5px] text-muted-foreground">{t("users.sheet.picker.loading")}</span>
            ) : options.length === 0 ? (
                <span className="text-[12.5px] text-muted-foreground">{emptyLabel}</span>
            ) : (
                <div className="flex flex-wrap gap-1.5">
                    {options.map((o) => (
                        <Button key={o.id} size="xs" variant="outline" disabled={busy} onClick={() => onPick(o.id, o.label)}>
                            {o.label}
                        </Button>
                    ))}
                </div>
            )}
        </div>
    );
}
