import { ArrowUpDown, Cog, Users } from "lucide-react";
import Image from "next/image";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "~/components/ui/card";
import { Progress } from "~/components/ui/progress";
import { Separator } from "~/components/ui/separator";
import { getAvatarById } from "~/helper";
import type { User } from "~/types/impl/api";

type FactoryProps = {
    data: User["building"]["rooms"]["MANUFACTURE"][string];
    chars: User["building"]["chars"];
    roomSlot: User["building"]["roomSlots"][string];
    roomId: string;
};

function Factory({ data, chars, roomSlot, roomId }: FactoryProps) {
    const assignedOperators = Object.values(chars).filter((char) => char.roomSlotId === roomId);
    return (
        <Card className="w-[360px] bg-card shadow-lg">
            <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                    <CardTitle className="font-bold text-lg">Factory</CardTitle>
                    <Badge className="cursor-pointer" variant="secondary">
                        Lv. {roomSlot.level}
                    </Badge>
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                        <Image alt="Product" height={24} src="/path/to/product-icon.png" width={24} />
                        <span className="font-medium text-sm">{data.formulaId === "4" ? "Gold" : data.formulaId === "3" ? "Exp. Cards" : "Unknown"}</span>
                    </div>
                    <div className="flex items-center space-x-2">
                        <span className="font-medium text-sm">
                            {data.remainSolutionCnt}/{data.capacity}
                        </span>
                    </div>
                </div>
                <Progress className="h-2 w-full bg-zinc-700" value={(data.processPoint / (data.completeWorkTime - data.lastUpdateTime)) * 100} />
                <div className="grid grid-cols-3 gap-2">
                    {[0, 1, 2].map((slot) => (
                        <Card className="bg-muted/50 p-2" key={slot}>
                            <div className="flex flex-col items-center justify-center">
                                {assignedOperators[slot] ? (
                                    <Image alt={assignedOperators[slot].charId} className="rounded-full" height={32} src={getAvatarById(assignedOperators[slot].charId)} width={32} />
                                ) : (
                                    <>
                                        <Users className="h-8 w-8 text-blue-400" />
                                        <span className="mt-1 text-xs">Empty</span>
                                    </>
                                )}
                            </div>
                        </Card>
                    ))}
                </div>
                <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                        <span className="font-medium text-sm">Efficiency:</span>
                        <span className="text-sm">{(data.buff.speed + data.buff.sSpeed).toFixed(2)}%</span>
                    </div>
                    <div className="flex items-center space-x-2">
                        <span className="font-medium text-sm">Drones:</span>
                        <span className="text-sm">{data.apCost}</span>
                    </div>
                </div>
            </CardContent>
            <Separator className="bg-secondary" />
            <CardFooter className="justify-between pt-4">
                <Button className="w-[48%] bg-secondary hover:bg-background" size="sm" variant="outline">
                    <ArrowUpDown className="mr-2 h-4 w-4" />
                    Trade
                </Button>
                <Button className="w-[48%] bg-secondary hover:bg-background" size="sm" variant="outline">
                    <Cog className="mr-2 h-4 w-4" />
                    Manage
                </Button>
            </CardFooter>
        </Card>
    );
}

export default Factory;
