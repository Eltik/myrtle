import { AdminKicker, Button, Card, CardContent, CardHeader, CardTitle } from "frontend";

export function SectionHeadings() {
    return (
        <div className="flex flex-col gap-5">
            <div className="flex flex-col gap-1.5">
                <AdminKicker>Overview</AdminKicker>
                <h2 className="font-semibold text-[22px] leading-tight tracking-[-0.02em]">Dashboard</h2>
            </div>
            <div className="flex flex-col gap-1.5">
                <AdminKicker>Manage · official tier lists</AdminKicker>
                <h2 className="font-semibold text-[22px] leading-tight tracking-[-0.02em]">New official tier list</h2>
            </div>
            <div className="flex flex-col gap-1.5">
                <AdminKicker>Operate</AdminKicker>
                <h2 className="font-semibold text-[22px] leading-tight tracking-[-0.02em]">Health &amp; cache</h2>
            </div>
        </div>
    );
}

export function AbovePanelTitle() {
    return (
        <Card className="max-w-md">
            <CardHeader>
                <AdminKicker>Manage · flair</AdminKicker>
                <CardTitle className="text-[18px] tracking-[-0.01em]">Change flair on /endgame-dps</CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
                <p className="text-[12.5px] text-muted-foreground leading-[1.55]">Flairs colour the list's rail entry and its badge on the browse page. Only tier_list_admin and above can reassign one.</p>
                <div className="mt-3.5 flex items-center gap-2">
                    <Button variant="outline" size="sm">
                        Cancel
                    </Button>
                    <Button size="sm">Save</Button>
                </div>
            </CardContent>
        </Card>
    );
}

export function AboveMetricGroup() {
    return (
        <div className="flex flex-col gap-2">
            <AdminKicker>Live · last 24 hours</AdminKicker>
            <div className="flex flex-wrap items-baseline gap-x-6 gap-y-2">
                <span className="font-bold text-[26px] tabular-nums leading-none tracking-[-0.02em]">
                    41,633 <span className="font-medium font-mono text-[12px] text-muted-foreground">placements</span>
                </span>
                <span className="font-bold text-[26px] tabular-nums leading-none tracking-[-0.02em]">
                    18,412 <span className="font-medium font-mono text-[12px] text-muted-foreground">doctors</span>
                </span>
            </div>
        </div>
    );
}
