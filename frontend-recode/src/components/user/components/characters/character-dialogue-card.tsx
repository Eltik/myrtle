import Image from "next/image";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import type { CharacterData } from "~/types/impl/api";
import { formatProfession, formatSkillType, formatSubProfession, getMaxAttributeStats, insertBlackboard, parseSkillStaticLevel, removeStyleTags } from "~/helper";
import type { OperatorRarity } from "~/types/impl/api/static/operator";
import { ChevronRight, Heart, Shield, Swords, Zap } from "lucide-react";
import { Badge } from "~/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Progress } from "~/components/ui/progress";
import { Skeleton } from "~/components/ui/skeleton";
import { useAttributeStats } from "~/hooks/use-attribute-stats";

function CharacterDialogueCard({ data }: { data: CharacterData }) {
    const { static: operatorData, skills } = data;
    const { attributeStats, fetchAttributeStats } = useAttributeStats(data);

    if (!operatorData) return null;

    const { name, appellation, rarity, profession, subProfessionId, description } = operatorData;

    const getRarityStars = (rarity: OperatorRarity) => {
        const starCount = parseInt(rarity.split("_")[1] ?? "0");
        return "★".repeat(starCount);
    };

    const handleTabChange = (value: string) => {
        if (value === "stats" && !attributeStats) {
            void fetchAttributeStats();
        }
    };

    return (
        <Card className="grid max-h-[calc(100vh-7rem)] w-full max-w-2xl gap-6 overflow-hidden rounded-lg border-0 shadow-lg md:py-6">
            <CardHeader className="relative">
                <div className="relative h-64 w-full">
                    <Image
                        loading="lazy"
                        src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${data.skin ? encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_")) : encodeURIComponent((data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").includes("@") ? (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("#", "_"))}.png`}
                        alt={name}
                        className="h-48 w-full rounded-t-lg"
                        fill
                        sizes="100vw"
                        style={{
                            objectFit: "contain",
                        }}
                    />
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
                <Tabs defaultValue="info" className="w-full" onValueChange={handleTabChange}>
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
                        {attributeStats ? (
                            <div className="grid grid-cols-2 gap-4">
                                <div>
                                    <h4 className="mb-2 font-semibold">Combat Stats</h4>
                                    <AttributeRow label="HP" value={Math.round(attributeStats?.maxHp ?? 0)} max={getMaxAttributeStats(data)?.maxHp} icon={<Heart className="h-4 w-4" />} />
                                    <AttributeRow label="ATK" value={Math.round(attributeStats?.atk ?? 0)} max={getMaxAttributeStats(data)?.atk} icon={<Swords className="h-4 w-4" />} />
                                    <AttributeRow label="DEF" value={Math.round(attributeStats?.def ?? 0)} max={getMaxAttributeStats(data)?.def} icon={<Shield className="h-4 w-4" />} />
                                    <AttributeRow label="RES" value={attributeStats?.magicResistance ?? 0} max={getMaxAttributeStats(data)?.magicResistance} icon={<Zap className="h-4 w-4" />} />
                                </div>
                                <div>
                                    <h4 className="mb-2 font-semibold">Deployment</h4>
                                    <AttributeRow label="Cost" value={attributeStats?.cost ?? 0} icon={<ChevronRight className="h-4 w-4" />} max={99} />
                                    <AttributeRow label="Block" value={attributeStats?.blockCnt ?? 0} icon={<ChevronRight className="h-4 w-4" />} max={5} />
                                    <AttributeRow label="Redeploy" value={attributeStats?.respawnTime ?? 0} icon={<ChevronRight className="h-4 w-4" />} max={100} />
                                </div>
                            </div>
                        ) : (
                            <>
                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <h4 className="mb-2 font-semibold">Combat Stats</h4>
                                        <AttributeRowSkeleton />
                                        <AttributeRowSkeleton />
                                        <AttributeRowSkeleton />
                                        <AttributeRowSkeleton />
                                    </div>
                                    <div>
                                        <h4 className="mb-2 font-semibold">Deployment</h4>
                                        <AttributeRowSkeleton />
                                        <AttributeRowSkeleton />
                                        <AttributeRowSkeleton />
                                    </div>
                                </div>
                            </>
                        )}
                    </TabsContent>
                    <TabsContent value="skills">
                        <ScrollArea className="h-[200px] w-full rounded-md border p-4">
                            <div className="flex flex-col gap-6">
                                {skills.length > 0 ? (
                                    skills.map((skill, index) => (
                                        <div className="space-y-1" key={`skill-${index}`}>
                                            <div className="flex w-full flex-row items-center gap-2">
                                                <Image
                                                    src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skill/skill_icon_${skill.static?.iconId ?? skill.static?.skillId}.png`}
                                                    width={35}
                                                    height={35}
                                                    alt="Skill"
                                                    style={{
                                                        maxWidth: "100%",
                                                        height: "auto",
                                                    }}
                                                />
                                                <div className="text-md">
                                                    <b className={`${data.defaultSkillIndex === index ? "text-blue-200" : "text-inherit"}`}>{skill.static?.levels[data.mainSkillLvl - 1]?.name}</b>
                                                </div>
                                            </div>
                                            <div className="flex flex-col">
                                                <div className="flex flex-row items-center">
                                                    <span className="text-base">Level {data.mainSkillLvl}</span>
                                                    {skill.specializeLevel > 0 ? (
                                                        <Image
                                                            src={`https://ak.gamepress.gg/sites/default/files/2019-10/m-${skill.specializeLevel}_0.png`}
                                                            className="h-8 w-8"
                                                            width={50}
                                                            height={50}
                                                            alt="M1"
                                                            style={{
                                                                maxWidth: "100%",
                                                                height: "auto",
                                                            }}
                                                        />
                                                    ) : null}
                                                </div>
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
                            {operatorData.talents.map((talent, index) => {
                                const parseCandidatePhase = (phase: string) => {
                                    if (phase === "PHASE_0") return 0;
                                    if (phase === "PHASE_1") return 1;
                                    if (phase === "PHASE_2") return 2;
                                    return 0;
                                };

                                let talentIndex = 0;
                                for (let i = 0; i < talent.candidates.length; i++) {
                                    const candidate = talent.candidates[i];
                                    if (data.evolvePhase >= parseCandidatePhase(candidate!.unlockCondition.phase) && data.level >= candidate!.unlockCondition.level && data.potentialRank >= candidate!.requiredPotentialRank) {
                                        talentIndex = i;
                                    }
                                }

                                const currentTalent = talent.candidates[talentIndex];

                                return (
                                    <div key={index} className="mb-4">
                                        <h3 className="font-semibold">{currentTalent?.name}</h3>
                                        <p className="text-sm text-muted-foreground">{removeStyleTags(currentTalent?.description ?? "N/A")}</p>
                                    </div>
                                );
                            })}
                        </ScrollArea>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    );
}

const getProgressColor = (value: number, max: number) => {
    const percentage = (value / max) * 100;
    if (percentage >= 80) return "bg-green-500";
    if (percentage >= 50) return "bg-yellow-500";
    return "bg-red-500";
};

const AttributeRow = ({ label, value, icon, max = 2000 }: { label: string; value: number; icon: React.ReactNode; max?: number }) => (
    <div className="mb-2 flex items-center space-x-2">
        {icon}
        <span className="w-24 text-sm">{label}</span>
        <Progress value={(value / max) * 100} className={`h-2 w-full ${getProgressColor(value, max)}`} />
        <span className="w-16 text-right text-sm">{value}</span>
    </div>
);

const AttributeRowSkeleton = () => (
    <div className="mb-2 flex items-center space-x-2">
        <Skeleton className="h-4 w-4" />
        <Skeleton className="h-4 w-24" />
        <Skeleton className="h-2 w-full" />
        <Skeleton className="h-4 w-16" />
    </div>
);

export default CharacterDialogueCard;
