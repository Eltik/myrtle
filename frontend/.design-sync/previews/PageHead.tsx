import { Button, HCode, PageHead } from "frontend";
import { ExternalLinkIcon, PlusIcon, RefreshCwIcon } from "lucide-react";

export function OverviewWithActions() {
    return (
        <PageHead
            kicker="Overview"
            title="Dashboard"
            sub={
                <>
                    Service-level signals from the Rust backend — surfaced verbatim from <HCode>GET /admin/stats</HCode> and <HCode>/health</HCode>.
                </>
            }
            action={
                <>
                    <Button variant="outline" size="sm">
                        <RefreshCwIcon />
                        Refresh
                    </Button>
                    <Button variant="outline" size="sm">
                        <ExternalLinkIcon />
                        View public stats
                    </Button>
                </>
            }
        />
    );
}

export function ManageWithPrimaryAction() {
    return (
        <PageHead
            kicker="Manage"
            title="Official tier lists"
            sub={
                <>
                    Tier lists with <HCode>list_type = "official"</HCode> surface in the Official rail. Same editor as community; tier_list_admin writes directly without per-list grants.
                </>
            }
            action={
                <>
                    <Button variant="outline" size="sm">
                        <ExternalLinkIcon />
                        View public rail
                    </Button>
                    <Button size="sm">
                        <PlusIcon />
                        New official list
                    </Button>
                </>
            }
        />
    );
}

export function WithoutSubtitle() {
    return <PageHead kicker="Operate" title="Settings" />;
}

export function LongSubtitleNoAction() {
    return (
        <PageHead
            kicker="Operate"
            title="Audit log"
            sub={
                <>
                    Append-only edit trail from <HCode>operator_notes_audit_log</HCode>. Each row was written when an admin saved an operator note. Permission grants and tier-list publishes don't yet emit audit rows in v3.
                </>
            }
        />
    );
}
