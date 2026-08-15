import { Alert, AlertDescription, AlertTitle } from "frontend";
import { ShieldIcon, TriangleAlertIcon, WrenchIcon } from "lucide-react";

export const Default = () => (
    <Alert className="max-w-xl" variant="info">
        <WrenchIcon />
        <AlertTitle>Base optimizer uses your live depot</AlertTitle>
        <AlertDescription>Room assignments are recalculated from the roster and trading-post orders captured at your last sync.</AlertDescription>
    </Alert>
);

export const WithoutTitle = () => (
    <Alert className="max-w-xl">
        <ShieldIcon strokeWidth={1.8} />
        <AlertDescription className="text-[15px] leading-[1.55]">By accessing or using Myrtle, you agree to be bound by these Terms of Service. Please read them carefully before continuing.</AlertDescription>
    </Alert>
);

export const RichContent = () => (
    <Alert className="max-w-xl" variant="warning">
        <TriangleAlertIcon />
        <AlertTitle>3 stages were dropped from this plan</AlertTitle>
        <AlertDescription>
            <p>These stages are not open on Global yet, so their drops were excluded from the sanity estimate:</p>
            <ul className="list-inside list-disc space-y-1 font-mono text-xs">
                <li>S4-1 — Oriron Block</li>
                <li>CE-6 — LMD</li>
                <li>AP-5 — Carbon Brick</li>
            </ul>
        </AlertDescription>
    </Alert>
);

export const MultiParagraph = () => (
    <Alert className="max-w-xl" variant="error">
        <TriangleAlertIcon />
        <AlertTitle>Account deletion is permanent</AlertTitle>
        <AlertDescription>
            <p>Deleting your account removes your roster snapshot, saved DPS configurations, tier lists, and planner state.</p>
            <p>Leaderboard entries are removed within 24 hours. This cannot be undone.</p>
        </AlertDescription>
    </Alert>
);
