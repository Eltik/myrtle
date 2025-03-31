import Image from "next/image";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { capitalize, formatSkillType } from "~/helper";
import { descriptionToHtml } from "~/helper/descriptionParser";
import { type Operator } from "~/types/impl/api/static/operator";
import OperatorRange from "../../operator-range";
import type { Range } from "~/types/impl/api/static/ranges";
import { useEffect, useState } from "react";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";
import { ArrowDownIcon, ArrowUpIcon } from "lucide-react";

function SkillCard({ skill, level }: { skill: Operator["skills"][0]; level: number }) {
    const [currentRange, setCurrentRange] = useState<Range | null>(null);

    const skillLevel = skill.static?.levels[level];
    const blackboard = skillLevel?.blackboard ?? [];

    useEffect(() => {
        if (skill.static?.levels[level]?.rangeId) {
            void fetchRange(skill.static.levels[level].rangeId).then((range) => {
                setCurrentRange(range);
            });
        }
    }, [skill, level]);

    async function fetchRange(id: string) {
        const data = (await (
            await fetch("/api/static", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "ranges",
                    id,
                }),
            })
        ).json()) as { data: Range };
        return data.data;
    }

    const formatBlackboardValue = (key: string, value: number) => {
        const percentageKeys = ["atk", "def", "attack@atk_scale", "hp_recovery_per_sec", "attack@atk_scale_ol", "attack@prob"];
        if (percentageKeys.includes(key)) {
            const percentValue = Math.round(value * 100);
            return `${value >= 0 ? "+" : ""}${percentValue}%`;
        }
        if (key === "base_attack_time") {
            return `${value > 0 ? "+" : ""}${value.toFixed(1)}s`;
        }
        if (key === "shield_duration" || key === "stun_duration") {
            return `${value}s`;
        }
        if (key === "shield_scale") {
            return `${value}x`;
        }
        return value > 0 ? `+${value}` : value.toString();
    };

    return (
        <Card>
            <CardHeader>
                <CardTitle className="flex flex-row items-center gap-2 text-xl font-bold">
                    <Image
                        src={skill.static?.image ?? ""}
                        width={50}
                        height={50}
                        alt="Skill"
                        style={{
                            maxWidth: "100%",
                            height: "auto",
                        }}
                        className="rounded-sm"
                    />
                    <div className="flex flex-col justify-start">
                        {skill.static?.levels[level]?.name}
                        <span
                            className="text-base text-muted-foreground"
                            style={{
                                color: (skill.static?.levels[level]?.spData.spType ?? "") === 8 ? "#9c9a9a" : (skill.static?.levels[level]?.spData.spType ?? "") === "INCREASE_WITH_TIME" ? "#a7e855" : (skill.static?.levels[level]?.spData.spType ?? "") === "INCREASE_WHEN_ATTACK" ? "#f98d3f" : "#ffcf53",
                            }}
                        >
                            {formatSkillType(skill.static?.levels[level]?.spData.spType ?? "") !== "Passive" ? formatSkillType(skill.static?.levels[level]?.spData.spType ?? "") + " Recovery" : "Passive"}
                        </span>
                    </div>
                </CardTitle>
                <CardDescription>
                    <span
                        dangerouslySetInnerHTML={{
                            __html: descriptionToHtml(skill.static?.levels[level]?.description ?? "", skill.static?.levels[level]?.blackboard ?? []),
                        }}
                    ></span>
                </CardDescription>
            </CardHeader>
            <CardContent>
                <div className="grid grid-cols-2">
                    <div className="rounded-tl-md border p-2">
                        <p className="flex flex-row items-center gap-1 text-sm text-muted-foreground">
                            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg" className="inline-block">
                                <path d="M2 13V1L12 7L2 13Z" fill="#D6D6E2" />
                            </svg>
                            SP Cost
                        </p>
                        <p className="font-bold">{skill.static?.levels[level]?.spData.spCost}</p>
                    </div>
                    <div className="rounded-tr-md border p-2">
                        <p className="flex flex-row items-center gap-1 text-sm text-muted-foreground">
                            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" xmlns="http://www.w3.org/2000/svg" className="inline-block">
                                <path d="M1 8.4L8.5 0L7.3 5.6H13L5.5 14L6.7 8.4H1Z" fill="#a7e855" />
                            </svg>
                            Initial SP
                        </p>
                        <p className="font-bold">{skill.static?.levels[level]?.spData.initSp}</p>
                    </div>
                    <div className="rounded-bl-md border p-2">
                        <p className="text-sm text-muted-foreground">Duration</p>
                        <p className="font-bold">{skill.static?.levels[level]?.duration === 0 ? "Instant" : skill.static?.levels[level]?.duration === -1 ? "Ammo" : `${skill.static?.levels[level]?.duration}s`}</p>
                    </div>
                    <div className="rounded-br-md border p-2">
                        <p className="text-sm text-muted-foreground">Skill Type</p>
                        <p className="font-bold">{capitalize(skill.static?.levels[level]?.skillType ?? "")}</p>
                    </div>
                </div>
                <div>
                    <h2 className="mb-1 mt-4 text-lg font-bold">Stats Change</h2>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
                        {blackboard.map((bb) => (
                            <TooltipProvider key={bb.key}>
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <div className="cursor-pointer rounded-lg bg-secondary px-2 py-1 transition-all hover:bg-secondary/80">
                                            <div className="flex items-center justify-between">
                                                <span className="truncate text-sm font-medium">{bb.key}</span>
                                                {(bb.value > 0 && bb.key !== "base_attack_time") || (bb.key === "base_attack_time" && bb.value < 0) ? <ArrowUpIcon className="h-4 w-4 text-green-500" /> : <ArrowDownIcon className="h-4 w-4 text-red-500" />}
                                            </div>
                                            <p className="mt-1 font-bold">{formatBlackboardValue(bb.key, bb.value)}</p>
                                        </div>
                                    </TooltipTrigger>
                                    <TooltipContent>
                                        <p>Change in {bb.key}</p>
                                    </TooltipContent>
                                </Tooltip>
                            </TooltipProvider>
                        ))}
                    </div>
                </div>
                {currentRange && (
                    <div className="mt-2 p-2">
                        <p className="text-sm text-muted-foreground">Range</p>
                        <OperatorRange range={currentRange} />
                    </div>
                )}
            </CardContent>
        </Card>
    );
}

export default SkillCard;
