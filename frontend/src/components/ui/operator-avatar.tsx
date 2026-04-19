import { useState } from "react";
import { operatorAvatarUrl } from "#/lib/utils";

interface OperatorAvatarProps {
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
export function OperatorAvatar({ charId, name, className }: OperatorAvatarProps) {
    const src = operatorAvatarUrl(charId);
    const [failed, setFailed] = useState(false);

    if (!src || failed) {
        return <>{name.charAt(0).toUpperCase()}</>;
    }

    return <img src={src} alt="" aria-hidden="true" loading="lazy" decoding="async" className={className ?? "op-avatar-img"} onError={() => setFailed(true)} />;
}
