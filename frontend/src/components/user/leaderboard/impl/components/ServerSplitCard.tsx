import type { IServerShare } from "#/lib/api/user";
import { SERVERS } from "../constants";
import { ServerTag } from "./ServerTag";

const SERVER_BAR_COLORS: Record<string, string> = {
    EN: "#4f74e0",
    JP: "#e04f74",
    CN: "#e0a04f",
    KR: "#4fb8b2",
    TW: "#9e7ad4",
};

export function ServerSplitCard({ shares }: { shares: IServerShare[] }) {
    const total = shares.reduce((acc, s) => acc + s.players, 0);
    const byServer = new Map<string, number>(shares.map((s) => [s.server.toUpperCase(), s.players]));

    return (
        <aside className="rounded-xl border border-border bg-card p-4 shadow-[0_1px_2px_rgb(0_0_0/0.04)]">
            <div className="mb-2.5 flex items-center justify-between">
                <span className="font-sans text-[13.5px] font-semibold leading-tight tracking-tight text-foreground">Server share · top 250</span>
            </div>
            <div className="flex h-2 overflow-hidden rounded-full bg-muted">
                {total === 0 ? (
                    <span className="block size-full bg-muted" />
                ) : (
                    SERVERS.map((code) => {
                        const count = byServer.get(code) ?? 0;
                        const pct = (count / total) * 100;
                        return <span key={code} title={`${code}: ${count}`} className="block h-full" style={{ width: `${pct}%`, background: SERVER_BAR_COLORS[code] }} />;
                    })
                )}
            </div>
            <div className="mt-3 grid grid-cols-5 gap-1.5">
                {SERVERS.map((code) => {
                    const count = byServer.get(code) ?? 0;
                    const pct = total === 0 ? 0 : Math.round((count / total) * 100);
                    return (
                        <div key={code} className="flex flex-col items-center gap-1">
                            <ServerTag server={code} />
                            <span className="font-mono text-[11px] leading-none text-muted-foreground tabular-nums">{pct}%</span>
                        </div>
                    );
                })}
            </div>
        </aside>
    );
}
