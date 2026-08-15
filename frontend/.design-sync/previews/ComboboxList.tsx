import { Fragment } from "react";
import { Combobox, ComboboxCollection, ComboboxEmpty, ComboboxGroup, ComboboxGroupLabel, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup } from "frontend";
import { Search } from "lucide-react";

const STAGES = ["1-7 — Vanguard Training", "4-8 — Iron Heart", "6-16 — Ashes", "7-18 — Bone Deep", "9-17 — Wintry Wind", "10-16 — Frostbite", "11-20 — Last Wish", "12-17 — Homecoming", "13-16 — Sanctuary", "14-19 — Sundown"];

const BANNERS = [
    { key: "limited", label: "Limited", items: ["Ambience Synesthesia", "Ideal City", "Break the Ice"] },
    { key: "standard", label: "Standard", items: ["Perilous Trails", "Guide Ahead", "Who Is Real"] },
];

export const FlatList = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="list-flat">
            Stage
        </label>
        <Combobox<string, false> defaultOpen items={STAGES}>
            <ComboboxInput id="list-flat" placeholder="Search 1,240 stages..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching stages.</ComboboxEmpty>
                <ComboboxList>
                    {(stage: string) => (
                        <ComboboxItem key={stage} value={stage}>
                            {stage}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const GroupedList = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="list-grouped">
            Banner
        </label>
        <Combobox<string, false> defaultOpen items={BANNERS}>
            <ComboboxInput id="list-grouped" placeholder="Search banners..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching banners.</ComboboxEmpty>
                <ComboboxList>
                    {(group: (typeof BANNERS)[number]) => (
                        <Fragment key={group.key}>
                            <ComboboxGroup items={group.items}>
                                <ComboboxGroupLabel>{group.label}</ComboboxGroupLabel>
                                <ComboboxCollection>
                                    {(banner: string) => (
                                        <ComboboxItem key={banner} value={banner}>
                                            {banner}
                                        </ComboboxItem>
                                    )}
                                </ComboboxCollection>
                            </ComboboxGroup>
                        </Fragment>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const ScrollingList = () => (
    <div className="h-96 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="list-scroll">
            Stage
        </label>
        <Combobox<string, false> defaultOpen items={[...STAGES, "S2-6 — Sunken Path", "S3-3 — Crossing", "S4-1 — Pyrolysis", "S5-9 — Dark Tide", "S6-4 — Rain Curtain", "S7-2 — Ashfall"]}>
            <ComboboxInput id="list-scroll" placeholder="Search 1,240 stages..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No matching stages.</ComboboxEmpty>
                <ComboboxList>
                    {(stage: string) => (
                        <ComboboxItem key={stage} value={stage}>
                            {stage}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);

export const EmptyList = () => (
    <div className="h-72 w-80 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="list-empty">
            Stage
        </label>
        <Combobox<string, false> defaultOpen defaultInputValue="CB-EX9" items={STAGES}>
            <ComboboxInput id="list-empty" placeholder="Search 1,240 stages..." startAddon={<Search />} />
            <ComboboxPopup>
                <ComboboxEmpty>No stage matches “CB-EX9”.</ComboboxEmpty>
                <ComboboxList>
                    {(stage: string) => (
                        <ComboboxItem key={stage} value={stage}>
                            {stage}
                        </ComboboxItem>
                    )}
                </ComboboxList>
            </ComboboxPopup>
        </Combobox>
    </div>
);
