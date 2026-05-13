import { useState } from "react";
import { getAvatarById } from "#/lib/utils";

interface IOperatorAvatarProps {
    charId?: string | null;
    name: string;
    className?: string;
}

/**
 * Inner content for an operator avatar chip. Renders an <img> that covers its
 * parent container, falling back to the operator's initial letter on 404 or
 * when no charId is supplied. Drop it inside any sized/styled wrapper
 * (e.g. .op-chip, .tl-chip-initials, .drawer-initials).
 */
export function OperatorAvatar({ charId, name, className }: IOperatorAvatarProps) {
    const [failed, setFailed] = useState(false);

    if (!charId || failed) {
        return <>{name.charAt(0).toUpperCase()}</>;
    }

    return <img src={getAvatarById(charId)} alt="" aria-hidden="true" loading="lazy" decoding="async" className={className ?? "block h-full w-full rounded-[inherit] object-cover"} onError={() => setFailed(true)} />;
}
