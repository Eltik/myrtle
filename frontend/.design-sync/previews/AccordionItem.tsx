import { Accordion, AccordionItem, AccordionPanel, AccordionTrigger } from "frontend";
import { LockIcon } from "lucide-react";

export const OpenAndClosed = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["ch8"]}>
        <AccordionItem value="ch8">
            <AccordionTrigger>Chapter 8 — Roaring Flare</AccordionTrigger>
            <AccordionPanel>14 stages, 3 challenge modes. First-clear rewards include 2 Chip Catalysts and 4 Orirock Cubes.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="ch9">
            <AccordionTrigger>Chapter 9 — Stormwatch</AccordionTrigger>
            <AccordionPanel>18 stages. Recommended average level E2 40 before attempting 9-17.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="ch10">
            <AccordionTrigger>Chapter 10 — Shatterpoint</AccordionTrigger>
            <AccordionPanel>17 stages. Introduces Sarkaz Sunborn enemies with stacking damage reduction.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const DisabledItem = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["owned"]}>
        <AccordionItem value="owned">
            <AccordionTrigger>Owned operators (231)</AccordionTrigger>
            <AccordionPanel>Synced from your EN account 6 minutes ago. 42 operators are at E2, 18 have a mastered skill.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="depot">
            <AccordionTrigger>Depot materials</AccordionTrigger>
            <AccordionPanel>Scan your depot screenshot to import quantities, or edit counts by hand.</AccordionPanel>
        </AccordionItem>
        <AccordionItem disabled value="friends">
            <AccordionTrigger className="text-muted-foreground">
                <span className="flex flex-1 items-center gap-2">
                    <LockIcon className="size-3.5 shrink-0" />
                    Support units
                </span>
                <span className="mr-2 text-xs">Link an account</span>
            </AccordionTrigger>
            <AccordionPanel>Support unit history requires a linked account.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const BorderlessItems = () => (
    <Accordion className="w-full max-w-lg rounded-xl border p-2" multiple defaultValue={["stats"]}>
        <AccordionItem className="border-b-0" value="stats">
            <AccordionTrigger className="px-2 py-2 font-medium text-sm">Attributes at E2 90</AccordionTrigger>
            <AccordionPanel className="px-2">
                <dl className="grid grid-cols-2 gap-x-6 gap-y-1 text-sm">
                    <div className="flex justify-between">
                        <dt>HP</dt>
                        <dd className="font-mono tabular-nums">3,368</dd>
                    </div>
                    <div className="flex justify-between">
                        <dt>ATK</dt>
                        <dd className="font-mono tabular-nums">1,047</dd>
                    </div>
                    <div className="flex justify-between">
                        <dt>DEF</dt>
                        <dd className="font-mono tabular-nums">507</dd>
                    </div>
                    <div className="flex justify-between">
                        <dt>RES</dt>
                        <dd className="font-mono tabular-nums">0</dd>
                    </div>
                </dl>
            </AccordionPanel>
        </AccordionItem>
        <AccordionItem className="border-b-0" value="trust">
            <AccordionTrigger className="px-2 py-2 font-medium text-sm">Trust bonus</AccordionTrigger>
            <AccordionPanel className="px-2">Trust 200 — Attack +65, Defence +50.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);
