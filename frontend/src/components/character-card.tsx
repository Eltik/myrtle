import Image from "next/image";
import { Card, CardContent } from "./ui/card";
import { type CharacterData } from "~/types/types";
import { Progress } from "./ui/progress";
import { Separator } from "./ui/separator";
import { capitalize } from "~/helper";

function CharacterCard({ data }: { data: CharacterData }) {
    return (
        <Card className="grid max-h-[calc(100vh-7rem)] w-full max-w-2xl gap-6 overflow-hidden overflow-y-scroll rounded-lg border-0 shadow-lg md:py-6">
            <CardContent>
                <div className="grid gap-3 md:grid-cols-2">
                    <div className="relative h-full w-full">
                        <Image className="h-full w-full rounded-lg object-cover" alt="Operator Image" width={500} height={500} src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_"))}.png`} />
                    </div>
                    <div className="space-y-4">
                        <div className="space-y-1">
                            <h2 className="text-2xl font-bold">{data.static?.name}</h2>
                            <p className="text-sm text-gray-500 dark:text-gray-400">
                                {capitalize(data.static?.profession)} | {capitalize(data.static?.subProfessionId)}
                            </p>
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                            <div className="">
                                <p className="text-xs text-gray-500 dark:text-gray-400">Promotion</p>
                                <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${data.evolvePhase}.png`} width={35} height={35} alt="Promotion" />
                            </div>
                            <div className="space-y-1">
                                <p className="text-xs text-gray-500 dark:text-gray-400">Potential</p>
                                <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${data.potentialRank + 1}.png`} width={40} height={40} alt="Potential" />
                            </div>
                        </div>
                        <div className="space-y-1">
                            <div className="flex flex-col">
                                <div className="pb-2">
                                    <p className="text-xs text-gray-500 dark:text-gray-400">Recruited</p>
                                    <p className="text-sm">{new Date(data.gainTime * 1000).toLocaleString()}</p>
                                </div>
                                <p className="text-xs text-gray-500 dark:text-gray-400">Trust</p>
                                <div className="flex flex-row items-center gap-3">
                                    <Progress value={data.static?.trust / 2} className="w-[60%]" />
                                    {data.static?.trust}%
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div className="flex w-full flex-col gap-5">
                    <div className="space-y-1">
                        <span className="font-bold">Skills</span>
                        <Separator />
                    </div>
                    <div className="flex flex-row flex-wrap gap-4">
                        {data.skills.map((skill, index) => (
                            <div className="space-y-1" key={`skill-${index}`}>
                                <div className="flex w-full flex-row items-center gap-2">
                                    <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/skills/skill_icon_${skill.static?.iconId ?? skill.static?.skillId}.png`} width={35} height={35} alt="Skill" />
                                    <div className="text-md">
                                        <span>
                                            Level {data.mainSkillLvl}, M{skill.specializeLevel}
                                        </span>
                                    </div>
                                </div>
                                <div>
                                    <p className="text-sm">
                                        <b className={`${data.defaultSkillIndex === index ? "text-blue-200" : "text-inherit"}`}>{skill.static?.levels[data.mainSkillLvl - 1]?.name}</b>
                                    </p>
                                    <span className="line-clamp-2 text-xs">{skill.static?.description}</span>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}

export default CharacterCard;
