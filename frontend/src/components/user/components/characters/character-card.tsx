import Image from "next/image";
import { Card, CardContent } from "~/components/ui/card";
import { Dialog, DialogContent, DialogTrigger } from "~/components/ui/dialog";
import { Progress } from "~/components/ui/progress";
import { Separator } from "~/components/ui/separator";
import { formatProfession, formatSkillType, formatSubProfession, insertBlackboard, parseSkillStaticLevel } from "~/helper";
import type { CharacterData } from "~/types/impl/api";
import CharacterDialogueCard from "./character-dialogue-card";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/hover-card";
import { cn } from "~/lib/utils";
import { OperatorRarity } from "~/types/impl/api/static/operator";

function CharacterCard({ data }: { data: CharacterData }) {
    return (
        <Dialog>
            <DialogTrigger className="w-full">
                <Card className="group relative h-[400px] w-full max-w-sm overflow-hidden bg-card transition-all duration-300 hover:bg-secondary/50">
                    <CardContent className="p-0">
                        <div className="relative h-full w-full">
                            <div className="relative h-[400px] w-full">
                                <Image loading="lazy" className="object-contain" alt="Operator Image" src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${data.skin ? encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_")) : encodeURIComponent((data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").includes("@") ? (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("#", "_"))}.png`} fill sizes="(max-width: 768px) 100vw, (max-width: 1200px) 50vw, 33vw" style={{ objectFit: "contain" }} />
                            </div>
                            <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent opacity-0 transition-opacity duration-300 group-hover:opacity-100" />
                            <div className="absolute bottom-0 left-0 right-0 p-4 text-white">
                                <h2 className="text-xl font-bold">{data.static?.name}</h2>
                                <p className="text-sm text-gray-200">
                                    {formatProfession(data.static?.profession ?? "")} | {formatSubProfession(data.static?.subProfessionId ?? "")}
                                </p>
                            </div>
                            <div className="absolute bottom-0 left-0 right-0">
                                <div className={`h-1 w-full ${data.static?.rarity === OperatorRarity.sixStar ? "bg-[#f7a452]" : data.static?.rarity === OperatorRarity.fiveStar ? "bg-[#f7e79e]" : data.static?.rarity === OperatorRarity.fourStar ? "bg-[#bcabdb]" : data.static?.rarity === OperatorRarity.threeStar ? "bg-[#88c8e3]" : data.static?.rarity === OperatorRarity.twoStar ? "bg-[#7ef2a3]" : "bg-white"}`} />
                                <div className={`h-2 w-full blur-sm ${data.static?.rarity === OperatorRarity.sixStar ? "bg-[#cc9b6a]" : data.static?.rarity === OperatorRarity.fiveStar ? "bg-[#d6c474]" : data.static?.rarity === OperatorRarity.fourStar ? "bg-[#9e87c7]" : data.static?.rarity === OperatorRarity.threeStar ? "bg-[#62a2bd]" : data.static?.rarity === OperatorRarity.twoStar ? "bg-[#57ab72]" : "bg-gray-500"}`} />
                            </div>
                        </div>
                    </CardContent>
                </Card>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
                <CharacterDialogueCard data={data} />
            </DialogContent>
        </Dialog>
    );
}

function CharacterHoverCard({ data }: { data: CharacterData }) {
    return (
        <HoverCard>
            <HoverCardTrigger asChild>
                <div>
                    <CharacterCard data={data} />
                </div>
            </HoverCardTrigger>
            <HoverCardContent className="w-80">
                <div className="space-y-4">
                    <div className="flex items-center justify-between">
                        <div className="space-y-1">
                            <h4 className="text-sm font-semibold">Promotion</h4>
                            <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${data.evolvePhase}.png`} width={35} height={35} alt="Promotion" className="object-contain" />
                        </div>
                        <div className="space-y-1">
                            <h4 className="text-sm font-semibold">Potential</h4>
                            <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${data.potentialRank + 1}.png`} width={40} height={40} alt="Potential" className="object-contain" />
                        </div>
                    </div>

                    <Separator />

                    <div className="space-y-2">
                        <div>
                            <h4 className="text-sm font-semibold">Recruited</h4>
                            <p className="text-sm text-muted-foreground">{new Date(data.gainTime * 1000).toLocaleString()}</p>
                        </div>
                        <div>
                            <h4 className="text-sm font-semibold">Trust</h4>
                            <div className="flex items-center gap-2">
                                <Progress value={(data.static?.trust ?? 0) / 2} className="w-[60%]" />
                                <span className="text-sm text-muted-foreground">{data.static?.trust}%</span>
                            </div>
                        </div>
                    </div>

                    <Separator />

                    <div className="space-y-2">
                        <h4 className="text-sm font-semibold">Skills</h4>
                        <div className="space-y-3">
                            {data.skills.map((skill, index) => (
                                <div key={`skill-${index}`} className="space-y-1">
                                    <div className="flex items-center gap-2">
                                        <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skill/skill_icon_${skill.static?.iconId ?? skill.static?.skillId}.png`} width={35} height={35} alt="Skill" className="object-contain" />
                                        <div>
                                            <p className={cn("text-sm font-medium", data.defaultSkillIndex === index && "text-blue-500")}>{skill.static?.levels[data.mainSkillLvl - 1]?.name}</p>
                                            <div className="flex items-center gap-1">
                                                <span className="text-xs text-muted-foreground">Level {data.mainSkillLvl}</span>
                                                {skill.specializeLevel > 0 && <Image src={`/m-${skill.specializeLevel}_0.webp`} width={20} height={20} alt="M1" className="object-contain" />}
                                            </div>
                                        </div>
                                    </div>
                                    <p className="text-xs text-muted-foreground">
                                        <span className="font-medium">{formatSkillType(skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.spData.spType ?? "")}</span> | Initial: {skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.spData.initSp ?? 0} SP | Cost: {skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.spData.spCost ?? 0} SP
                                    </p>
                                    <p
                                        className="text-xs text-muted-foreground"
                                        dangerouslySetInnerHTML={{
                                            __html:
                                                insertBlackboard(
                                                    skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.description ?? "",
                                                    skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.blackboard?.concat({
                                                        key: "duration",
                                                        value: skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.duration ?? 0,
                                                        valueStr: skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.duration?.toString() ?? "",
                                                    }) ?? [],
                                                ) ?? "",
                                        }}
                                    />
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}

export default CharacterHoverCard;
