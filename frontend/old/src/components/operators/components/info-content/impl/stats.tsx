import { BadgeDollarSign, CircleGauge, Cross, Diamond, Hourglass, Shield, ShieldBan, Swords } from "lucide-react";
import Image from "next/image";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import type { Operator } from "~/types/impl/api/static/operator";

export const Stats = ({ operator, attributeStats, setPhaseIndex, setLevel }: { operator: Operator; attributeStats: Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null; setPhaseIndex: (index: number) => void; setLevel: (level: number) => void }) => {
    return (
        <Tabs
            className="mt-4 w-full"
            defaultValue={`phase_${operator.phases.length - 1}`}
            onValueChange={(value) => {
                setPhaseIndex(parseInt(value.split("_")[1] ?? "0", 10));
                setLevel(operator.phases[parseInt(value.split("_")[1] ?? "0", 10)]?.maxLevel ?? 1);
            }}
        >
            <div className="rounded-sm bg-muted pt-1">
                <TabsList>
                    {operator.phases.map((_, index) => (
                        <TabsTrigger className="data-[state=active]:bg-card/70" key={index} value={`phase_${index}`}>
                            <Image
                                alt="Promotion"
                                height={35}
                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${index}.png`}
                                style={{
                                    maxWidth: "100%",
                                    height: "auto",
                                    objectFit: "contain",
                                }}
                                width={35}
                            />
                        </TabsTrigger>
                    ))}
                </TabsList>
            </div>
            <div>
                {operator.phases.map((_, index) => (
                    <TabsContent key={index} value={`phase_${index}`}>
                        <div className="grid grid-cols-1 gap-2 text-sm sm:grid-cols-2 md:text-base">
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <Cross size={24} />
                                    <span className="ml-2 text-muted-foreground">Health</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.maxHp ?? 0)}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <Swords size={24} />
                                    <span className="ml-2 text-muted-foreground">ATK</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.atk ?? 0)}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <Shield size={24} />
                                    <span className="ml-2 text-muted-foreground">DEF</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.def ?? 0)}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <CircleGauge size={24} />
                                    <span className="ml-2 text-muted-foreground">ATK Interval</span>
                                </div>
                                <span className="font-bold">{attributeStats?.attackSpeed.toFixed(2) ?? 0}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <Diamond size={24} />
                                    <span className="ml-2 text-muted-foreground">RES</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.magicResistance ?? 0)}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <ShieldBan size={24} />
                                    <span className="ml-2 text-muted-foreground">Block</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.blockCnt ?? 0)}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <Hourglass size={24} />
                                    <span className="ml-2 text-muted-foreground">Redeploy</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.respawnTime ?? 0)}</span>
                            </div>
                            <div className="flex flex-row items-center justify-between rounded-md bg-muted p-[12px_16px]">
                                <div className="flex items-center">
                                    <BadgeDollarSign size={24} />
                                    <span className="ml-2 text-muted-foreground">DP Cost</span>
                                </div>
                                <span className="font-bold">{Math.round(attributeStats?.cost ?? 0)}</span>
                            </div>
                        </div>
                    </TabsContent>
                ))}
            </div>
        </Tabs>
    );
};
