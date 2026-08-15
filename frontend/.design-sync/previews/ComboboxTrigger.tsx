import { Combobox, ComboboxEmpty, ComboboxInput, ComboboxItem, ComboboxList, ComboboxPopup, ComboboxTrigger, ComboboxValue } from "frontend";
import { ChevronsUpDown, Search } from "lucide-react";

const SERVERS = ["Global (EN)", "Japan (JP)", "Korea (KR)", "Taiwan (TW)", "Mainland (CN)"];

const Popup = () => (
    <ComboboxPopup>
        <ComboboxEmpty>No matching servers.</ComboboxEmpty>
        <ComboboxList>
            {(server: string) => (
                <ComboboxItem key={server} value={server}>
                    {server}
                </ComboboxItem>
            )}
        </ComboboxList>
    </ComboboxPopup>
);

export const InsideInput = () => (
    <div className="w-72 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="server-input">
            Server
        </label>
        <Combobox<string, false> defaultValue="Global (EN)" defaultInputValue="Global (EN)" items={SERVERS}>
            <ComboboxInput id="server-input" placeholder="Pick a server" />
            <Popup />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">The chevron button toggles the list without clearing what you typed.</p>
    </div>
);

export const StandaloneTrigger = () => (
    <div className="w-72 space-y-2">
        <span className="block font-medium text-[12px] text-muted-foreground leading-none">Server</span>
        <Combobox<string, false> defaultValue="Japan (JP)" items={SERVERS}>
            <ComboboxTrigger className="flex h-9 w-full cursor-pointer items-center justify-between gap-2 rounded-lg border border-input bg-background px-3 text-left text-sm shadow-xs/5 outline-none hover:bg-accent/40">
                <ComboboxValue placeholder="Pick a server" />
                <ChevronsUpDown className="size-4 shrink-0 opacity-60" />
            </ComboboxTrigger>
            <Popup />
        </Combobox>
    </div>
);

// Opened from the input's built-in trigger: focus stays in the input, so the
// popup renders without the list's focus outline (a trigger-only combobox moves
// focus into the list and paints a ring around it).
export const OpenState = () => (
    <div className="h-72 w-72 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="server-open">
            Server
        </label>
        <Combobox<string, false> defaultOpen defaultValue="Global (EN)" defaultInputValue="Global (EN)" items={SERVERS}>
            <ComboboxInput id="server-open" placeholder="Pick a server" />
            <Popup />
        </Combobox>
    </div>
);

export const HiddenTrigger = () => (
    <div className="w-72 space-y-2">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="server-notrigger">
            Server
        </label>
        <Combobox<string, false> items={SERVERS}>
            <ComboboxInput id="server-notrigger" showTrigger={false} placeholder="Search servers..." startAddon={<Search />} />
            <Popup />
        </Combobox>
        <p className="text-[11px] text-muted-foreground leading-snug">Search-first fields drop the chevron so the field reads as a search box.</p>
    </div>
);
