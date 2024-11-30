import Image from "next/image";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { capitalize, formatSkillType } from "~/helper";
import { descriptionToHtml } from "~/helper/descriptionParser";
import { type Operator } from "~/types/impl/api/static/operator";
import OperatorRange from "../../operator-range";
import type { Range } from "~/types/impl/api/static/ranges";
import { useEffect, useState } from "react";

function SkillCard({ skill, level }: { skill: Operator["skills"][0]; level: number }) {
    const [currentRange, setCurrentRange] = useState<Range | null>(null);

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
                                color: (skill.static?.levels[level]?.spData.spType ?? "") === "INCREASE_WITH_TIME" ? "#a7e855" : (skill.static?.levels[level]?.spData.spType ?? "") === "INCREASE_WHEN_ATTACK" ? "#f98d3f" : "#ffcf53",
                            }}
                        >
                            {formatSkillType(skill.static?.levels[level]?.spData.spType ?? "")} Recovery
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
