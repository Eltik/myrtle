import Image from "next/image";
import { Card, CardContent } from "~/components/ui/card";
import { Dialog, DialogContent, DialogTrigger } from "~/components/ui/dialog";
import { Progress } from "~/components/ui/progress";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Separator } from "~/components/ui/separator";
import { formatProfession, formatSkillType, formatSubProfession, insertBlackboard, parseSkillStaticLevel } from "~/helper";
import type { CharacterData } from "~/types/impl/api";
import CharacterDialogueCard from "./character-dialogue-card";

function CharacterCard({ data }: { data: CharacterData }) {
    return (
        <Dialog>
            <DialogTrigger>
                <Card className="h-[620px] w-full max-w-sm overflow-hidden bg-card transition-all duration-150 hover:bg-secondary/50 md:h-[400px]">
                    <CardContent className="text-left">
                        <ScrollArea className="h-[620px] pb-6 md:h-[400px]">
                            <div className="grid gap-3 md:grid-cols-2">
                                <div className="relative h-full w-full">
                                    <Image
                                        loading="lazy"
                                        className="invisible hidden h-full w-full rounded-lg md:visible md:block"
                                        alt="Operator Image"
                                        src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${data.skin ? encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_")) : encodeURIComponent((data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").includes("@") ? (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("#", "_"))}.png`}
                                        fill
                                        sizes="100vw"
                                        style={{
                                            objectFit: "contain",
                                        }}
                                        decoding="async"
                                    />
                                    <Image
                                        loading="lazy"
                                        className="visible block h-full w-full rounded-lg md:invisible md:hidden"
                                        alt="Operator Image"
                                        width={500}
                                        height={500}
                                        src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/characters/${data.skin ? encodeURIComponent(data.skin.includes("@") ? data.skin.replaceAll("@", "_") : data.skin.replaceAll("#", "_")) : encodeURIComponent((data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").includes("@") ? (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("#", "_"))}.png`}
                                        style={{
                                            maxWidth: "100%",
                                            height: "auto",
                                            objectFit: "contain",
                                        }}
                                        decoding="async"
                                    />
                                </div>
                                <div className="space-y-4 py-3">
                                    <div className="space-y-1">
                                        <h2 className="text-2xl font-bold">{data.static?.name}</h2>
                                        <p className="text-sm text-gray-500 dark:text-gray-400">
                                            {formatProfession(data.static?.profession ?? "")} | {formatSubProfession(data.static?.subProfessionId ?? "")}
                                        </p>
                                    </div>
                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="">
                                            <p className="text-xs text-gray-500 dark:text-gray-400">Promotion</p>
                                            <Image
                                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${data.evolvePhase}.png`}
                                                width={35}
                                                height={35}
                                                alt="Promotion"
                                                style={{
                                                    maxWidth: "100%",
                                                    height: "auto",
                                                    objectFit: "contain",
                                                }}
                                            />
                                        </div>
                                        <div className="space-y-1">
                                            <p className="text-xs text-gray-500 dark:text-gray-400">Potential</p>
                                            <Image
                                                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${data.potentialRank + 1}.png`}
                                                width={40}
                                                height={40}
                                                alt="Potential"
                                                style={{
                                                    maxWidth: "100%",
                                                    height: "auto",
                                                    objectFit: "contain",
                                                }}
                                            />
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
                                                <Progress value={(data.static?.trust ?? 0) / 2} className="w-[60%]" />
                                                {data.static?.trust}%
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div className="flex w-full flex-col gap-5">
                                <div className="space-y-1 text-left">
                                    <span className="font-bold">Skills</span>
                                    <Separator />
                                </div>
                                <div className="flex h-full flex-row flex-wrap gap-4 text-left">
                                    {data.skills.length > 0 ? (
                                        data.skills.map((skill, index) => (
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
                                                            objectFit: "contain",
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
                                                                src={`/m-${skill.specializeLevel}_0.webp`}
                                                                className="h-8 w-8"
                                                                width={50}
                                                                height={50}
                                                                alt="M1"
                                                                style={{
                                                                    maxWidth: "100%",
                                                                    height: "auto",
                                                                    objectFit: "contain",
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
                            </div>
                        </ScrollArea>
                    </CardContent>
                </Card>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
                <CharacterDialogueCard data={data} />
            </DialogContent>
        </Dialog>
    );
}
export default CharacterCard;
