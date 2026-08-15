import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "frontend";

const ROWS = [
    { stage: "S4-1", drop: "Orirock Cluster", rate: "54.1%", sanity: "27.7" },
    { stage: "1-7", drop: "Orirock", rate: "72.0%", sanity: "8.3" },
    { stage: "4-6", drop: "Polyester Pack", rate: "38.4%", sanity: "46.9" },
];

/** The header row draws the muted column labels and the rule under them. */
export const ColumnHeaders = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Stage</TableHead>
                <TableHead>Drop</TableHead>
                <TableHead className="text-right">Rate</TableHead>
                <TableHead className="text-right">Sanity / item</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {ROWS.map((r) => (
                <TableRow key={r.stage}>
                    <TableCell className="font-mono">{r.stage}</TableCell>
                    <TableCell>{r.drop}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{r.rate}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{r.sanity}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** In the card variant the header sits outside the elevated body. */
export const CardVariant = () => (
    <Table variant="card">
        <TableHeader>
            <TableRow>
                <TableHead>Stage</TableHead>
                <TableHead>Drop</TableHead>
                <TableHead className="text-right">Rate</TableHead>
                <TableHead className="text-right">Sanity / item</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {ROWS.map((r) => (
                <TableRow key={r.stage}>
                    <TableCell className="font-mono">{r.stage}</TableCell>
                    <TableCell>{r.drop}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{r.rate}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{r.sanity}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** Grouped headers — a two-row header for the recruitment tag matrix. */
export const GroupedHeaders = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead rowSpan={2}>Tag</TableHead>
                <TableHead className="text-right" colSpan={2}>
                    Guaranteed
                </TableHead>
                <TableHead className="text-right" rowSpan={2}>
                    Pool
                </TableHead>
            </TableRow>
            <TableRow>
                <TableHead className="text-right">5★</TableHead>
                <TableHead className="text-right">6★</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            <TableRow>
                <TableCell className="font-medium">Top Operator</TableCell>
                <TableCell className="text-right font-mono tabular-nums">0</TableCell>
                <TableCell className="text-right font-mono tabular-nums">18</TableCell>
                <TableCell className="text-right font-mono tabular-nums">18</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-medium">Senior Operator</TableCell>
                <TableCell className="text-right font-mono tabular-nums">41</TableCell>
                <TableCell className="text-right font-mono tabular-nums">0</TableCell>
                <TableCell className="text-right font-mono tabular-nums">41</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-medium">Defender</TableCell>
                <TableCell className="text-right font-mono tabular-nums">7</TableCell>
                <TableCell className="text-right font-mono tabular-nums">3</TableCell>
                <TableCell className="text-right font-mono tabular-nums">34</TableCell>
            </TableRow>
        </TableBody>
    </Table>
);
