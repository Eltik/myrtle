import { useState } from "react";
import { assetURL } from "#/lib/api/stages";

/**
 * Renders a backend-resolved art path (stage map preview or event banner). The
 * backend verifies the file exists on disk before emitting the path, so this is
 * a single request; on the rare miss it hides and reveals the parent's gradient
 * placeholder.
 */
export function StagePreview({ src, className }: { src: string; className?: string }) {
    const [failed, setFailed] = useState(false);
    if (failed) return null;

    return <img src={assetURL(src)} alt="" loading="lazy" decoding="async" onError={() => setFailed(true)} className={className} style={{ position: "absolute", inset: 0, height: "100%", width: "100%", objectFit: "cover", display: "block" }} />;
}
