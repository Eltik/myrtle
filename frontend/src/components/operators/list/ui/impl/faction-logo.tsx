import Image from "next/image";
import { cn } from "~/lib/utils";

interface FactionLogoProps {
    nationId?: string | null;
    teamId?: string | null;
    groupId?: string | null;
    size?: number;
    className?: string;
}

export function FactionLogo({ nationId, teamId, groupId, size = 24, className }: FactionLogoProps) {
    const logoId = nationId && nationId.length > 0 ? nationId : teamId && teamId.length > 0 ? teamId : groupId && groupId.length > 0 ? groupId : "rhodes";
    const alt = String(nationId && nationId.length > 0 ? nationId : teamId && teamId.length > 0 ? teamId : groupId && groupId.length > 0 ? groupId : "Rhodes Island");

    return <Image alt={alt} className={cn("icon-theme-aware", className)} height={size} src={`/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${logoId}.png`} width={size} />;
}
