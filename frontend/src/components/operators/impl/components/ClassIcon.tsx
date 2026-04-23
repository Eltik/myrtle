import { env } from "#/env";
import { formatProfession } from "#/lib/utils";

export function ClassIcon({ profession, size = 20 }: { profession: string; size?: number }) {
    const base = env.VITE_BACKEND_URL ?? "";
    const src = `${base}/api/assets/textures/arts/ui/%5Buc%5Dcharcommon/icon_profession_${profession.toLowerCase()}.png`;
    return <img alt={formatProfession(profession)} src={src} width={size} height={size} />;
}
