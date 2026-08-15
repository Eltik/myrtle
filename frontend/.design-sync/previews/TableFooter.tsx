import { Table, TableBody, TableCell, TableFooter, TableHead, TableHeader, TableRow } from "frontend";

const MATERIALS = [
    { name: "Bipolar Nanoflake", needed: 12, owned: 4 },
    { name: "D32 Steel", needed: 9, owned: 2 },
    { name: "Polymerization Preparation", needed: 6, owned: 6 },
    { name: "Crystalline Electronic Unit", needed: 4, owned: 0 },
];

/** The footer carries the totals row, set apart by a rule and a tinted band. */
export const Totals = () => (
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
                <TableRow key={m.name}>
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

/** A spanning summary cell — the sanity estimate for everything still missing. */
export const SanitySummary = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Stage</TableHead>
                <TableHead>Farming for</TableHead>
                <TableHead className="text-right">Runs</TableHead>
                <TableHead className="text-right">Sanity</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            <TableRow>
                <TableCell className="font-mono">S4-1</TableCell>
                <TableCell>Orirock Cluster</TableCell>
                <TableCell className="text-right font-mono tabular-nums">22</TableCell>
                <TableCell className="text-right font-mono tabular-nums">396</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-mono">4-6</TableCell>
                <TableCell>Polyester Pack</TableCell>
                <TableCell className="text-right font-mono tabular-nums">17</TableCell>
                <TableCell className="text-right font-mono tabular-nums">357</TableCell>
            </TableRow>
        </TableBody>
        <TableFooter>
            <TableRow>
                <TableCell colSpan={3}>Estimated sanity to finish the plan</TableCell>
                <TableCell className="text-right font-mono text-primary tabular-nums">753</TableCell>
            </TableRow>
        </TableFooter>
    </Table>
);

/** The card variant drops the footer's band so the totals float under the surface. */
export const CardVariant = () => (
    <Table variant="card">
        <TableHeader>
            <TableRow>
                <TableHead>Material</TableHead>
                <TableHead className="text-right">Needed</TableHead>
                <TableHead className="text-right">Owned</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {MATERIALS.slice(0, 3).map((m) => (
                <TableRow key={m.name}>
                    <TableCell className="font-medium">{m.name}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.needed}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.owned}</TableCell>
                </TableRow>
            ))}
        </TableBody>
        <TableFooter>
            <TableRow>
                <TableCell>Total</TableCell>
                <TableCell className="text-right font-mono tabular-nums">27</TableCell>
                <TableCell className="text-right font-mono tabular-nums">12</TableCell>
            </TableRow>
        </TableFooter>
    </Table>
);
