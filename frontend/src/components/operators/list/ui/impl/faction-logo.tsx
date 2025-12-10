import Image from "next/image";

interface FactionLogoProps {
    nationId?: string | null;
    teamId?: string | null;
    size?: number;
    className?: string;
}

export function FactionLogo({ nationId, teamId, size = 24, className }: FactionLogoProps) {
    const logoId = nationId ?? teamId ?? "rhodes";
    const alt = String(nationId ?? teamId ?? "Rhodes Island");

    return <Image alt={alt} className={className} height={size} src={`/api/cdn/upk/spritepack/ui_camp_logo_0/logo_${logoId}.png`} width={size} />;
}
