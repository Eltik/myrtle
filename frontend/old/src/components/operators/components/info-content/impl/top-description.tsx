import { AnimatePresence, motion } from "framer-motion";
import { Activity, Cake, ChevronDown, FolderPen, MapPinHouse, Ruler, User, Users } from "lucide-react";
import { useState } from "react";
import { Badge } from "~/components/ui/badge";
import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";

export const TopDescription = ({ operator }: { operator: Operator }) => {
    const [isDescriptionExpanded, setIsDescriptionExpanded] = useState(false);

    return (
        <>
            <div className="relative">
                <AnimatePresence initial={false}>
                    <motion.div animate={{ height: isDescriptionExpanded ? "auto" : "180px" }} className="overflow-hidden" exit={{ height: "180px" }} initial={{ height: "80px" }} transition={{ duration: 0.3 }}>
                        <div className="text-sm md:text-base">
                            <p>{operator.itemUsage}</p>
                            <p>{operator.itemDesc}</p>
                        </div>
                        {operator.profile && (
                            <>
                                <Separator className="mt-2" />
                                <div className="mt-2 flex w-full flex-col gap-3">
                                    <div className="grid gap-2 md:grid-cols-2">
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <FolderPen className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Code Name:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.codeName}</span>
                                            </div>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <Cake className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Date of Birth:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.dateOfBirth}</span>
                                            </div>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <MapPinHouse className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Place of Birth:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.placeOfBirth}</span>
                                            </div>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <User className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Gender:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.gender}</span>
                                            </div>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <Ruler className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Height:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.height}</span>
                                            </div>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <Users className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Race:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.race}</span>
                                            </div>
                                        </div>
                                        <div className="space-y-2">
                                            <div className="flex items-center space-x-2 overflow-hidden">
                                                <Activity className="h-4 w-4 flex-shrink-0" />
                                                <span className="flex-shrink-0 text-muted-foreground text-xs md:text-sm">Combat Exp:</span>
                                                <span className="truncate font-medium text-sm md:text-base">{operator.profile?.basicInfo.combatExperience}</span>
                                            </div>
                                        </div>
                                    </div>
                                    <Separator />
                                    <div>
                                        <h3 className="mb-2 font-semibold text-lg">Physical Examination</h3>
                                        <div className="grid gap-2 md:grid-cols-2">
                                            {Object.entries(operator.profile?.physicalExam ?? {}).map(([key, value]) => (
                                                <Badge className="justify-between overflow-hidden text-sm" key={key} variant="secondary">
                                                    <span className="truncate font-semibold capitalize">{key.replace(/([A-Z])/g, " $1").trim()}</span>
                                                    <span className="ml-1 truncate font-normal">{value}</span>
                                                </Badge>
                                            ))}
                                        </div>
                                    </div>
                                </div>
                            </>
                        )}
                    </motion.div>
                </AnimatePresence>
                {!isDescriptionExpanded && <div className="absolute right-0 bottom-0 left-0 h-16 bg-gradient-to-t from-background to-transparent" />}
            </div>
            <span className="flex cursor-pointer flex-row items-center self-start text-sm transition-all duration-150 hover:text-muted-foreground" onClick={() => setIsDescriptionExpanded(!isDescriptionExpanded)}>
                {isDescriptionExpanded ? "Show Less" : "Show More"}
                <ChevronDown className={`ml-auto h-5 w-5 transition-transform ${isDescriptionExpanded ? "rotate-180" : ""}`} />
            </span>
        </>
    );
};
