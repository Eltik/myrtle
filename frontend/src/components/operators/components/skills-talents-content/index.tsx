import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import type { Operator } from "~/types/impl/api/static/operator";
import SkillCard from "./impl/skill-card";
import { useState } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { ChevronDown } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { Slider } from "~/components/ui/slider";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import Image from "next/image";

function SkillsContent({ operator }: { operator: Operator }) {
    const [skillLevel, setSkillLevel] = useState(9);
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
                                <AnimatePresence initial={false}>
                                    {isSkillsOpen && (
                                        <motion.div
                                            key="content"
                                            initial="collapsed"
                                            animate="open"
                                            exit="collapsed"
                                            variants={{
                                                open: { opacity: 1, height: "auto" },
                                                collapsed: { opacity: 0, height: 0 },
                                            }}
                                            transition={{ duration: 0.3, ease: [0.04, 0.62, 0.23, 0.98] }}
                                        >
                                            <CollapsibleContent forceMount>
                                                <CardContent>
                                                    <Tabs defaultValue={operator.skills[operator.skills.length - 1]?.skillId} className="w-full">
                                                        <TabsList className={`grid w-full grid-cols-${operator.skills.length}`}>
                                                            {operator.skills.map((skill) => (
                                                                <TabsTrigger value={skill.skillId} key={skill.skillId}>
                                                                    {skill.static?.levels[0]?.name}
                                                                </TabsTrigger>
                                                            ))}
                                                        </TabsList>
                                                        <div className="mb-6 mt-4 flex flex-col">
                                                            <div className="flex flex-col">
                                                                <span className="text-lg font-bold">Skill Level</span>
                                                                <span className="text-sm text-muted-foreground">Drag the slider to change the skill level</span>
                                                            </div>
                                                            <div className="flex max-w-[80%] flex-col items-center gap-2 md:flex-row">
                                                                <Slider className="w-full" defaultValue={[skillLevel]} value={[skillLevel]} onValueChange={(value) => setSkillLevel(value[0] ?? 0)} min={0} max={(operator.skills[0]?.static?.levels ?? []).length - 1} step={1} />
                                                                <div className="flex w-full flex-row">
                                                                    <Select value={skillLevel.toString()} onValueChange={(e) => setSkillLevel(parseInt(e))}>
                                                                        <SelectTrigger className="w-[180px]">
                                                                            <SelectValue placeholder={skillLevel} />
                                                                        </SelectTrigger>
                                                                        <SelectContent>
                                                                            {(operator.skills[0]?.static?.levels ?? []).map((_, i) => (
                                                                                <SelectItem key={i} value={i.toString()}>
                                                                                    {i < 7 ? <span>Lv.{i + 1}</span> : <span>M{i === 7 ? "1" : i === 8 ? "2" : "3"}</span>}
                                                                                </SelectItem>
                                                                            ))}
                                                                        </SelectContent>
                                                                    </Select>
                                                                    {skillLevel >= 7 && (
                                                                        <Image
                                                                            src={`https://ak.gamepress.gg/sites/default/files/2019-10/m-${skillLevel === 7 ? "1" : skillLevel === 8 ? "2" : "3"}_0.png`}
                                                                            className="h-8 w-9"
                                                                            width={50}
                                                                            height={50}
                                                                            alt="M1"
                                                                            style={{
                                                                                maxWidth: "100%",
                                                                                height: "auto",
                                                                            }}
                                                                        />
                                                                    )}
                                                                </div>
                                                            </div>
                                                        </div>
                                                        {operator.skills.map((skill) => (
                                                            <TabsContent value={skill.skillId} key={skill.skillId}>
                                                                <SkillCard skill={skill} level={skillLevel} />
                                                            </TabsContent>
                                                        ))}
                                                    </Tabs>
                                                </CardContent>
                                            </CollapsibleContent>
                                        </motion.div>
                                    )}
                                </AnimatePresence>
                            </Card>
                        </Collapsible>
                    </div>
                </div>
            </div>
        </>
    );
}

export default SkillsContent;
