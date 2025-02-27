import { OperatorRarity, type Operator } from "~/types/impl/api/static/operator";
import Link from "next/link";
import Image from "next/image";
import { formatProfession } from "~/helper";

/**
 * @author https://wuwatracker.com/resonator
 * A lot of credit to them! They have an amazing design and I copied almost all of it for this.
 * I will be changing it in the future but for now, it's a good placeholder.
 */

export function OperatorsGrid({ operators }: { operators: Operator[] }) {
    return (
        <>
            <div className="grid grid-cols-3 gap-4 sm:grid-cols-4 lg:grid-cols-5 lg:gap-6 xl:grid-cols-6 xl:gap-8">
                {operators.map((operator) => {
                    if (!operator.id?.startsWith("char")) return null;
                    return (
                        <Link href={`/operators?id=${operator.id}`} key={operator.id} className="group relative flex aspect-[2/3] overflow-clip rounded-md border border-muted/50 bg-card transition hover:rounded-lg">
                            <div className="absolute -translate-x-8 -translate-y-4">
                                <Image src={operator.nationId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${String(operator.nationId)}.png` : operator.teamId ? `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/logo_${operator.teamId}.png` : `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/factions/none.png`} alt={String(operator.nationId)} loading="lazy" width={360} height={360} decoding="async" className="opacity-5 transition-opacity group-hover:opacity-10" />
                            </div>
                            <div className="absolute inset-0">
                                <div className="relative h-full w-full scale-100 transition-all duration-150 group-hover:scale-105 grayscale group-hover:grayscale-0">
                                    <Image
                                        loading="lazy"
                                        className="h-full w-full rounded-lg object-contain"
                                        alt="Operator Image"
                                        src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/portrait/${operator.id ?? ""}_1.png`}
                                        fill
                                        decoding="async"
                                    />
                                </div>
                            </div>
                            <div className="absolute inset-x-0 bottom-0 z-10">
                                <div className="relative">
                                    <div className="h-16 w-full bg-background/80 backdrop-blur-sm" />
                                    <h2 className="absolute bottom-2 left-1 line-clamp-2 text-xl font-bold uppercase opacity-60 transition-opacity group-hover:opacity-100 pr-12 max-w-[85%]">{operator.name}</h2>
                                    <div className="absolute bottom-2 right-1 flex scale-75 items-center opacity-0 transition-all duration-200 group-hover:scale-100 group-hover:opacity-100">
                                        <div className="h-6 w-6 md:h-10 md:w-10">
                                            <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${formatProfession(operator.profession).toLowerCase()}.png`} alt={formatProfession(operator.profession)} loading="lazy" width={160} height={160} decoding="async" />
                                        </div>
                                    </div>
                                    <div className={`absolute bottom-0 h-1 w-full ${operator.rarity === OperatorRarity.sixStar ? "bg-[#f7a452]" : operator.rarity === OperatorRarity.fiveStar ? "bg-[#f7e79e]" : operator.rarity === OperatorRarity.fourStar ? "bg-[#bcabdb]" : operator.rarity === OperatorRarity.threeStar ? "bg-[#88c8e3]" : operator.rarity === OperatorRarity.twoStar ? "bg-[#7ef2a3]" : "bg-white"} grayscale group-hover:grayscale-0`} />
                                    <div className={`absolute -bottom-1 h-2 w-full blur-sm ${operator.rarity === OperatorRarity.sixStar ? "bg-[#cc9b6a]" : operator.rarity === OperatorRarity.fiveStar ? "bg-[#d6c474]" : operator.rarity === OperatorRarity.fourStar ? "bg-[#9e87c7]" : operator.rarity === OperatorRarity.threeStar ? "bg-[#62a2bd]" : operator.rarity === OperatorRarity.twoStar ? "bg-[#57ab72]" : "bg-gray-500"}`} />
                                </div>
                            </div>
                        </Link>
                    );
                })}
            </div>
        </>
    );
}
