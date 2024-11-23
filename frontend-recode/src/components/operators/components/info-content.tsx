import Image from "next/image";
import { Separator } from "~/components/ui/separator";
import { formatProfession, formatSubProfession, getAvatarById } from "~/helper";
import type { Operator } from "~/types/impl/api/static/operator";

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
                                <div className="box-border grid h-full grid-flow-col items-center bg-muted p-[8px_12px]">
                                    <div className="max-full box-border inline-block h-8 w-8">
                                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/subclass/sub_${operator.subProfessionId}_icon.png`} alt={formatSubProfession(operator.subProfessionId)} loading="lazy" width={160} height={160} decoding="async" />
                                    </div>
                                    <span className="text-sm font-medium">{formatSubProfession(operator.subProfessionId)}</span>
                                </div>
                            </div>
                        </div>
                        <div className="m-0 p-0">
                            <div className="h-[calc(104px)] w-[calc(104px)] rounded-md border transition-all duration-150 hover:bg-secondary">
                                <Image src={getAvatarById(operator.id ?? "")} alt={operator.name} width={160} height={160} loading="lazy" decoding="async" />
                            </div>
                        </div>
                    </div>
                    <div></div>
                </div>
            </div>
        </div>
    );
}

export default InfoContent;
