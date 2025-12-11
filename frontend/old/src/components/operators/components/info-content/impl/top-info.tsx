import { Star } from "lucide-react";
import Image from "next/image";
import { formatGroupId, formatNationId, formatProfession, formatSubProfession, getAvatarById, rarityToNumber } from "~/helper";
import type { Operator } from "~/types/impl/api/static/operator";
import { OperatorPosition } from "~/types/impl/api/static/operator";

export const TopInfoContent = ({ operator }: { operator: Operator }) => {
    return (
        <div className="flex w-full flex-col md:grid md:items-center md:gap-4 lg:grid-cols-[auto,1fr,auto]">
            <div className="flex flex-row-reverse justify-end">
                <div className="grid grid-cols-[max-content,minmax(0,1fr)] grid-rows-[max-content,max-content] gap-2 px-3 align-baseline sm:px-5">
                    <div className="col-span-2">
                        <span className="font-bold text-2xl md:text-4xl">{operator.name}</span>
                    </div>
                    <div className="flex items-center border">
                        <div className="bg-card p-2">
                            <div className="max-full box-border inline-block h-8 w-8">
                                <Image alt={formatProfession(operator.profession)} decoding="async" height={160} loading="lazy" src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${formatProfession(operator.profession).toLowerCase()}.png`} width={160} />
                            </div>
                        </div>
                        <div className="box-border grid h-full grid-flow-col items-center gap-2 bg-muted p-[8px_12px]">
                            <div className="max-full box-border inline-block h-8 max-h-8 w-8">
                                <Image alt={formatSubProfession(operator.subProfessionId)} decoding="async" height={160} loading="lazy" src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/subclass/sub_${operator.subProfessionId}_icon.png`} width={160} />
                            </div>
                            <span className="truncate font-medium text-sm">{formatSubProfession(operator.subProfessionId)}</span>
                        </div>
                    </div>
                </div>
                <div className="m-0 p-0">
                    <div className="relative mb-3 flex h-[104px] w-[104px] items-center justify-center rounded-md border bg-muted/50 backdrop-blur-lg transition-all duration-150 hover:bg-secondary">
                        <div>
                            <Image alt={operator.name} decoding="async" height={160} loading="lazy" src={getAvatarById(operator.id ?? "")} width={160} />
                        </div>
                        <div className="-bottom-5 absolute left-0 mb-3 flex w-full justify-center">
                            <div className="flex flex-row items-center justify-center">{operator.rarity && Array.from({ length: rarityToNumber(operator.rarity) }).map((_, index) => <Star fill="#ed9634" key={index} size={16} stroke="#000000" strokeWidth={2} />)}</div>
                        </div>
                    </div>
                </div>
            </div>
            <div className="mt-4 grid h-[max-content] grid-cols-3 gap-2 md:mt-0 md:justify-end">
                <div className="flex flex-col justify-between p-2 md:p-4">
                    <span className="text-muted-foreground text-sm">Nation</span>
                    <span className="truncate font-normal text-sm md:text-lg">{operator.nationId && String(operator.nationId).length > 0 ? formatNationId(String(operator.nationId)) : "N/A"}</span>
                </div>
                <div className="flex flex-col justify-between p-2 md:p-4">
                    <span className="text-muted-foreground text-sm">Faction</span>
                    <span className="truncate font-normal text-sm md:text-lg">{operator.groupId && operator.groupId.length > 0 ? formatGroupId(operator.groupId) : "N/A"}</span>
                </div>
                <div className="flex flex-col justify-between p-2 md:p-4">
                    <span className="text-muted-foreground text-sm">Position</span>
                    <span className="font-normal text-sm md:text-lg">{operator.position === OperatorPosition.MELEE ? "Melee" : "Ranged"}</span>
                </div>
            </div>
        </div>
    );
};
