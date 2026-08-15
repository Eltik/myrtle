import { Card, CardContent, CardDescription, CardHeader, CardTitle, HCode } from "frontend";

export function InlineInProse() {
    return (
        <p className="max-w-[64ch] text-[13.5px] text-muted-foreground leading-[1.6]">
            Service-level signals from the Rust backend — surfaced verbatim from <HCode>GET /admin/stats</HCode> and <HCode>/health</HCode>. Counts reflect the live in-memory dataset loaded from <HCode>GAME_DATA_DIR</HCode>, so any drift means the asset import is out of date.
        </p>
    );
}

export function EndpointReference() {
    return (
        <Card className="max-w-lg">
            <CardHeader>
                <CardTitle className="text-sm">Admin endpoints</CardTitle>
                <CardDescription className="text-xs">Every route below requires tier_list_admin or above.</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-2.5 pt-0">
                <div className="flex items-center justify-between gap-4 text-[13px]">
                    <HCode>GET /admin/stats</HCode>
                    <span className="text-[12px] text-muted-foreground">Role counts, roster totals, game-data snapshot</span>
                </div>
                <div className="flex items-center justify-between gap-4 text-[13px]">
                    <HCode>GET /health</HCode>
                    <span className="text-[12px] text-muted-foreground">Postgres + Redis probe, round-trip ms</span>
                </div>
                <div className="flex items-center justify-between gap-4 text-[13px]">
                    <HCode>POST /tier-lists</HCode>
                    <span className="text-[12px] text-muted-foreground">Creates a draft with list_type=official</span>
                </div>
            </CardContent>
        </Card>
    );
}

export function DatabaseIdentifiers() {
    return (
        <p className="max-w-[64ch] text-[13.5px] text-muted-foreground leading-[1.6]">
            Edits write to <HCode>operator_notes</HCode> and append a diff row to <HCode>operator_notes_audit_log</HCode>. Roles come from the <HCode>users.role</HCode> column; per-list grants live in <HCode>tier_list_permissions</HCode>.
        </p>
    );
}
