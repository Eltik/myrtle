import { Accordion, AccordionItem, AccordionPanel, AccordionTrigger, Progress, Separator } from "frontend";

export const TextPanel = () => (
    <Accordion className="w-full max-w-lg" defaultValue={["rates"]}>
        <AccordionItem value="rates">
            <AccordionTrigger>Headhunting rates</AccordionTrigger>
            <AccordionPanel>Base 6-star rate is 2%. After 50 pulls without a 6-star the rate rises by 2 percentage points per pull, so a 6-star is guaranteed by pull 99. The counter resets on any 6-star, rate-up or not.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="sparks">
            <AccordionTrigger>Kernel banners</AccordionTrigger>
            <AccordionPanel>Kernel Locating uses a separate 300-pull spark counter and only offers operators from the launch roster.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const RichPanel = () => (
    <Accordion className="w-full max-w-lg" multiple defaultValue={["progress"]}>
        <AccordionItem value="progress">
            <AccordionTrigger className="py-2 font-medium text-sm">Mlynar — E2 78</AccordionTrigger>
            <AccordionPanel>
                <div className="space-y-3">
                    <div className="space-y-1.5">
                        <div className="flex items-center justify-between text-xs">
                            <span className="text-foreground">Level 78 / 90</span>
                            <span className="font-mono tabular-nums">86%</span>
                        </div>
                        <Progress value={86} />
                    </div>
                    <div className="space-y-1.5">
                        <div className="flex items-center justify-between text-xs">
                            <span className="text-foreground">Trust 176 / 200</span>
                            <span className="font-mono tabular-nums">88%</span>
                        </div>
                        <Progress value={88} />
                    </div>
                    <Separator />
                    <p className="text-xs">Remaining to max: 1.42M LMD and 214 EXP cards.</p>
                </div>
            </AccordionPanel>
        </AccordionItem>
        <AccordionItem value="cost">
            <AccordionTrigger className="py-2 font-medium text-sm">Remaining upgrade cost</AccordionTrigger>
            <AccordionPanel>
                <ul className="space-y-1 text-sm">
                    <li className="flex justify-between">
                        <span>Sanity</span>
                        <span className="font-mono tabular-nums">3,120</span>
                    </li>
                    <li className="flex justify-between">
                        <span>LMD</span>
                        <span className="font-mono tabular-nums">1,420,000</span>
                    </li>
                    <li className="flex justify-between">
                        <span>Days at 4 refills / week</span>
                        <span className="font-mono tabular-nums">17</span>
                    </li>
                </ul>
            </AccordionPanel>
        </AccordionItem>
    </Accordion>
);

export const AllClosed = () => (
    <Accordion className="w-full max-w-lg">
        <AccordionItem value="a">
            <AccordionTrigger>Squad presets</AccordionTrigger>
            <AccordionPanel>Save up to 12 squads and recall them from the stage planner.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="b">
            <AccordionTrigger>Notification settings</AccordionTrigger>
            <AccordionPanel>Get a reminder when your sanity is about to overflow.</AccordionPanel>
        </AccordionItem>
        <AccordionItem value="c">
            <AccordionTrigger>Data export</AccordionTrigger>
            <AccordionPanel>Download your roster, depot, and planner state as JSON.</AccordionPanel>
        </AccordionItem>
    </Accordion>
);
