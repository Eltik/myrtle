import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import type { Operator } from "~/types/impl/api/static/operator";
import SkillCard from "./impl/skill-card";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { ChevronDown } from "lucide-react";

function SkillsContent({ operator }: { operator: Operator }) {
    const [skillLevel, setSkillLevel] = useState(7);
    const [isSkillsOpen, setIsSkillsOpen] = useState(true);

    return (
        <>
            <div>
                <div className="p-2 px-4 backdrop-blur-2xl">
                    <span className="text-3xl font-bold">Skills & Talents</span>
                </div>
                <Separator />
                <div className="grid px-3 md:p-4">
                    <div className="space-y-8 md:col-span-2">
                        <Collapsible defaultOpen={true} open={isSkillsOpen} onOpenChange={setIsSkillsOpen}>
                            <Card>
                                <CollapsibleTrigger className="w-full cursor-pointer" asChild>
                                    <CardHeader className="flex w-full flex-row items-center justify-between space-y-0 p-4">
                                        <CardTitle>Skills</CardTitle>
                                        <ChevronDown className={`ml-auto h-6 w-6 transition-transform ${isSkillsOpen ? "rotate-180" : ""}`} />
                                    </CardHeader>
                                </CollapsibleTrigger>
                                <CollapsibleContent>
                                    <CardContent>
                                        <Tabs defaultValue={operator.skills[operator.skills.length - 1]?.skillId} className="w-full">
                                            <TabsList className={`grid w-full grid-cols-${operator.skills.length}`}>
                                                {operator.skills.map((skill) => (
                                                    <TabsTrigger value={skill.skillId} key={skill.skillId}>
                                                        {skill.static?.levels[0]?.name}
                                                    </TabsTrigger>
                                                ))}
                                            </TabsList>
                                            {operator.skills.map((skill) => (
                                                <TabsContent value={skill.skillId} key={skill.skillId}>
                                                    <SkillCard skill={skill} level={skillLevel} />
                                                </TabsContent>
                                            ))}
                                        </Tabs>
                                    </CardContent>
                                </CollapsibleContent>
                            </Card>
                        </Collapsible>
                    </div>
                </div>
            </div>
        </>
    );
}

export default SkillsContent;
