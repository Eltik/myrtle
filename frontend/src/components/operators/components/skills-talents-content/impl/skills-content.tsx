import { Zap } from "lucide-react";
import { useState } from "react";
import { Card, CardHeader, CardTitle } from "~/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import type { Operator } from "~/types/impl/api/static/operator";
import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown } from "lucide-react";
import { CardContent } from "~/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "~/components/ui/tabs";
import { Slider } from "~/components/ui/slider";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "~/components/ui/select";
import Image from "next/image";
import SkillCard from "./skill-card";

function SkillsContent({ operator }: { operator: Operator }) {
    const [skillLevel, setSkillLevel] = useState((operator.skills[0]?.static?.levels ?? []).length - 1);
    const [isSkillsOpen, setIsSkillsOpen] = useState(true);

    // Animation variants
    const motionVariants = {
        open: { opacity: 1, height: "auto" },
        collapsed: { opacity: 0, height: 0 },
    };

    // Calculate the grid template columns based on the number of skills
    const getGridColumns = (skillCount: number) => {
        switch (skillCount) {
            case 1:
                return "grid-cols-1";
            case 2:
                return "grid-cols-2";
            case 3:
                return "grid-cols-3";
            case 4:
                return "grid-cols-4";
            default:
                return "grid-cols-3"; // Fallback to 3 columns
        }
    };

    return (
        <>
            <Collapsible defaultOpen={true} open={isSkillsOpen} onOpenChange={setIsSkillsOpen}>
                <Card>
                    <CollapsibleTrigger className="w-full cursor-pointer" asChild>
                        <CardHeader className="flex w-full flex-row items-center justify-between space-y-0 p-4">
                            <CardTitle className="flex items-center gap-2">
                                <Zap className="h-5 w-5 text-blue-400" />
                                Skills
                            </CardTitle>
                            <ChevronDown className={`ml-auto h-6 w-6 transition-transform ${isSkillsOpen ? "rotate-180" : ""}`} />
                        </CardHeader>
                    </CollapsibleTrigger>
                    <AnimatePresence initial={false}>
                        {isSkillsOpen && (
                            <motion.div key="skills-content" initial="collapsed" animate="open" exit="collapsed" variants={motionVariants} transition={{ duration: 0.3, ease: [0.04, 0.62, 0.23, 0.98] }}>
                                <CollapsibleContent forceMount>
                                    <CardContent>
                                        <Tabs defaultValue={operator.skills[operator.skills.length - 1]?.skillId} className="w-full">
                                            <TabsList className={`grid w-full ${getGridColumns(operator.skills.length)}`}>
                                                {operator.skills.map((skill) => (
                                                    <TabsTrigger value={skill.skillId} key={skill.skillId} className="truncate px-2 text-sm">
                                                        <span className="truncate" title={skill.static?.levels[0]?.name}>
                                                            {skill.static?.levels[0]?.name}
                                                        </span>
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
                                                                src={`/m-${skillLevel === 7 ? "1" : skillLevel === 8 ? "2" : "3"}_0.webp`}
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
                                                    {/* Damage Calculation */}
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
        </>
    );
}

export default SkillsContent;
