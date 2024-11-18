import Image from "next/image";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import type { CharacterData } from "~/types/impl/api";
import { Progress } from "../ui/progress";
import { formatProfession, formatSkillType, formatSubProfession, insertBlackboard, parseSkillStaticLevel } from "~/helper";
import { Badge } from "../ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { ScrollArea } from "../ui/scroll-area";
import type { OperatorRarity } from "~/types/impl/api/static/operator";

function CharacterDialogueCard({ data }: { data: CharacterData }) {
    const { static: operatorData, level, evolvePhase, skills } = data;

    if (!operatorData) return null;

    const { name, appellation, rarity, profession, subProfessionId, description } = operatorData;

    const getRarityStars = (rarity: OperatorRarity) => {
        const starCount = parseInt(rarity.split("_")[1] ?? "0");
        return "★".repeat(starCount);
    };

    return (
        <Card className="grid max-h-[calc(100vh-7rem)] w-full max-w-2xl gap-6 overflow-hidden rounded-lg border-0 shadow-lg md:py-6">
            <CardHeader className="relative">
                <div className="relative h-64 w-full">
                    <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${data.skin ? encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_")) : encodeURIComponent((data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").includes("@") ? (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("#", "_"))}.png`} alt={name} layout="fill" objectFit="contain" className="h-48 w-full rounded-t-lg" />
                </div>
                <div className="absolute bottom-0 left-0 right-0 rounded-md bg-gradient-to-t from-gray-900 to-transparent p-4">
                    <CardTitle className="text-3xl font-bold">{name}</CardTitle>
                    <p className="text-lg">{appellation}</p>
                    <div className="mt-2 flex items-center space-x-2">
                        <Badge variant="outline" className="bg-card">
                            {getRarityStars(rarity)}
                        </Badge>
                        <Badge className="cursor-pointer">{formatProfession(profession)}</Badge>
                        <Badge className="cursor-pointer">{formatSubProfession(subProfessionId)}</Badge>
                    </div>
                </div>
            </CardHeader>
            <CardContent>
                <Tabs defaultValue="info" className="w-full">
                    <TabsList className="grid w-full grid-cols-4">
                        <TabsTrigger value="info">Info</TabsTrigger>
                        <TabsTrigger value="stats">Stats</TabsTrigger>
                        <TabsTrigger value="skills">Skills</TabsTrigger>
                        <TabsTrigger value="talents">Talents</TabsTrigger>
                    </TabsList>
                    <TabsContent value="info">
                        <ScrollArea className="h-[200px] w-full rounded-md border p-4">
                            <div className="space-y-2">
                                <div className="flex flex-col gap-4">
                                    <div>
                                        <h3 className="font-semibold">Description</h3>
                                        <p className="text-sm text-muted-foreground">{insertBlackboard(description, [])}</p>
                                    </div>
                                    <div className="flex flex-col">
                                        <h3 className="font-semibold">Recruited</h3>
                                        <span className="text-sm text-muted-foreground">{new Date(data.gainTime * 1000).toLocaleString()}</span>
                                    </div>
                                </div>
                                <div>
                                    <h3 className="font-semibold">Trust</h3>
                                    <div className="flex flex-row items-center justify-start gap-2">
                                        <Progress value={(data.static?.trust ?? 0) / 2} max={100} className="w-[60%]" />
                                        <p className="text-sm text-muted-foreground">{(data.static?.trust ?? 0) / 2}%</p>
                                    </div>
                                </div>
                            </div>
                        </ScrollArea>
                    </TabsContent>
                    <TabsContent value="stats">
                        <ScrollArea className="h-[200px] w-full rounded-md border p-4">
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <p>Level: {level}</p>
                                    <p>Elite: {evolvePhase}</p>
                                </div>
                                <div>
                                    <p>HP: {operatorData.phases[evolvePhase]?.attributesKeyFrames[0]?.data.maxHp}</p>
                                    <p>ATK: {operatorData.phases[evolvePhase]?.attributesKeyFrames[0]?.data.atk}</p>
                                    <p>DEF: {operatorData.phases[evolvePhase]?.attributesKeyFrames[0]?.data.def}</p>
                                </div>
                            </div>
                        </ScrollArea>
                    </TabsContent>
                    <TabsContent value="skills">
                        <ScrollArea className="h-[200px] w-full rounded-md border p-4">
                            <div className="flex flex-col gap-6">
                                {skills.length > 0 ? (
                                    skills.map((skill, index) => (
                                        <div className="space-y-1" key={`skill-${index}`}>
                                            <div className="flex w-full flex-row items-center gap-2">
                                                <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skill/skill_icon_${skill.static?.iconId ?? skill.static?.skillId}.png`} width={35} height={35} alt="Skill" />
                                                <div className="text-md">
                                                    <b className={`${data.defaultSkillIndex === index ? "text-blue-200" : "text-inherit"}`}>{skill.static?.levels[data.mainSkillLvl - 1]?.name}</b>
                                                </div>
                                            </div>
                                            <div className="flex flex-col">
                                                <span className="text-base">
                                                    Level {data.mainSkillLvl}
                                                    {skill.specializeLevel > 0 ? `, M${skill.specializeLevel}` : ""}
                                                </span>
                                                <span className="mb-2 text-sm">
                                                    <b>{formatSkillType(skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.spData.spType ?? "")}</b> | <b>Initial: </b>
                                                    <span className="text-muted-foreground">{skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.spData.initSp ?? 0} SP</span> - <b>Cost: </b>
                                                    <span className="text-muted-foreground">{skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.spData.spCost ?? 0} SP</span>
                                                </span>
                                                <span
                                                    className="text-xs"
                                                    dangerouslySetInnerHTML={{
                                                        __html: insertBlackboard(skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.description ?? "", skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.blackboard?.concat({ key: "duration", value: skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.duration ?? 0, valueStr: skill.static?.levels[parseSkillStaticLevel(data.mainSkillLvl, skill.specializeLevel)]?.duration?.toString() ?? "" }) ?? []) ?? "",
                                                    }}
                                                ></span>
                                            </div>
                                        </div>
                                    ))
                                ) : (
                                    <div className="text-sm text-gray-500 dark:text-gray-400">No skills found.</div>
                                )}
                            </div>
                        </ScrollArea>
                    </TabsContent>
                    <TabsContent value="talents">
                        <ScrollArea className="h-[200px] w-full rounded-md border p-4">
                            {operatorData.talents.map((talent, index) => (
                                <div key={index} className="mb-4">
                                    <h3 className="font-semibold">{talent.candidates[0]?.name}</h3>
                                    <p className="text-sm text-muted-foreground">{talent.candidates[0]?.description}</p>
                                </div>
                            ))}
                        </ScrollArea>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    );
}

export default CharacterDialogueCard;
