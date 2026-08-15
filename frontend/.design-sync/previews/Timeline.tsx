import { Card, CardContent, CardHeader, CardTitle, Timeline } from "frontend";

const quickLinks = [
    { when: "manage", what: "Tier list permissions", who: "View / Edit / Publish / Admin" },
    { when: "manage", what: "Official tier lists", who: "Flair-tagged lists in the Official rail" },
    { when: "manage", what: "Operator notes", who: "Community guidance per operator" },
    { when: "operate", what: "Health & cache", who: "Redis + Postgres probes" },
    { when: "operate", what: "Audit log", who: "Permission grants + note edits" },
];

const auditTrail = [
    { when: "4m ago", what: "Edited `pros` on Mlynar", who: "Kyostinv · super_admin" },
    { when: "1h ago", what: "Published version 14 of /endgame-dps", who: "Dr. Reisen · tier_list_admin" },
    { when: "3h ago", what: "Cleared `trivia` on Texas the Omertosa", who: "Ceylonade · tier_list_editor" },
    { when: "yesterday", what: "Granted Edit on /cc12-sandbox-picks", who: "Kyostinv · super_admin" },
];

const versionHistory = [
    { when: "2024-05-15 · v14", what: "Muelsyse moved S → S+, Skadi the Corrupting Heart added to A", who: "Dr. Reisen" },
    { when: "2024-04-02 · v13", what: "Rebalanced the A tier after the CC#12 ruleset", who: "Dr. Reisen" },
    { when: "2024-02-18 · v12", what: "Initial public snapshot — 96 operators across 6 tiers", who: "Kyostinv", muted: true },
    { when: "2024-02-11 · draft", what: "List created as `list_type = official`", who: "Kyostinv", muted: true },
];

export function QuickLinks() {
    return (
        <Card className="max-w-sm">
            <CardHeader>
                <CardTitle className="text-sm">Quick links</CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
                <Timeline items={quickLinks} />
            </CardContent>
        </Card>
    );
}

export function AuditTrail() {
    return (
        <div className="max-w-md">
            <Timeline items={auditTrail} />
        </div>
    );
}

export function VersionHistoryWithMuted() {
    return (
        <Card className="max-w-lg">
            <CardHeader>
                <CardTitle className="text-sm">Version history · /endgame-dps</CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
                <Timeline items={versionHistory} />
            </CardContent>
        </Card>
    );
}

export function SingleEntry() {
    return (
        <div className="max-w-md">
            <Timeline items={[{ when: "just now", what: "Draft created — add tiers in the editor", who: "Kyostinv · super_admin" }]} />
        </div>
    );
}
