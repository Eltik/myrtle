import { SERVER_TINTS } from "../constants";

export function ServerTag({ server, className }: { server: string; className?: string }) {
    const code = server.toUpperCase();
    const tint = SERVER_TINTS[code];
    const style = tint ? { color: tint.fg, background: tint.bg } : undefined;
    return (
        <span className={`inline-flex h-5 items-center rounded font-mono text-[10px] font-semibold uppercase leading-none tracking-[0.08em] ${tint ? "" : "bg-muted text-muted-foreground"} ${className ?? ""}`.trim()} style={{ ...style, padding: "0 6px" }}>
            {code}
        </span>
    );
}
