import { Alert, AlertAction, AlertDescription, AlertTitle, Button } from "frontend";
import { RefreshCwIcon, ServerCrashIcon, SparklesIcon } from "lucide-react";

export const Default = () => (
    <Alert className="max-w-xl" variant="error">
        <ServerCrashIcon />
        <AlertTitle>Roster sync failed</AlertTitle>
        <AlertDescription>Yostar returned a 502 while fetching your operator list. Nothing was overwritten.</AlertDescription>
        <AlertAction>
            <Button size="sm" variant="outline">
                <RefreshCwIcon />
                Retry
            </Button>
        </AlertAction>
    </Alert>
);

export const TwoActions = () => (
    <Alert className="max-w-2xl" variant="warning">
        <SparklesIcon />
        <AlertTitle>A newer CN gamedata build is available</AlertTitle>
        <AlertDescription>Build 2026-08-12 adds 4 operators and reprices 96 base buffs. Global values are unaffected.</AlertDescription>
        <AlertAction>
            <Button size="sm" variant="ghost">
                Dismiss
            </Button>
            <Button size="sm">Switch to CN</Button>
        </AlertAction>
    </Alert>
);

export const WithoutIcon = () => (
    <Alert className="max-w-xl" variant="info">
        <AlertTitle>Public profile is off</AlertTitle>
        <AlertDescription>Other doctors can't open your roster or see your leaderboard placement.</AlertDescription>
        <AlertAction>
            <Button size="sm" variant="outline">
                Open privacy settings
            </Button>
        </AlertAction>
    </Alert>
);

export const TitleOnly = () => (
    <Alert className="max-w-xl" variant="success">
        <SparklesIcon />
        <AlertTitle>Depot import complete — 96 material types</AlertTitle>
        <AlertAction>
            <Button size="sm" variant="outline">
                View depot
            </Button>
        </AlertAction>
    </Alert>
);
