import Image from "next/image";
import { Card, CardContent } from "./ui/card";
import { type CharacterData } from "~/types/types";
import { Progress } from "./ui/progress";
import { Separator } from "./ui/separator";

function CharacterCard({ data }: { data: CharacterData }) {
    return (
        <Card className="w-full max-w-2xl md:py-6 grid gap-6 rounded-lg overflow-hidden shadow-lg border-0 overflow-y-scroll max-h-[calc(100vh-7rem)]">
            <CardContent>
                <div className="grid md:grid-cols-2 gap-3">
                    <div className="relative">
                        <Image className="w-full h-full object-cover rounded-lg" alt="Operator Image"
                            width={500}
                            height={500}
                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${data.skin.replaceAll("#", "_")}.png`}
                        />
                    </div>
                    <div className="space-y-4">
                        <div className="space-y-1">
                            <h2 className="text-2xl font-bold">{data.static.name}</h2>
                            <p className="text-sm text-gray-500 dark:text-gray-400">Faction | Class</p>
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
                                <p className="text-xs text-gray-500 dark:text-gray-400">Trust</p>
                                <div className="flex flex-row items-center gap-3">
                                    <Progress value={data.static.trust} className="w-[60%]" />
                                    {data.static.trust}%
                                </div>
                            </div>
                        </div>
                    </div>
                    <div className="flex flex-col gap-5 w-full">
                        <div className="space-y-1">
                            <span className="font-bold">Skills</span>
                            <Separator />
                        </div>
                        <div className="flex flex-row flex-wrap gap-4">
                            {data.skills.reverse().map((skill, index) => (
                                <div className="space-y-1" key={`skill-${index}`}>
                                    <div className="flex flex-row gap-2 w-full items-center">
                                        <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/skills/skill_icon_${skill.static.iconId ?? skill.static.skillId}.png`} width={35} height={35} alt="Skill" />
                                        <div className="">
                                            <span>Level {data.mainSkillLvl}, M{skill.specializeLevel}</span>
                                        </div>
                                    </div>
                                    <div>
                                        <p>
                                            <b className={`${data.defaultSkillIndex === index ? "text-blue-200" : "text-inherit"}`}>
                                                {skill.static.levels[data.mainSkillLvl - 1]?.name}
                                            </b>
                                        </p>
                                        <span className="text-xs line-clamp-2">{skill.static.description}</span>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
};

export default CharacterCard;