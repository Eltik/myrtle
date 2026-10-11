import { useState } from "react";
import { getAvatarById } from "#/lib/utils";

interface IOperatorAvatarProps {
    charId?: string | null;
    name: string;
    className?: string;
    server?: string;
}

/**
 * Fills a sized wrapper (e.g. `.op-chip`) with the avatar, or the operator's
 * initial on a 404 or with no charId.
 */
export function OperatorAvatar({ charId, name, className, server }: IOperatorAvatarProps) {
    const [failed, setFailed] = useState(false);

    if (!charId || failed) {
        return <>{name.charAt(0).toUpperCase()}</>;
    }

    return <img src={getAvatarById(charId, server)} alt="" aria-hidden="true" loading="lazy" decoding="async" draggable={false} onDragStart={(e) => e.preventDefault()} className={className ?? "block h-full w-full rounded-[inherit] object-cover"} onError={() => setFailed(true)} />;
}
