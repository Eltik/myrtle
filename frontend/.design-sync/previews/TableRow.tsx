import { Badge, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "frontend";

const DOCTORS = [
    { rank: 1, name: "Kal'tsit Enjoyer", server: "EN", level: 120, completion: "98.4%" },
    { rank: 2, name: "Rhodes Logistics", server: "EN", level: 120, completion: "97.1%" },
    { rank: 3, name: "SilverAshFan", server: "JP", level: 119, completion: "96.8%" },
    { rank: 4, name: "Doctor 1145", server: "CN", level: 118, completion: "95.2%" },
];

/** Rows carry the bottom rule and the hover tint. */
export const Rows = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead className="w-12">Rank</TableHead>
                <TableHead>Doctor</TableHead>
                <TableHead>Server</TableHead>
                <TableHead className="text-right">Completion</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {DOCTORS.map((d) => (
                <TableRow key={d.name}>
                    <TableCell className="font-mono text-muted-foreground tabular-nums">{d.rank}</TableCell>
                    <TableCell className="font-medium">{d.name}</TableCell>
                    <TableCell>
                        <Badge size="sm" variant="outline">
                            {d.server}
                        </Badge>
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{d.completion}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** `data-state="selected"` tints the row the doctor is comparing against. */
export const Selected = () => (
    <Table>
        <TableHeader>
            <TableRow>
                <TableHead className="w-12">Rank</TableHead>
                <TableHead>Doctor</TableHead>
                <TableHead className="text-right">Level</TableHead>
                <TableHead className="text-right">Completion</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {DOCTORS.map((d) => (
                <TableRow data-state={d.rank === 3 ? "selected" : undefined} key={d.name}>
                    <TableCell className="font-mono text-muted-foreground tabular-nums">{d.rank}</TableCell>
                    <TableCell className="font-medium">{d.name}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{d.level}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{d.completion}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);

/** In the card variant the first and last rows round the surface's corners. */
export const CardVariant = () => (
    <Table variant="card">
        <TableHeader>
            <TableRow>
                <TableHead className="w-12">Rank</TableHead>
                <TableHead>Doctor</TableHead>
                <TableHead>Server</TableHead>
                <TableHead className="text-right">Completion</TableHead>
            </TableRow>
        </TableHeader>
        <TableBody>
            {DOCTORS.map((d) => (
                <TableRow data-state={d.rank === 1 ? "selected" : undefined} key={d.name}>
                    <TableCell className="font-mono text-muted-foreground tabular-nums">{d.rank}</TableCell>
                    <TableCell className="font-medium">{d.name}</TableCell>
                    <TableCell className="text-muted-foreground">{d.server}</TableCell>
                    <TableCell className="text-right font-mono tabular-nums">{d.completion}</TableCell>
                </TableRow>
            ))}
        </TableBody>
    </Table>
);
