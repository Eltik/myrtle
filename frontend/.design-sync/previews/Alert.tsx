import { Alert, AlertAction, AlertDescription, AlertTitle, Button } from "frontend";
import { CircleCheckIcon, EyeIcon, InfoIcon, ServerCrashIcon, TriangleAlertIcon } from "lucide-react";

export const Variants = () => (
    <div className="flex w-full max-w-xl flex-col gap-3">
        <Alert>
            <InfoIcon />
            <AlertTitle>Roster synced 4 minutes ago</AlertTitle>
            <AlertDescription>Next automatic sync runs when you open the site after 6 hours.</AlertDescription>
        </Alert>
        <Alert variant="info">
            <EyeIcon />
            <AlertTitle>What's public, and what isn't</AlertTitle>
            <AlertDescription>Public profiles show your nickname, UID, roster, and leaderboard rankings. Tokens, settings, and email stay private.</AlertDescription>
        </Alert>
        <Alert variant="success">
            <CircleCheckIcon />
            <AlertTitle>Depot import complete</AlertTitle>
            <AlertDescription>1,284 items across 96 material types were written to your depot.</AlertDescription>
        </Alert>
        <Alert variant="warning">
            <TriangleAlertIcon />
            <AlertTitle>CN-only operator</AlertTitle>
            <AlertDescription>Mlynar's Module Y is not on Global yet. DPS numbers below use CN data.</AlertDescription>
        </Alert>
        <Alert variant="error">
            <ServerCrashIcon />
            <AlertTitle>Yostar sign-in failed</AlertTitle>
            <AlertDescription>The verification code expired before it was submitted. Request a new one and try again.</AlertDescription>
        </Alert>
    </div>
);

export const Info = () => (
    <Alert className="max-w-xl" variant="info">
        <EyeIcon />
        <AlertTitle>What's public, and what isn't</AlertTitle>
        <AlertDescription>Public profiles show your nickname, UID, roster, account scores, and leaderboard rankings. Authentication tokens, settings, saved DPS configs, and email are always private.</AlertDescription>
    </Alert>
);

export const WithAction = () => (
    <Alert className="max-w-xl" variant="error">
        <ServerCrashIcon />
        <AlertTitle>Couldn't reach the stage index</AlertTitle>
        <AlertDescription>Stage drop rates for Chapter 8 are stale. The planner is using the cached copy from 2 days ago.</AlertDescription>
        <AlertAction>
            <Button size="sm" variant="outline">
                Retry
            </Button>
        </AlertAction>
    </Alert>
);

export const TitleOnly = () => (
    <div className="flex w-full max-w-xl flex-col gap-3">
        <Alert variant="success">
            <CircleCheckIcon />
            <AlertTitle>Plan saved — 12 stages, 4,180 sanity</AlertTitle>
        </Alert>
        <Alert variant="warning">
            <TriangleAlertIcon />
            <AlertTitle>Sanity budget exceeded by 340</AlertTitle>
        </Alert>
        <Alert>
            <AlertTitle>Recruitment pool updated for the 2026-08 banner</AlertTitle>
        </Alert>
    </div>
);
