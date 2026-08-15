import { Accordion, AccordionItem, AccordionPanel, AccordionTrigger, Badge } from "frontend";

export const PlainTriggers = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["e2"]}>
        <AccordionItem value="e2">
            <AccordionTrigger>What does Elite 2 promotion cost?</AccordionTrigger>
            <AccordionPanel>For a 6-star: 180,000 LMD, 4 Dualchip, and the operator at level 80. Promotion resets the level to 1 but unlocks the second skill upgrade track.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="mastery">
            <AccordionTrigger>Is Mastery 3 worth it on every skill?</AccordionTrigger>
            <AccordionPanel>No. Prioritise skills whose mastery changes the breakpoint — Mlynar S3 and Skadi S2 both gain a full extra activation, while most passive skills gain only a few percent.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const WithTrailingMeta = () => (
    <Accordion className="w-full max-w-lg" multiple defaultValue={["chips"]}>
        <AccordionItem value="chips">
            <AccordionTrigger>
                <span className="flex flex-1 items-center gap-2">
                    Chip Catalyst
                    <Badge variant="secondary">T4</Badge>
                </span>
                <span className="mr-2 font-mono text-muted-foreground text-xs tabular-nums">12 owned</span>
            </AccordionTrigger>
            <AccordionPanel>Farmed from Chip Search (LS/CE rotation is not a source). Trade 2 chips of any class for 1 catalyst at the certificate shop.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="rock">
            <AccordionTrigger>
                <span className="flex flex-1 items-center gap-2">
                    Orirock Concentration
                    <Badge variant="secondary">T3</Badge>
                </span>
                <span className="mr-2 font-mono text-muted-foreground text-xs tabular-nums">37 owned</span>
            </AccordionTrigger>
            <AccordionPanel>Best rate at 1-7 (0.72 per sanity when cubes are converted). Byproduct of Orirock Cluster synthesis.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const TwoLineTrigger = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["s4-1"]}>
        <AccordionItem value="s4-1">
            <AccordionTrigger>
                <span className="flex flex-col gap-0.5">
                    <span>S4-1 — Sanity efficiency 0.68 / run</span>
                    <span className="font-normal text-muted-foreground text-xs">Polyester Pack · 18 sanity · 1,240 recorded clears</span>
                </span>
            </AccordionTrigger>
            <AccordionPanel>Recommended for Polyester Pack farming once you can auto-deploy with a single E2 Guard. Drops Polyester Lump as a common byproduct.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="1-7">
            <AccordionTrigger>
                <span className="flex flex-col gap-0.5">
                    <span>1-7 — Sanity efficiency 0.72 / run</span>
                    <span className="font-normal text-muted-foreground text-xs">Orirock Cube · 6 sanity · 9,880 recorded clears</span>
                </span>
            </AccordionTrigger>
            <AccordionPanel>The cheapest reliable Orirock source in the game, and the standard benchmark every other cube stage is measured against.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);
