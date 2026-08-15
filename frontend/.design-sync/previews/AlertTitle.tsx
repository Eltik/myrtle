import { Alert, AlertDescription, AlertTitle } from "frontend";
import { InfoIcon, SparklesIcon, TriangleAlertIcon } from "lucide-react";

export const Default = () => (
    <Alert className="max-w-xl" variant="info">
        <InfoIcon />
        <AlertTitle>Chapter 8 — Roaring Flare is now farmable</AlertTitle>
        <AlertDescription>8-16 is the most efficient source of Crystalline Component at 0.42 per sanity.</AlertDescription>
    </Alert>
);

export const TitleOnly = () => (
    <Alert className="max-w-xl" variant="success">
        <SparklesIcon />
        <AlertTitle>Mlynar promoted to Elite 2, Level 90</AlertTitle>
    </Alert>
);

export const LongTitle = () => (
    <Alert className="max-w-md" variant="warning">
        <TriangleAlertIcon />
        <AlertTitle>Your saved DPS configuration references three operators that are no longer in your roster</AlertTitle>
        <AlertDescription>Skadi the Corrupting Heart, Ceobe, and Specter the Unchained were removed on the last sync.</AlertDescription>
    </Alert>
);

export const WithoutIcon = () => (
    <Alert className="max-w-xl">
        <AlertTitle>Leaderboard snapshot</AlertTitle>
        <AlertDescription>Rankings are recomputed every 6 hours. Your last submitted account score was 84,120.</AlertDescription>
    </Alert>
);
