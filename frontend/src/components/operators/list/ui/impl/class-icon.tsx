import Image from "next/image";
import { CLASS_DISPLAY, CLASS_ICON } from "../../constants";

interface ClassIconProps {
    profession: string;
    size?: number;
    className?: string;
}

export function ClassIcon({ profession, size = 24, className }: ClassIconProps) {
    const iconName = CLASS_ICON[profession] ?? profession.toLowerCase();
    const displayName = CLASS_DISPLAY[profession] ?? profession;

    return <Image alt={displayName} className={className} height={size} src={`/api/cdn/upk/arts/ui/[uc]charcommon/icon_profession_${iconName}.png`} width={size} />;
}
