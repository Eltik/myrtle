import { Table, TableBody, TableCaption, TableCell, TableHead, TableHeader, TableRow } from "frontend";

const ROWS = [
    { stage: "S4-1", drop: "Orirock Cluster", rate: "54.1%", sanity: "27.7" },
    { stage: "1-7", drop: "Orirock", rate: "72.0%", sanity: "8.3" },
    { stage: "4-6", drop: "Polyester Pack", rate: "38.4%", sanity: "46.9" },
];

/** The caption sits under the table and carries the data's provenance. */
export const DataProvenance = () => (
    <Table>
        <TableCaption>Estimated from 1,284 community drop reports · updated 6 minutes ago.</TableCaption>
        <TableHeader>
            <TableRow>
                <TableHead className="w-20">Stage</TableHead>
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

/** `caption-side` is bottom by default; the card variant just adds breathing room. */
export const CardVariant = () => (
    <Table variant="card">
        <TableCaption>Sanity costs assume the EN server's current event multipliers.</TableCaption>
        <TableHeader>
            <TableRow>
                <TableHead className="w-20">Stage</TableHead>
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
