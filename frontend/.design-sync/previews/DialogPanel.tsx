import { Badge, Button, Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle, Separator } from "frontend";
import type { ReactNode } from "react";

const Stage = ({ children }: { children?: ReactNode }) => <div className="relative min-h-[520px] w-full">{children}</div>;

const Stat = ({ kicker, value, tone }: { kicker: string; value: string; tone?: "success" | "muted" }) => (
    <div className="flex flex-col gap-1.5 rounded-lg border bg-muted/50 p-3">
        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">{kicker}</span>
        <span className={`font-bold text-lg tabular-nums leading-none ${tone === "success" ? "text-success-foreground" : tone === "muted" ? "text-muted-foreground" : "text-foreground"}`}>{value}</span>
    </div>
);

// The panel is the scrollable body between header and footer — here holding a
// stat grid that fits without scrolling.
export const Stats = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-lg">
                <DialogHeader className="pr-12">
                    <DialogTitle>Orirock Cube</DialogTitle>
                    <DialogDescription>Tier 3 material. Crafted from 3 Orirock Cluster in any Factory.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-5">
                    <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                        <Stat kicker="On hand" value="184" />
                        <Stat kicker="Planned" value="126" />
                        <Stat kicker="Surplus" tone="success" value="58" />
                        <Stat kicker="Rarity" tone="muted" value="3★" />
                    </div>
                    <div className="flex flex-col gap-2.5">
                        <span className="font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tracking-[0.12em]">Obtain channels</span>
                        <div className="flex flex-wrap gap-1.5">
                            <Badge variant="outline">Factory</Badge>
                            <Badge variant="outline">1-7</Badge>
                            <Badge variant="outline">Annihilation</Badge>
                            <Badge variant="secondary">Store</Badge>
                        </div>
                    </div>
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Add to planner</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// Overflowing content scrolls inside the panel while the header and footer stay
// put; `scrollFade` fades the clipped edge.
export const Scrollable = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Farm plan</DialogTitle>
                    <DialogDescription>Cheapest sanity route for the 12 goals in your planner.</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-0">
                    {[
                        { code: "4-7", drop: "Polyester Pack", runs: 18, sanity: 21 },
                        { code: "1-7", drop: "Orirock Cube", runs: 34, sanity: 6 },
                        { code: "S4-1", drop: "Sugar Substitute", runs: 12, sanity: 18 },
                        { code: "2-8", drop: "Damaged Device", runs: 22, sanity: 18 },
                        { code: "S3-6", drop: "Aketon", runs: 9, sanity: 21 },
                        { code: "CE-6", drop: "LMD 7,500", runs: 4, sanity: 30 },
                        { code: "10-8", drop: "White Horse Kohl", runs: 27, sanity: 21 },
                        { code: "6-16", drop: "Manganese Ore", runs: 15, sanity: 21 },
                        { code: "7-16", drop: "RMA70-12", runs: 11, sanity: 21 },
                        { code: "9-15", drop: "Crystalline Component", runs: 19, sanity: 21 },
                        { code: "5-8", drop: "Grindstone", runs: 14, sanity: 18 },
                        { code: "S5-7", drop: "Loxic Kohl", runs: 21, sanity: 18 },
                        { code: "3-4", drop: "Device", runs: 16, sanity: 15 },
                        { code: "AP-5", drop: "Skill Summary 3", runs: 8, sanity: 30 },
                    ].map((row, i) => (
                        <div key={row.code}>
                            {i > 0 && <Separator />}
                            <div className="flex items-center gap-3 py-2.5">
                                <span className="w-12 shrink-0 font-mono font-semibold text-sm tabular-nums">{row.code}</span>
                                <span className="min-w-0 flex-1 truncate text-sm">{row.drop}</span>
                                <span className="font-mono text-muted-foreground text-xs tabular-nums">
                                    {row.runs} runs · {row.sanity * row.runs} sanity
                                </span>
                            </div>
                        </div>
                    ))}
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Copy route</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);

// `scrollFade={false}` keeps a hard edge — right for content that already ends
// on a rule or a summary row.
export const WithoutScrollFade = () => (
    <Stage>
        <Dialog open>
            <DialogPopup className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Recent pulls</DialogTitle>
                    <DialogDescription>Standard banner · 14 most recent, recorded by doctor#8815</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-0" scrollFade={false}>
                    {[
                        { name: "Mlynar", rarity: 6, pity: 62 },
                        { name: "Blemishine", rarity: 6, pity: 41 },
                        { name: "Whislash", rarity: 5, pity: 12 },
                        { name: "Franka", rarity: 5, pity: 8 },
                        { name: "Cuora", rarity: 4, pity: 3 },
                        { name: "Gravel", rarity: 4, pity: 2 },
                        { name: "Ansel", rarity: 4, pity: 1 },
                        { name: "Vigna", rarity: 5, pity: 27 },
                        { name: "Myrtle", rarity: 4, pity: 6 },
                        { name: "Perfumer", rarity: 5, pity: 19 },
                        { name: "Jaye", rarity: 4, pity: 4 },
                        { name: "Popukar", rarity: 3, pity: 3 },
                        { name: "Beagle", rarity: 3, pity: 2 },
                        { name: "Melantha", rarity: 3, pity: 1 },
                    ].map((pull, i) => (
                        <div key={pull.name}>
                            {i > 0 && <Separator />}
                            <div className="flex items-center gap-3 py-2.5">
                                <span className="min-w-0 flex-1 truncate text-sm">{pull.name}</span>
                                <Badge variant={pull.rarity === 6 ? "warning" : pull.rarity === 5 ? "secondary" : "outline"}>{pull.rarity}★</Badge>
                                <span className="w-16 text-right font-mono text-muted-foreground text-xs tabular-nums">{pull.pity} pity</span>
                            </div>
                        </div>
                    ))}
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button type="button" variant="outline" />}>Close</DialogClose>
                    <Button type="button">Full history</Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    </Stage>
);
