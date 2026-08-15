import { Badge, Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow } from "frontend";

const DOCTORS = [
    { rank: 1, name: "Kal'tsit Enjoyer", uid: "10023481", server: "EN", level: 120, completion: "98.4%" },
    { rank: 2, name: "Rhodes Logistics", uid: "10044902", server: "EN", level: 120, completion: "97.1%" },
    { rank: 3, name: "SilverAshFan", uid: "10011337", server: "JP", level: 119, completion: "96.8%" },
    { rank: 4, name: "Doctor 1145", uid: "10098215", server: "CN", level: 118, completion: "95.2%" },
];

const MATERIALS = [
    { name: "Bipolar Nanoflake", tier: "T5", needed: 12, owned: 4 },
    { name: "D32 Steel", tier: "T5", needed: 9, owned: 2 },
    { name: "Polymerization Preparation", tier: "T5", needed: 6, owned: 6 },
    { name: "Crystalline Electronic Unit", tier: "T4", needed: 4, owned: 0 },
];

/** The default variant — the leaderboard's ranked Doctor table. */
export const Default = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead className="w-12">Rank</TableHead>
                <TableHead>Doctor</TableHead>
                <TableHead>Server</TableHead>
                <TableHead className="text-right">Level</TableHead>
                <TableHead className="text-right">Completion</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {DOCTORS.map((d) => (
                <TableRow key={d.uid}>
                    <TableCell className="font-mono text-muted-foreground tabular-nums">{d.rank}</TableCell>
                    <TableCell className="font-medium">{d.name}</TableCell>
                    <TableCell>
                        <Badge size="sm" variant="outline">
                            {d.server}
                        </Badge>
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{d.level}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{d.completion}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** `variant="card"` rounds and elevates the body — the depot's material shortfall. */
export const CardVariant = () => (
    <Table variant="card">
        <TableHeader>
            <TableRow>
                <TableHead>Material</TableHead>
                <TableHead>Tier</TableHead>
                <TableHead className="text-right">Needed</TableHead>
                <TableHead className="text-right">Owned</TableHead>
                <TableHead className="text-right">Missing</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {MATERIALS.map((m) => (
                <TableRow key={m.name}>
                    <TableCell className="font-medium">{m.name}</TableCell>
                    <TableCell className="font-mono text-muted-foreground">{m.tier}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.needed}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.owned}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.needed - m.owned || "—"}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** A selected row plus a totals footer, the shape the planner's cost table ships. */
export const WithFooterAndSelection = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Material</TableHead>
                <TableHead className="text-right">Needed</TableHead>
                <TableHead className="text-right">Owned</TableHead>
                <TableHead className="text-right">Missing</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {MATERIALS.map((m) => (
                <TableRow data-state={m.name === "D32 Steel" ? "selected" : undefined} key={m.name}>
                    <TableCell className="font-medium">{m.name}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.needed}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.owned}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.needed - m.owned || "—"}</TableCell>
                </TableRow>
            ))}
        </TableBody>
        <TableFooter>
            <TableRow>
                <TableCell>Total</TableCell>
                <TableCell className="text-right font-mono tabular-nums">31</TableCell>
                <TableCell className="text-right font-mono tabular-nums">12</TableCell>
                <TableCell className="text-right font-mono tabular-nums">19</TableCell>
            </TableRow>
        </TableFooter>
    </Table>
);

/** With a caption — the sanity budget the estimate assumes. */
export const WithCaption = () => (
    <Table>
        <TableCaption>Estimated from 1,284 community drop reports · updated 6 minutes ago.</TableCaption>
        <TableHeader>
            <TableRow>
                <TableHead>Stage</TableHead>
                <TableHead>Drop</TableHead>
                <TableHead className="text-right">Rate</TableHead>
                <TableHead className="text-right">Sanity / item</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            <TableRow>
                <TableCell className="font-mono">S4-1</TableCell>
                <TableCell>Orirock Cluster</TableCell>
                <TableCell className="text-right font-mono tabular-nums">54.1%</TableCell>
                <TableCell className="text-right font-mono tabular-nums">27.7</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-mono">1-7</TableCell>
                <TableCell>Orirock</TableCell>
                <TableCell className="text-right font-mono tabular-nums">72.0%</TableCell>
                <TableCell className="text-right font-mono tabular-nums">8.3</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-mono">4-6</TableCell>
                <TableCell>Polyester Pack</TableCell>
                <TableCell className="text-right font-mono tabular-nums">38.4%</TableCell>
                <TableCell className="text-right font-mono tabular-nums">46.9</TableCell>
            </TableRow>
        </TableBody>
    </Table>
);
