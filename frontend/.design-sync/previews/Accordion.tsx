import { Accordion, AccordionItem, AccordionPanel, AccordionTrigger } from "frontend";
import { LockIcon } from "lucide-react";

export const Single = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["sanity"]}>
        <AccordionItem value="sanity">
            <AccordionTrigger>How is sanity regeneration calculated?</AccordionTrigger>
            <AccordionPanel>One sanity every 6 minutes, capped at your max sanity (135 at level 120). Overflow from potions is stored, not lost, so a full refill before a farming session is never wasted.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="drops">
            <AccordionTrigger>Where do the drop rates come from?</AccordionTrigger>
            <AccordionPanel>Community-reported clears from Penguin Statistics, refreshed nightly. Stages with fewer than 300 recorded runs are marked as low confidence.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="accounts">
            <AccordionTrigger>Can I link more than one account?</AccordionTrigger>
            <AccordionPanel>Yes — link one account per server (EN, CN, JP, KR). Your roster, depot, and planner sync independently for each.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const Multiple = () => (
    <Accordion className="w-full max-w-lg" multiple defaultValue={["potential", "skills"]}>
        <AccordionItem value="potential">
            <AccordionTrigger className="py-2 font-medium text-sm">Potential</AccordionTrigger>
            <AccordionPanel>
                <div className="flex items-center justify-between py-1 text-sm">
                    <span className="text-foreground">Current potential</span>
                    <span className="font-mono tabular-nums">+3</span>
                </div>
                <div className="flex items-center justify-between py-1 text-sm">
                    <span className="text-foreground">Next bonus</span>
                    <span>Attack +28</span>
                </div>
            </AccordionPanel>
        </AccordionItem>
        <AccordionItem value="skills">
            <AccordionTrigger className="py-2 font-medium text-sm">Skills</AccordionTrigger>
            <AccordionPanel>
                <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm">
                        <span className="text-foreground">Leithanien Recollections</span>
                        <span className="font-mono tabular-nums">M3</span>
                    </div>
                    <div className="flex items-center justify-between text-sm">
                        <span className="text-foreground">Fulminarum Contemplatus</span>
                        <span className="font-mono tabular-nums">M2</span>
                    </div>
                </div>
            </AccordionPanel>
        </AccordionItem>
        <AccordionItem value="modules">
            <AccordionTrigger className="py-2 font-medium text-sm">Modules</AccordionTrigger>
            <AccordionPanel>SWD-X Stage 3 equipped. SWD-Y unlocked at Trust 100.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const Disabled = () => (
    <Accordion className="w-full max-w-lg" disabled defaultValue={["locked"]}>
        <AccordionItem value="locked">
            <AccordionTrigger className="text-muted-foreground">
                <span className="flex flex-1 items-center gap-2">
                    <LockIcon className="size-3.5 shrink-0" />
                    Integrated Strategies #4 — Expeditioner's Joklumarkar
                </span>
            </AccordionTrigger>
            <AccordionPanel>Clear Chapter 4 to unlock the Roguelike game mode and its collectible relics.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="locked-2">
            <AccordionTrigger className="text-muted-foreground">
                <span className="flex flex-1 items-center gap-2">
                    <LockIcon className="size-3.5 shrink-0" />
                    Reclamation Algorithm
                </span>
            </AccordionTrigger>
            <AccordionPanel>Event archive unavailable outside of a live rerun.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);
