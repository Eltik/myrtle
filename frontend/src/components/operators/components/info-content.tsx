import { Star } from "lucide-react";
import Image from "next/image";
import { Separator } from "~/components/ui/separator";
import { formatGroupId, formatNationId, formatProfession, formatSubProfession, getAvatarById, rarityToNumber } from "~/helper";
import { OperatorPosition, type Operator } from "~/types/impl/api/static/operator";

// https://aceship.github.io/AN-EN-Tags/akhrchars.html?opname=Projekt_Red
// https://sanitygone.help/operators/gavial-the-invincible#page-content

function InfoContent({ operator }: { operator: Operator }) {
    return (
        <div>
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-3xl font-bold">Operator Info</span>
            </div>
            <Separator />
            <div className="p-4">
                <div className="grid w-full grid-cols-[max-content,1fr,max-content] items-center">
                    <div className="flex flex-row-reverse justify-end">
                        <div className="grid grid-cols-[max-content,1fr] grid-rows-[max-content,max-content] gap-2 px-5 align-baseline">
                            <div className="col-span-2">
                                <span className="text-4xl font-bold">{operator.name}</span>
                            </div>
                            <div className="flex items-center border">
                                <div className="bg-card p-2">
                                    <div className="max-full box-border inline-block h-8 w-8">
                                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${formatProfession(operator.profession).toLowerCase()}.png`} alt={formatProfession(operator.profession)} loading="lazy" width={160} height={160} decoding="async" />
                                    </div>
                                </div>
                                <div className="box-border grid h-full grid-flow-col items-center gap-2 bg-muted p-[8px_12px]">
                                    <div className="max-full box-border inline-block h-8 max-h-8 w-8">
                                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/subclass/sub_${operator.subProfessionId}_icon.png`} alt={formatSubProfession(operator.subProfessionId)} loading="lazy" width={160} height={160} decoding="async" />
                                    </div>
                                    <span className="text-sm font-medium">{formatSubProfession(operator.subProfessionId)}</span>
                                </div>
                            </div>
                        </div>
                        <div className="m-0 p-0">
                            <div className="relative mb-3 flex h-[calc(104px)] w-[calc(104px)] items-center justify-center rounded-md border bg-muted/50 backdrop-blur-lg transition-all duration-150 hover:bg-secondary">
                                <div>
                                    <Image src={getAvatarById(operator.id ?? "")} alt={operator.name} width={160} height={160} loading="lazy" decoding="async" />
                                </div>
                                <div className="absolute -bottom-5 left-0 mb-3 flex w-full justify-center">
                                    <div className="flex flex-row items-center justify-center">{operator.rarity && Array.from({ length: rarityToNumber(operator.rarity) }).map((_, index) => <Star key={index} size={16} fill="#ed9634" strokeWidth={2} stroke="#000000" />)}</div>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="grid h-[max-content] grid-cols-[repeat(3,max-content)] justify-end gap-x-6">
                        <div className="flex flex-col justify-between p-4">
                            <span className="text-sm text-muted-foreground">Nation</span>
                            <span className="text-lg font-normal">{operator.nationId && operator.nationId.length > 0 ? formatNationId(operator.nationId) : "N/A"}</span>
                        </div>
                        <div className="flex flex-col justify-between p-4">
                            <span className="text-sm text-muted-foreground">Faction</span>
                            <span className="text-lg font-normal">{operator.groupId && operator.groupId.length > 0 ? formatGroupId(operator.groupId) : "N/A"}</span>
                        </div>
                        <div className="flex flex-col justify-between p-4">
                            <span className="text-sm text-muted-foreground">Position</span>
                            <span className="text-lg font-normal">{operator.position === OperatorPosition.MELEE ? "Melee" : "Ranged"}</span>
                        </div>
                    </div>
                </div>
                <div className="mt-3 block">
                    <div>
                        <p>{operator.itemUsage}</p>
                        <p>{operator.itemDesc}</p>
                    </div>
                    <div className=""></div>
                </div>
            </div>
        </div>
    );
}

export default InfoContent;
