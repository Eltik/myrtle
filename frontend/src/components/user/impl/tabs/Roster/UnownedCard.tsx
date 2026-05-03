import { eliteIcon, operatorElite0 } from "#/components/operators/detail/impl/assets";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { Card, CardContent } from "#/components/ui/card";
import { Progress } from "#/components/ui/progress";
import { Separator } from "#/components/ui/separator";
import { formatProfession, getAvatarById } from "#/lib/utils";
import { parseOperatorName, rarityIcon } from "./helpers.card";
import type { IUnownedEntry, ViewMode } from "./types";

interface IUnownedCardProps {
    entry: IUnownedEntry;
    viewMode: ViewMode;
    lastRef?: ((node: HTMLElement | null) => void) | null;
}

export function UnownedCard({ entry, viewMode, lastRef }: IUnownedCardProps) {
    return <div ref={lastRef ?? undefined}>{viewMode === "detailed" ? <UnownedDetailed entry={entry} /> : <UnownedCompact entry={entry} />}</div>;
}

function UnownedCompact({ entry }: { entry: IUnownedEntry }) {
    const rarityColor = RARITY_COLORS[entry.rarity] ?? "#ffffff";
    const { displayName, subtitle } = parseOperatorName(entry.name);
    const nameIsLong = displayName.split(" ").length > 1 && displayName.length >= 16;

    return (
        <div
            className="fade-in slide-in-from-bottom-2 relative flex h-min w-min animate-in flex-col rounded bg-card opacity-60 grayscale"
            style={{
                padding: "4px 8px 4px 6px",
                margin: "2px 4px 4px 10px",
                boxShadow: "0 1px 3px 0 rgb(0 0 0 / 0.1)",
            }}
        >
            <div className="ml-px flex h-4.25 flex-col justify-center text-left sm:h-5">
                {subtitle && <span className="z-10 text-[0.4375rem] text-foreground leading-normal sm:text-[0.5625rem] sm:leading-loose">{subtitle}</span>}
                <span className="z-10 text-foreground" style={{ fontSize: nameIsLong ? "9px" : "12px", lineHeight: nameIsLong ? "9px" : "17px" }}>
                    {displayName}
                </span>
            </div>

            <div className="relative box-content aspect-square h-20 sm:h-30" style={{ borderBottom: `4px solid ${rarityColor}` }}>
                <img alt={entry.name} className="h-full w-full object-contain" decoding="async" height={120} loading="lazy" src={getAvatarById(entry.operator_id)} width={120} />
            </div>
        </div>
    );
}

function UnownedDetailed({ entry }: { entry: IUnownedEntry }) {
    const star = entry.rarity;
    const profession = entry.meta.profession;
    const heroSrc = operatorElite0(entry.operator_id, entry.static?.skin ?? null, entry.static?.portrait ?? null);

    return (
        <Card className="fade-in slide-in-from-bottom-4 flex w-full animate-in flex-col gap-0 overflow-hidden border-2 border-muted/30 py-0 pb-1 opacity-60 grayscale transition-all duration-300 hover:border-muted hover:opacity-80 hover:shadow-lg">
            <div className="relative">
                <div className="relative h-64 w-full overflow-hidden">
                    <img alt={entry.name} className="h-full w-full object-contain object-top" decoding="async" loading="lazy" src={heroSrc} />
                    <div className="absolute inset-0 bg-linear-to-t from-black/50 to-transparent opacity-70" />
                    <div className="absolute right-0 bottom-0 left-0 p-4">
                        <h3 className="mt-2 max-w-3/4 text-left font-bold text-white text-xl">{entry.name}</h3>
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2">
                                <img alt={`${star} Star`} className="h-4.5 w-auto object-contain" decoding="async" height={18} loading="lazy" src={rarityIcon(star)} width={60} />
                                <div className="flex flex-row items-center gap-1">
                                    <ClassIcon profession={profession} size={20} />
                                    <span className="text-sm text-white">{formatProfession(profession)}</span>
                                </div>
                            </div>
                            <img alt="Elite 0" className="h-6 w-6 object-contain" decoding="async" height={24} loading="lazy" src={eliteIcon(0)} width={24} />
                        </div>
                    </div>
                </div>

                <div className="absolute top-2 z-10 rounded-r-md bg-muted/80 px-2 py-0.5 text-center font-semibold text-muted-foreground text-xs shadow-md">Not Owned</div>
            </div>

            <CardContent className="min-w-0 flex-1 overflow-hidden px-4 pt-2 pb-2">
                <div className="grid grid-cols-2 gap-4">
                    <BlankBar label="Level" />
                    <BlankBar label="Trust" />
                </div>

                <Separator className="my-3" />

                <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
                    {["HP", "ATK", "DEF", "RES", "Cost", "Block"].map((label) => (
                        <div className="flex items-center justify-between" key={label}>
                            <span className="text-muted-foreground">{label}</span>
                            <span className="font-medium text-muted-foreground">--</span>
                        </div>
                    ))}
                </div>

                <Separator className="my-3" />

                <div className="w-full">
                    {["Potential", "Skills", "Modules"].map((label) => (
                        <div className="flex items-center justify-between border-b-0 py-2" key={label}>
                            <span className="font-medium text-muted-foreground/50 text-sm">{label}</span>
                        </div>
                    ))}
                </div>
            </CardContent>
        </Card>
    );
}

function BlankBar({ label }: { label: string }) {
    return (
        <div>
            <div className="flex items-center justify-between">
                <span className="font-medium text-sm">{label}</span>
                <span className="font-bold text-muted-foreground text-sm">--</span>
            </div>
            <Progress className="h-1.5" value={0} />
        </div>
    );
}
