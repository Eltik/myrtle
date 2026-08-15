import { Badge, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "frontend";

const MATERIALS = [
    { name: "Bipolar Nanoflake", tier: "T5", needed: 12, owned: 4 },
    { name: "D32 Steel", tier: "T5", needed: 9, owned: 2 },
    { name: "Polymerization Preparation", tier: "T5", needed: 6, owned: 6 },
    { name: "Crystalline Electronic Unit", tier: "T4", needed: 4, owned: 0 },
];

/** The body holds the data rows; hover and selection styling live here. */
export const MaterialRows = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Material</TableHead>
                <TableHead>Tier</TableHead>
                <TableHead className="text-right">Needed</TableHead>
                <TableHead className="text-right">Owned</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {MATERIALS.map((m) => (
                <TableRow key={m.name}>
                    <TableCell className="font-medium">{m.name}</TableCell>
                    <TableCell>
                        <Badge size="sm" variant="outline">
                            {m.tier}
                        </Badge>
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.needed}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{m.owned}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** In the card variant the body is what gets the rounded, elevated surface. */
export const CardVariant = () => (
    <Table variant="card">
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
    </Table>
);

/** Nothing left to farm — a single spanning row instead of an empty body. */
export const NoResults = () => (
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
            <TableRow>
                <TableCell className="py-8 text-center text-muted-foreground" colSpan={4}>
                    Every material for this plan is already in your depot.
                </TableCell>
            </TableRow>
        </TableBody>
    </Table>
);
