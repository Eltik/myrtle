import { AnimatePresence, motion } from "framer-motion";
import { ChevronDown, Zap } from "lucide-react";
import Image from "next/image";
import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { Slider } from "~/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import type { Operator } from "~/types/impl/api/static/operator";
import SkillCard from "./skill-card";

function SkillsContent({ operator }: { operator: Operator }) {
    const initialSkillLevel = operator.skills?.[0]?.static?.levels?.length ? operator.skills[0].static.levels.length - 1 : 0;
    const [skillLevel, setSkillLevel] = useState(initialSkillLevel);
    const [isSkillsOpen, setIsSkillsOpen] = useState(true);

    // If there are no skills, don't render anything
    if (!operator.skills || operator.skills.length === 0) {
        return null;
    }

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
        <Collapsible defaultOpen={true} onOpenChange={setIsSkillsOpen} open={isSkillsOpen}>
            <Card>
                <CollapsibleTrigger asChild className="w-full cursor-pointer">
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
                        <motion.div animate="open" exit="collapsed" initial="collapsed" key="skills-content" transition={{ duration: 0.3, ease: [0.04, 0.62, 0.23, 0.98] }} variants={motionVariants}>
                            <CollapsibleContent forceMount>
                                <CardContent>
                                    <Tabs className="w-full" defaultValue={operator.skills[operator.skills.length - 1]?.skillId}>
                                        <TabsList className={`grid w-full ${getGridColumns(operator.skills.length)}`}>
                                            {operator.skills.map((skill) => (
                                                <TabsTrigger className="truncate px-2 text-sm" key={skill.skillId} value={skill.skillId}>
                                                    <span className="truncate" title={skill.static?.levels[0]?.name}>
                                                        {skill.static?.levels[0]?.name}
                                                    </span>
                                                </TabsTrigger>
                                            ))}
                                        </TabsList>
                                        <div className="mt-4 mb-6 flex flex-col">
                                            <div className="flex flex-col">
                                                <span className="font-bold text-lg">Skill Level</span>
                                                <span className="text-muted-foreground text-sm">Drag the slider to change the skill level</span>
                                            </div>
                                            <div className="flex max-w-[80%] flex-col items-center gap-2 md:flex-row">
                                                <Slider className="w-full" defaultValue={[skillLevel]} max={(operator.skills[0]?.static?.levels ?? []).length - 1} min={0} onValueChange={(value) => setSkillLevel(value[0] ?? 0)} step={1} value={[skillLevel]} />
                                                <div className="flex w-full flex-row">
                                                    <Select onValueChange={(e) => setSkillLevel(parseInt(e, 10))} value={skillLevel.toString()}>
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
                                                            alt="M1"
                                                            className="h-8 w-9"
                                                            height={50}
                                                            src={`/m-${skillLevel === 7 ? "1" : skillLevel === 8 ? "2" : "3"}_0.webp`}
                                                            style={{
                                                                maxWidth: "100%",
                                                                height: "auto",
                                                            }}
                                                            width={50}
                                                        />
                                                    )}
                                                </div>
                                            </div>
                                        </div>
                                        {operator.skills.map((skill) => (
                                            <TabsContent key={skill.skillId} value={skill.skillId}>
                                                <SkillCard level={skillLevel} skill={skill} />
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
    );
}

export default SkillsContent;
