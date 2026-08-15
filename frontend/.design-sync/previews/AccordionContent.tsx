import { Accordion, AccordionContent, AccordionItem, AccordionTrigger, Badge } from "frontend";

export const StageGuide = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["ce-6"]}>
        <AccordionItem value="ce-6">
            <AccordionTrigger>CE-6 — Cargo Escort</AccordionTrigger>
            <AccordionContent>
                <p>36 sanity for 10,000 LMD, the best LMD-per-sanity rate available. Open Tuesday, Thursday, Saturday, and Sunday.</p>
                <p className="mt-2">A single E2 Thorns or Mlynar can hold the right lane unassisted; most doctors run it on auto-deploy.</p>
            </AccordionContent>
        </AccordionItem>
        <AccordionItem value="ls-6">
            <AccordionTrigger>LS-6 — Tactical Drill</AccordionTrigger>
            <AccordionContent>30 sanity for 7,400 EXP. Open daily, and the only EXP source worth farming once your roster is past E1.</AccordionContent>
        </AccordionItem>
        <AccordionItem value="ap-5">
            <AccordionTrigger>AP-5 — Solid Defence</AccordionTrigger>
            <AccordionContent>Skill summary drops. Only worth running while a mastery is actively blocked on summaries.</AccordionContent>
        </AccordionItem>
    </Accordion>
);

export const ChangelogPanels = () => (
    <Accordion className="w-full max-w-lg" multiple defaultValue={["v12", "v11"]}>
        <AccordionItem value="v12">
            <AccordionTrigger>
                <span className="flex flex-1 items-center gap-2">
                    Tier list v12
                    <Badge variant="secondary">Current</Badge>
                </span>
                <span className="mr-2 text-muted-foreground text-xs">15 May 2024</span>
            </AccordionTrigger>
            <AccordionContent>
                <ul className="list-disc space-y-1 pl-4">
                    <li>Muelsyse moved S → S+ after her module release.</li>
                    <li>Texas the Omertosa held at S+ on 312 votes.</li>
                    <li>Six Sniper entries re-scored against the new DPS harness.</li>
                </ul>
            </AccordionContent>
        </AccordionItem>
        <AccordionItem value="v11">
            <AccordionTrigger>
                <span className="flex-1">Tier list v11</span>
                <span className="mr-2 text-muted-foreground text-xs">2 Apr 2024</span>
            </AccordionTrigger>
            <AccordionContent>
                <ul className="list-disc space-y-1 pl-4">
                    <li>Added the Chapter 12 enemy set to the survivability weighting.</li>
                    <li>Skadi the Corrupting Heart moved A → S.</li>
                </ul>
            </AccordionContent>
        </AccordionItem>
    </Accordion>
);

export const NestedList = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["mats"]}>
        <AccordionItem value="mats">
            <AccordionTrigger>Materials still needed (4)</AccordionTrigger>
            <AccordionContent>
                <ul className="space-y-1.5">
                    <li className="flex items-center justify-between">
                        <span className="text-foreground">Bipolar Nanoflake</span>
                        <span className="font-mono tabular-nums">3 / 5</span>
                    </li>
                    <li className="flex items-center justify-between">
                        <span className="text-foreground">Crystalline Electronic Unit</span>
                        <span className="font-mono tabular-nums">1 / 4</span>
                    </li>
                    <li className="flex items-center justify-between">
                        <span className="text-foreground">Polymerized Gel</span>
                        <span className="font-mono tabular-nums">6 / 8</span>
                    </li>
                    <li className="flex items-center justify-between">
                        <span className="text-foreground">D32 Steel</span>
                        <span className="font-mono tabular-nums">0 / 2</span>
                    </li>
                </ul>
            </AccordionContent>
        </AccordionItem>
        <AccordionItem value="have">
            <AccordionTrigger>Already covered by depot (11)</AccordionTrigger>
            <AccordionContent>Orirock Cube, Sugar Pack, Polyester Pack, Oriron Cluster and 7 more are fully covered by your current depot.</AccordionContent>
        </AccordionItem>
    </Accordion>
);
