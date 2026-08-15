import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "frontend";
import { ArrowDownIcon, ArrowUpDownIcon } from "lucide-react";

const ROWS = [
    { stage: "S4-1", drop: "Orirock Cluster", rate: "54.1%", sanity: "27.7" },
    { stage: "1-7", drop: "Orirock", rate: "72.0%", sanity: "8.3" },
    { stage: "4-6", drop: "Polyester Pack", rate: "38.4%", sanity: "46.9" },
];

/** Numeric columns are right-aligned; the first column is width-pinned. */
export const Alignment = () => (
    <Table>
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

/** Sortable headers: the active column keeps its direction arrow. */
export const Sortable = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead className="w-20">Stage</TableHead>
                <TableHead>Drop</TableHead>
                <TableHead className="text-right">
                    <span className="inline-flex items-center gap-1">
                        Rate
                        <ArrowUpDownIcon className="size-3.5 opacity-64" />
                    </span>
                </TableHead>
                <TableHead className="text-right text-foreground">
                    <span className="inline-flex items-center gap-1">
                        Sanity / item
                        <ArrowDownIcon className="size-3.5" />
                    </span>
                </TableHead>
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

/** A `colSpan` header groups the two guaranteed-rarity columns. */
export const Spanning = () => (
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
        </TableBody>
    </Table>
);
