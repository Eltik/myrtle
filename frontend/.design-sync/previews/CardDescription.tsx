import { Card, CardDescription, CardFooter, CardHeader, CardPanel, CardTitle } from "frontend";

export const BelowTitle = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Chapter 8 — Roaring Flare</CardTitle>
            <CardDescription>14 stages, 3 challenge modes. First-clear rewards include 2 Chip Catalysts.</CardDescription>
        </CardHeader>
        <CardPanel className="text-muted-foreground text-sm">
            Recommended average level: <span className="font-medium text-foreground">E2 40</span>
        </CardPanel>
    </Card>
);

export const AsEyebrow = () => (
    <Card className="max-w-xs">
        <CardHeader>
            <CardDescription className="font-mono text-xs uppercase tracking-widest">Headhunting</CardDescription>
            <CardTitle>Ines banner</CardTitle>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Pulls since 6★</span>
            <span className="font-mono tabular-nums">37</span>
        </CardPanel>
    </Card>
);

export const InFooter = () => (
    <Card className="max-w-sm">
        <CardHeader>
            <CardTitle>Bipolar Nanoflake</CardTitle>
            <CardDescription>T5 material · used by 42 skill masteries and 9 modules.</CardDescription>
        </CardHeader>
        <CardPanel className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Best stage</span>
            <span className="font-mono tabular-nums">CE-6 · 30 sanity</span>
        </CardPanel>
        <CardFooter>
            <CardDescription>Drop rates sampled from 12,480 community runs on Penguin Stats.</CardDescription>
        </CardFooter>
    </Card>
);
