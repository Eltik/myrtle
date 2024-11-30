import { ArrowUpDown, Cog, Users } from "lucide-react";
import Image from "next/image";
import { Badge } from "~/components/ui/badge";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "~/components/ui/card";
import { Progress } from "~/components/ui/progress";
import { Separator } from "~/components/ui/separator";
import { getAvatarById } from "~/helper";
import type { User } from "~/types/impl/api";

type TradingPostProps = {
    data: User["building"]["rooms"]["TRADING"][string];
    chars: User["building"]["chars"];
    roomSlot: User["building"]["roomSlots"][string];
    roomId: string;
};

function TradingPost({ data, chars, roomSlot, roomId }: TradingPostProps) {
    const stockCount = data.stock.length;
    const stockLimit = data.stockLimit;
    const efficiency = data.buff.speed * 100;
    const orderProgress = Math.round((data.next.processPoint / data.next.maxPoint) * 100);

    const assignedOperators = Object.values(chars).filter((char) => char.roomSlotId === roomId);

    return (
        <>
            <Card className="w-[360px] bg-card shadow-lg">
                <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                        <CardTitle className="text-lg font-bold">Trading Post</CardTitle>
                        <Badge variant="secondary" className="cursor-pointer">
                            Lv. {roomSlot.level}
                        </Badge>
                    </div>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center space-x-2">
                            <span className="text-sm font-medium">
                                Stock: {stockCount}/{stockLimit}
                            </span>
                        </div>
                        <div className="flex items-center space-x-2">
                            <span className="text-sm font-medium">Orders: {data.next.order}</span>
                        </div>
                    </div>
                    <Progress value={orderProgress} className="h-2 w-full bg-zinc-700" />
                    <div className="grid grid-cols-3 gap-2">
                        {[0, 1, 2].map((slot) => (
                            <Card key={slot} className="bg-muted/50 p-2">
                                <div className="flex flex-col items-center justify-center">
                                    {assignedOperators[slot] ? (
                                        <>
                                            <Image src={getAvatarById(assignedOperators[slot].charId)} alt={assignedOperators[slot].charId} width={32} height={32} className="rounded-full" />
                                        </>
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
                            <span className="text-sm font-medium">Efficiency: {efficiency}%</span>
                        </div>
                        <div className="flex items-center space-x-2">
                            <span className="text-sm font-medium">
                                {data.apCost}/{data.buff.apCost.all}
                            </span>
                        </div>
                    </div>
                </CardContent>
                <Separator className="bg-secondary" />
                <CardFooter className="justify-between pt-4">
                    <Button variant="outline" size="sm" className="w-[48%] bg-secondary hover:bg-background">
                        <ArrowUpDown className="mr-2 h-4 w-4" />
                        Trade
                    </Button>
                    <Button variant="outline" size="sm" className="w-[48%] bg-secondary hover:bg-background">
                        <Cog className="mr-2 h-4 w-4" />
                        Manage
                    </Button>
                </CardFooter>
            </Card>
        </>
    );
}

export default TradingPost;
