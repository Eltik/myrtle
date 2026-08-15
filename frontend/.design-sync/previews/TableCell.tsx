import { Badge, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "frontend";

const AVATAR = (id: string) => `https://api.myrtle.moe/api/avatar/${id}`;

const ROSTER = [
    { id: "char_4064_mlynar", name: "Młynar", cls: "Guard", promotion: "E2 90", potential: 3, module: "GUA-Y" },
    { id: "char_263_skadi", name: "Skadi", cls: "Guard", promotion: "E2 80", potential: 1, module: "—" },
    { id: "char_180_amgoat", name: "Eyjafjalla", cls: "Caster", promotion: "E2 90", potential: 6, module: "CST-X" },
    { id: "char_102_texas", name: "Texas", cls: "Vanguard", promotion: "E2 70", potential: 2, module: "PIO-X" },
];

/** Cells hold whatever the column needs — art, text, badges, mono numbers. */
export const MixedContent = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Operator</TableHead>
                <TableHead>Class</TableHead>
                <TableHead>Promotion</TableHead>
                <TableHead className="text-right">Potential</TableHead>
                <TableHead>Module</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {ROSTER.map((op) => (
                <TableRow key={op.id}>
                    <TableCell>
                        <span className="flex items-center gap-2">
                            <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted">
                                <img alt="" className="block h-full w-full object-cover" src={AVATAR(op.id)} />
                            </span>
                            <span className="font-medium">{op.name}</span>
                        </span>
                    </TableCell>
                    <TableCell className="text-muted-foreground">{op.cls}</TableCell>
                    <TableCell>
                        <Badge size="sm" variant="secondary">
                            {op.promotion}
                        </Badge>
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">P{op.potential}</TableCell>
                    <TableCell className="font-mono text-muted-foreground">{op.module}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** Numeric cells: Geist Mono, tabular figures, right-aligned so the columns line up. */
export const NumericColumns = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Operator</TableHead>
                <TableHead className="text-right">DPS</TableHead>
                <TableHead className="text-right">DPH</TableHead>
                <TableHead className="text-right">Total damage</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            <TableRow>
                <TableCell className="font-medium">Młynar</TableCell>
                <TableCell className="text-right font-mono tabular-nums">4,182</TableCell>
                <TableCell className="text-right font-mono tabular-nums">2,911</TableCell>
                <TableCell className="text-right font-mono tabular-nums">125,460</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-medium">Eyjafjalla</TableCell>
                <TableCell className="text-right font-mono tabular-nums">3,764</TableCell>
                <TableCell className="text-right font-mono tabular-nums">5,209</TableCell>
                <TableCell className="text-right font-mono tabular-nums">112,920</TableCell>
            </TableRow>
            <TableRow>
                <TableCell className="font-medium">Skadi</TableCell>
                <TableCell className="text-right font-mono tabular-nums">2,948</TableCell>
                <TableCell className="text-right font-mono tabular-nums">1,704</TableCell>
                <TableCell className="text-right font-mono tabular-nums">88,440</TableCell>
            </TableRow>
        </TableBody>
    </Table>
);

/** A spanning cell fills the width when a group has nothing to show. */
export const SpanningCell = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead>Operator</TableHead>
                <TableHead>Class</TableHead>
                <TableHead className="text-right">Potential</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {ROSTER.slice(0, 2).map((op) => (
                <TableRow key={op.id}>
                    <TableCell className="font-medium">{op.name}</TableCell>
                    <TableCell className="text-muted-foreground">{op.cls}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">P{op.potential}</TableCell>
                </TableRow>
            ))}
            <TableRow>
                <TableCell className="py-8 text-center text-muted-foreground" colSpan={3}>
                    No other 6★ Guards in this roster.
                </TableCell>
            </TableRow>
        </TableBody>
    </Table>
);
