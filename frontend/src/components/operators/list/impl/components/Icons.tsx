import type { SyntheticEvent } from "react";
import { assetURL } from "#/lib/assets";
import { cn, formatGroupId, formatProfession, formatSubProfession, formatTeamId } from "#/lib/utils";

/** A missing icon file (a new faction's logo, say) made the browser paint the alt text, which a
 *  16px box stacked one letter per line. The box keeps its size and clips; the image goes
 *  transparent, so the alt still names it to a screen reader. */
function hideBroken(event: SyntheticEvent<HTMLImageElement>) {
    event.currentTarget.style.opacity = "0";
}

export function ClassIcon({ profession, size = 20, className }: { profession: string; size?: number; className?: string }) {
    const src = assetURL(`/textures/arts/ui/%5Buc%5Dcharcommon/icon_profession_${profession.toLowerCase()}.png`);
    return <img alt={formatProfession(profession)} src={src} width={size} height={size} className={cn("icon-theme-aware overflow-hidden", className)} onError={hideBroken} />;
}

export function SubProfessionIcon({ subProfession, size = 20, className }: { subProfession: string; size?: number; className?: string }) {
    const src = assetURL(`/textures/spritepack/ui_sub_profession_icon_hub_h2_0/sub_${subProfession.toLowerCase()}_icon.png`);
    return <img alt={formatSubProfession(subProfession)} src={src} width={size} height={size} className={cn("icon-theme-aware overflow-hidden", className)} onError={hideBroken} />;
}

export function TeamIcon({ teamId, size = 20 }: { teamId: string; size?: number }) {
    const src = assetURL(`/textures/spritepack/ui_team_icon_h2_0/org_${teamId.toLowerCase()}_tiny.png`);
    return <img alt={formatTeamId(teamId)} src={src} width={size} height={size} className="icon-theme-aware overflow-hidden" onError={hideBroken} />;
}

export function CampIcon({ groupId, size = 20, className }: { groupId: string; size?: number; className?: string }) {
    const src = assetURL(`/textures/spritepack/ui_camp_logo_0/logo_${groupId.toLowerCase()}.png`);
    return <img alt={formatGroupId(groupId)} src={src} width={size} height={size} className={cn("icon-theme-aware overflow-hidden", className)} onError={hideBroken} />;
}
