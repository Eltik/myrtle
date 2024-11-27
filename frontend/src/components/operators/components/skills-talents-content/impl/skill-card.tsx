import Image from "next/image";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/card";
import { descriptionToHtml } from "~/helper/descriptionParser";
import { type Operator } from "~/types/impl/api/static/operator";

function SkillCard({ skill, level }: { skill: Operator["skills"][0]; level: number }) {
    return (
        <Card>
            <CardHeader>
                <CardTitle className="flex flex-row items-center gap-2 text-xl font-bold">
                    <Image
                        src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skill/skill_icon_${skill.static?.iconId ?? skill.static?.skillId}.png`}
                        width={35}
                        height={35}
                        alt="Skill"
                        style={{
                            maxWidth: "100%",
                            height: "auto",
                        }}
                        className="rounded-sm"
                    />
                    {skill.static?.levels[level]?.name}
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
                        <p className="text-sm text-gray-400">SP Cost</p>
                        <p className="font-bold">{skill.static?.levels[level]?.spData.spCost}</p>
                    </div>
                    <div className="rounded-tr-md border p-2">
                        <p className="text-sm text-gray-400">Initial SP</p>
                        <p className="font-bold">{skill.static?.levels[level]?.spData.initSp}</p>
                    </div>
                    <div className="rounded-bl-md border p-2">
                        <p className="text-sm text-gray-400">Duration</p>
                        <p className="font-bold">{skill.static?.levels[level]?.duration}</p>
                    </div>
                    <div className="rounded-br-md border p-2">
                        <p className="text-sm text-gray-400">Skill Type</p>
                        <p className="font-bold">{skill.static?.levels[level]?.skillType}</p>
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}

export default SkillCard;
