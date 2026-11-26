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
        // The initial is a picture of the name, not the name: the caller labels
        // the chip, and a lone "S" read aloud before it is noise.
        return <span aria-hidden="true">{name.charAt(0).toUpperCase()}</span>;
    }

    return <img src={getAvatarById(charId, server)} alt="" aria-hidden="true" loading="lazy" decoding="async" draggable={false} onDragStart={(e) => e.preventDefault()} className={className ?? "block h-full w-full rounded-[inherit] object-cover"} onError={() => setFailed(true)} />;
}
