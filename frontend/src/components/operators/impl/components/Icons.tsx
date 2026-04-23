import { env } from "#/env";
import { formatGroupId, formatProfession, formatSubProfession, formatTeamId } from "#/lib/utils";

export function ClassIcon({ profession, size = 20 }: { profession: string; size?: number }) {
    const base = env.VITE_BACKEND_URL ?? "";
    const src = `${base}/api/assets/textures/arts/ui/%5Buc%5Dcharcommon/icon_profession_${profession.toLowerCase()}.png`;
    return <img alt={formatProfession(profession)} src={src} width={size} height={size} />;
}

export function SubProfessionIcon({ subProfession, size = 20 }: { subProfession: string; size?: number }) {
    const base = env.VITE_BACKEND_URL ?? "";
    const src = `${base}/api/assets/textures/spritepack/ui_sub_profession_icon_hub_h2_0/sub_${subProfession.toLowerCase()}_icon.png`;
    return <img alt={formatSubProfession(subProfession)} src={src} width={size} height={size} />;
}

export function TeamIcon({ teamId, size = 20 }: { teamId: string; size?: number }) {
    const base = env.VITE_BACKEND_URL ?? "";
    const src = `${base}/api/assets/textures/spritepack/ui_team_icon_h2_0/org_${teamId.toLowerCase()}_tiny.png`;
    return <img alt={formatTeamId(teamId)} src={src} width={size} height={size} />;
}

export function CampIcon({ groupId, size = 20 }: { groupId: string; size?: number }) {
    const base = env.VITE_BACKEND_URL ?? "";
    const src = `${base}/api/assets/textures/spritepack/ui_camp_logo_0/logo_${groupId.toLowerCase()}.png`;
    return <img alt={formatGroupId(groupId)} src={src} width={size} height={size} />;
}
