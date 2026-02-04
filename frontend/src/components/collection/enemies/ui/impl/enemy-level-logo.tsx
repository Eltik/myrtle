import Image from "next/image";
import { cn } from "~/lib/utils";
import type { EnemyLevel } from "~/types/api";

interface EnemyLevelLogoProps {
    level?: EnemyLevel | null;
    size?: number;
    className?: string;
}

export function EnemyLevelLogo({ level, size = 24, className }: EnemyLevelLogoProps) {
    const logoId = level === "BOSS" ? "enemy_handbook_41.png" : level === "ELITE" ? "enemy_handbook_45.png" : "";
    return <Image alt={level?.toString() ?? ""} className={cn("icon-theme-aware", className)} height={size} src={`/api/cdn/upk/ui/stage/enemyhandbook/enemy_handbook_state/sprites/${logoId}`} width={size} />;
}
