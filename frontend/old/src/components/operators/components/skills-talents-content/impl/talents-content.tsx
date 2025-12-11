import { AnimatePresence, motion } from "framer-motion";
import { Award, ChevronDown, ChevronRight } from "lucide-react";
import Image from "next/image";
import { useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/collapsible";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";
import { parsePhase } from "~/helper";
import { descriptionToHtml } from "~/helper/descriptionParser";
import { type Operator, OperatorPhase } from "~/types/impl/api/static/operator";

function TalentsContent({ operator }: { operator: Operator }) {
    const [isTalentsOpen, setIsTalentsOpen] = useState(true);

    // Animation variants
    const motionVariants = {
        open: { opacity: 1, height: "auto" },
        collapsed: { opacity: 0, height: 0 },
    };

    return (
        <Collapsible defaultOpen={true} onOpenChange={setIsTalentsOpen} open={isTalentsOpen}>
            <Card className="overflow-hidden">
                <CollapsibleTrigger asChild className="w-full cursor-pointer">
                    <CardHeader className="flex w-full flex-row items-center justify-between space-y-0 bg-gradient-to-r from-card to-background/80 p-4 backdrop-blur-sm">
                        <CardTitle className="flex items-center gap-2">
                            <Award className="h-5 w-5 text-amber-500" />
                            Talents
                        </CardTitle>
                        <ChevronDown className={`ml-auto h-6 w-6 transition-transform ${isTalentsOpen ? "rotate-180" : ""}`} />
                    </CardHeader>
                </CollapsibleTrigger>
                <AnimatePresence initial={false}>
                    {isTalentsOpen && (
                        <motion.div animate="open" exit="collapsed" initial="collapsed" key="talents-content" transition={{ duration: 0.3, ease: [0.04, 0.62, 0.23, 0.98] }} variants={motionVariants}>
                            <CollapsibleContent forceMount>
                                <CardContent className="p-0">
                                    <div className="divide-y divide-border">
                                        {operator.talents?.map((talent, index) => (
                                            <div className="bg-gradient-to-r from-background to-background/80 p-4 transition-colors hover:from-background/90 hover:to-background/70" key={index}>
                                                <div className="mb-2 flex items-center">
                                                    <Badge className="mr-2 border-border bg-secondary/50 font-semibold" variant="outline">
                                                        Talent {index + 1}
                                                    </Badge>
                                                </div>

                                                {talent.candidates?.map((candidate, candidateIndex) => (
                                                    <div className={`mb-2 rounded-lg p-3 ${candidateIndex === talent.candidates.length - 1 ? "border border-border bg-secondary/50" : "bg-secondary/40"}`} key={candidateIndex}>
                                                        <div className="mb-2 flex items-center justify-between">
                                                            <h3 className="font-semibold text-base">{candidate.name}</h3>
                                                            <div className="flex items-center gap-2">
                                                                {candidate.unlockCondition && (
                                                                    <Badge className="text-xs" variant="secondary">
                                                                        {parsePhase(candidate.unlockCondition.phase) === OperatorPhase.ELITE_0
                                                                            ? "Elite 0"
                                                                            : parsePhase(candidate.unlockCondition.phase) === OperatorPhase.ELITE_1
                                                                              ? "Elite 1"
                                                                              : parsePhase(candidate.unlockCondition.phase) === OperatorPhase.ELITE_2
                                                                                ? "Elite 2"
                                                                                : "Unknown"}{" "}
                                                                        Lv{candidate.unlockCondition.level}
                                                                    </Badge>
                                                                )}
                                                                {candidate.requiredPotentialRank > 0 && (
                                                                    <Badge className="flex cursor-pointer items-center gap-1 border-border text-xs hover:bg-background" variant="outline">
                                                                        <Image
                                                                            alt="Potential"
                                                                            height={25}
                                                                            src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${candidate.requiredPotentialRank + 1}.png`}
                                                                            style={{
                                                                                maxWidth: "100%",
                                                                                height: "auto",
                                                                                objectFit: "contain",
                                                                            }}
                                                                            width={25}
                                                                        />
                                                                    </Badge>
                                                                )}
                                                            </div>
                                                        </div>
                                                        <div
                                                            className="text-muted-foreground text-sm leading-relaxed"
                                                            dangerouslySetInnerHTML={{
                                                                __html: descriptionToHtml(candidate.description, candidate.blackboard),
                                                            }}
                                                        />

                                                        {candidate.blackboard.length > 0 && (
                                                            <div className="mt-3 flex flex-wrap gap-2">
                                                                {candidate.blackboard.map((item, i) => (
                                                                    <Badge className="flex items-center gap-1 text-xs" key={i} variant="secondary">
                                                                        <TooltipProvider>
                                                                            <Tooltip>
                                                                                <TooltipTrigger asChild>
                                                                                    <span className="max-w-[50vw] truncate font-mono">{item.key}</span>
                                                                                </TooltipTrigger>
                                                                                <TooltipContent>
                                                                                    <p>{item.key}</p>
                                                                                </TooltipContent>
                                                                            </Tooltip>
                                                                        </TooltipProvider>
                                                                        <ChevronRight className="h-3 w-3 opacity-50" />
                                                                        <span className="font-semibold">{item.value < 0 ? item.value : `+${item.value}`}</span>
                                                                    </Badge>
                                                                ))}
                                                            </div>
                                                        )}
                                                    </div>
                                                ))}
                                            </div>
                                        ))}
                                        {(!operator.talents || operator.talents.length === 0) && (
                                            <div className="flex h-24 items-center justify-center">
                                                <p className="text-center text-muted-foreground">This operator has no talents</p>
                                            </div>
                                        )}
                                    </div>
                                </CardContent>
                            </CollapsibleContent>
                        </motion.div>
                    )}
                </AnimatePresence>
            </Card>
        </Collapsible>
    );
}

export default TalentsContent;
