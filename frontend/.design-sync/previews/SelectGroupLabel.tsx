import { Select, SelectGroup, SelectGroupLabel, SelectItem, SelectPopup, SelectSeparator, SelectTrigger, SelectValue } from "frontend";

const MATERIALS: Record<string, string> = {
    d32: "D32 Steel",
    bipolar: "Bipolar Nanoflake",
    ceu: "Crystalline Electronic Unit",
    grindstone: "Grindstone Pentahydrate",
    rma24: "RMA70-24",
    polymer: "Polymerization Preparation",
    orirock: "Orirock Cluster",
    sugar: "Sugar Pack",
};

const SERVERS: Record<string, string> = {
    en: "EN (Yostar)",
    jp: "JP (Yostar)",
    kr: "KR (Yostar)",
    cn: "CN (Hypergryph)",
    bili: "CN (Bilibili)",
};

export const MaterialTiers = () => (
    <div className="flex h-96 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="grouplabel-materials">
            Material to farm
        </label>
        <Select defaultOpen defaultValue="bipolar">
            <SelectTrigger id="grouplabel-materials">
                <SelectValue placeholder="Material">{(value: string) => MATERIALS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Tier 5 — Elite</SelectGroupLabel>
                    <SelectItem value="d32">D32 Steel</SelectItem>
                    <SelectItem value="bipolar">Bipolar Nanoflake</SelectItem>
                    <SelectItem value="ceu">Crystalline Electronic Unit</SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectGroupLabel>Tier 4 — Advanced</SelectGroupLabel>
                    <SelectItem value="grindstone">Grindstone Pentahydrate</SelectItem>
                    <SelectItem value="rma24">RMA70-24</SelectItem>
                    <SelectItem value="polymer">Polymerization Preparation</SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectGroupLabel>Tier 3 — Common</SelectGroupLabel>
                    <SelectItem value="orirock">Orirock Cluster</SelectItem>
                    <SelectItem value="sugar">Sugar Pack</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);

export const RegionHeaders = () => (
    <div className="flex h-96 w-64 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="grouplabel-servers">
            Sync roster from
        </label>
        <Select defaultOpen defaultValue="en">
            <SelectTrigger id="grouplabel-servers">
                <SelectValue placeholder="Server">{(value: string) => SERVERS[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Global</SelectGroupLabel>
                    <SelectItem value="en">EN (Yostar)</SelectItem>
                    <SelectItem value="jp">JP (Yostar)</SelectItem>
                    <SelectItem value="kr">KR (Yostar)</SelectItem>
                </SelectGroup>
                <SelectSeparator />
                <SelectGroup>
                    <SelectGroupLabel>Mainland</SelectGroupLabel>
                    <SelectItem value="cn">CN (Hypergryph)</SelectItem>
                    <SelectItem value="bili">CN (Bilibili)</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);

export const LongLabel = () => (
    <div className="flex h-80 w-72 flex-col items-start gap-1.5">
        <label className="block font-medium text-[12px] text-muted-foreground leading-none" htmlFor="grouplabel-long">
            Tier list version
        </label>
        <Select defaultOpen defaultValue="v12">
            <SelectTrigger id="grouplabel-long">
                <SelectValue placeholder="Version">{(value: string) => ({ v12: "v12 — current", v11: "v11 — Lonetrail", v10: "v10 — Ideal City" })[value] ?? value}</SelectValue>
            </SelectTrigger>
            <SelectPopup alignItemWithTrigger={false}>
                <SelectGroup>
                    <SelectGroupLabel>Community-voted rankings, updated weekly</SelectGroupLabel>
                    <SelectItem value="v12">v12 — current</SelectItem>
                    <SelectItem value="v11">v11 — Lonetrail</SelectItem>
                    <SelectItem value="v10">v10 — Ideal City</SelectItem>
                </SelectGroup>
            </SelectPopup>
        </Select>
    </div>
);
