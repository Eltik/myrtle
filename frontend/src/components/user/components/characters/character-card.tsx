import type { CharacterData } from "~/types/impl/api";
import { Progress } from "~/components/ui/progress";
import { Card, CardContent } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/accordion";
import { ScrollArea } from "~/components/ui/scroll-area";
import Image from "next/image";
import { formatProfession } from "~/helper";
import { useState, useEffect, useRef } from "react";
import { Dialog, DialogContent } from "~/components/ui/dialog";
import CharacterDialogueCard from "./character-dialogue-card";
import { descriptionToHtml } from "~/helper/descriptionParser";

function CharacterCard({ data }: { data: CharacterData }) {
    // Get operator static data
    const operator = data.static;

    // Calculate trust percentage
    const trustPercentage = operator ? (operator.trust / 200) * 100 : 0;

    // State for animation effects
    const [isHovered, setIsHovered] = useState(false);
    const [isLoaded, setIsLoaded] = useState(false);
    const [levelProgress, setLevelProgress] = useState(0);
    const [trustProgress, setTrustProgress] = useState(0);
    const [isDialogOpen, setIsDialogOpen] = useState(false);

    // Ref for the card element
    const cardRef = useRef<HTMLDivElement>(null);

    // Animation effect on mount
    useEffect(() => {
        setIsLoaded(true);
    }, []);

    // Intersection Observer setup
    useEffect(() => {
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries.length > 0) {
                    const entry = entries[0];
                    if (entry?.isIntersecting) {
                        // Animate progress bars when in view
                        setTimeout(() => {
                            setLevelProgress((data.level / (operator?.phases[data.evolvePhase]?.maxLevel ?? 1)) * 100);
                            setTrustProgress(trustPercentage);
                        }, 300); // Small delay for better visual effect
                        if (entry.target) {
                            observer.unobserve(entry.target);
                        }
                    }
                }
            },
            { threshold: 0.2 }, // Trigger when 20% of the card is visible
        );

        if (cardRef.current) {
            observer.observe(cardRef.current);
        }

        return () => {
            if (cardRef.current) {
                // eslint-disable-next-line react-hooks/exhaustive-deps
                observer.unobserve(cardRef.current);
            }
        };
    }, [data.level, operator, data.evolvePhase, trustPercentage]);

    // Handle card click to open dialog
    const handleCardClick = (e: React.MouseEvent) => {
        // Check if the click target is part of an accordion item
        const isAccordionClick = (e.target as HTMLElement).closest(".accordion-item") !== null;

        // Only open dialog if not clicking on accordion items
        if (!isAccordionClick) {
            setIsDialogOpen(true);
        }
    };

    // Full artwork for desktop
    const operatorImageDesktop = `https://raw.githubusercontent.com/fexli/ArknightsResource/main/charpack/${data.skin ? data.skin.replaceAll("@", "_").replaceAll("#", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_").replaceAll("#", "_")}.png`;

    // Full artwork for mobile
    const operatorImageMobile = `https://raw.githubusercontent.com/fexli/ArknightsResource/main/charpack/${data.skin ? data.skin.replaceAll("@", "_").replaceAll("#", "_") : (data.tmpl?.[data.currentTmpl ?? 0]?.skinId ?? "").replaceAll("@", "_").replaceAll("#", "_")}.png`;

    // Get current phase
    const getPhaseIcon = () => {
        return (
            <Image
                src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/elite/${data.evolvePhase}.png`}
                width={20}
                height={20}
                alt="Promotion"
                style={{
                    maxWidth: "100%",
                    height: "auto",
                    objectFit: "contain",
                }}
                className="transition-transform duration-300 hover:scale-110"
            />
        );
    };

    // Get rarity stars
    const getRarityStars = (rarity: string) => {
        const starCount =
            {
                TIER_6: 6,
                TIER_5: 5,
                TIER_4: 4,
                TIER_3: 3,
                TIER_2: 2,
                TIER_1: 1,
            }[rarity] ?? 0;

        return Array(starCount)
            .fill(0)
            .map((_, i) => <Image key={i} src="https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/star.png" width={16} height={16} alt="Star" className="h-4 w-4 transition-all duration-300 hover:rotate-12 hover:scale-125" style={{ transitionDelay: `${i * 50}ms` }} />);
    };

    // Get profession icon
    const getProfessionIcon = (profession: string) => {
        const professionMap = {
            MEDIC: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_medic.png",
            CASTER: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_caster.png",
            WARRIOR: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_guard.png",
            PIONEER: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_vanguard.png",
            SNIPER: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_sniper.png",
            SPECIAL: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_specialist.png",
            SUPPORT: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_supporter.png",
            TANK: "https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_defender.png",
        };

        return <Image src={professionMap[profession.toLowerCase() as keyof typeof professionMap] || `https://raw.githubusercontent.com/Aceship/Arknight-Images/main/classes/class_${profession.toLowerCase()}.png`} width={24} height={24} alt={profession} className="h-6 w-6 transition-transform duration-300 hover:rotate-12 hover:scale-110" />;
    };

    // Get potential level display
    const getPotentialDisplay = (rank: number) => {
        return (
            <div className="flex items-center gap-1">
                <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/potential/${rank + 1}.png`} width={24} height={24} alt={`Potential ${rank + 1}`} className="h-6 w-6 transition-transform duration-300 hover:scale-110" />
                <span className="text-xs text-gray-500">+{rank}</span>
            </div>
        );
    };

    // Get skill level display
    const getSkillLevelDisplay = (level: number) => {
        if (level === 0) return null;

        return (
            <div className="flex items-center gap-1">
                <Image src={`/m-${level}_0.webp`} width={24} height={24} alt={`Mastery ${level}`} className="h-6 w-6 transition-transform duration-300 hover:scale-110" />
                <span className="text-xs text-gray-500">M{level}</span>
            </div>
        );
    };

    // Get module level display
    const getModuleLevelDisplay = (level: number) => {
        if (level === 0) return null;

        return (
            <div className="flex items-center gap-0">
                <span className="text-xs text-gray-500">Lv.</span>
                <Image src={`https://raw.githubusercontent.com/Aceship/Arknight-Images/main/ui/rank/${level}.png`} width={16} height={16} alt={`Module Level ${level}`} className="h-4 w-4 transition-transform duration-300 hover:scale-110" />
            </div>
        );
    };

    // Get operator name
    const getOperatorName = () => {
        return operator?.name ?? "Unknown Operator";
    };

    // Get operator profession
    const getOperatorProfession = () => {
        return formatProfession(operator?.profession ?? "Unknown");
    };

    // Get operator rarity
    const getOperatorRarity = () => {
        return operator?.rarity ?? "TIER_1";
    };

    // Get operator stats
    const getOperatorStats = () => {
        if (!operator?.phases || operator.phases.length <= data.evolvePhase) {
            return null;
        }

        const phase = operator.phases[data.evolvePhase];
        if (!phase) return null;

        const maxLevel = phase.maxLevel;
        const level = Math.min(data.level, maxLevel);

        // Find the closest keyframe
        const keyframe = phase.attributesKeyFrames.find((kf) => kf.level <= level);
        if (!keyframe) return null;

        return keyframe.data;
    };

    const stats = getOperatorStats();

    return (
        <>
            <Card ref={cardRef} className={`w-full overflow-hidden border-2 border-muted/30 transition-all duration-300 hover:border-muted hover:shadow-lg ${isLoaded ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0"}`} onMouseEnter={() => setIsHovered(true)} onMouseLeave={() => setIsHovered(false)} onClick={handleCardClick} style={{ cursor: "pointer" }}>
                <div className="relative">
                    {/* Operator Image */}
                    <div className="relative h-64 w-full overflow-hidden">
                        {/* Desktop image */}
                        <Image loading="lazy" decoding="async" src={operatorImageDesktop} alt={getOperatorName()} className={`hidden h-full w-full object-cover transition-transform duration-300 md:block ${isHovered ? "scale-105" : "scale-100"}`} width={200} height={200} />
                        {/* Mobile image */}
                        <Image loading="lazy" decoding="async" src={operatorImageMobile} alt={getOperatorName()} className={`block h-full w-full object-cover transition-transform duration-300 md:hidden ${isHovered ? "scale-105" : "scale-100"}`} width={200} height={200} />
                        <div className={`absolute inset-0 bg-gradient-to-t from-black/70 to-transparent transition-opacity duration-300 ${isHovered ? "opacity-90" : "opacity-70"}`} />

                        {/* Operator Info Overlay */}
                        <div className="absolute bottom-0 left-0 right-0 p-4">
                            <h3 className={`mt-2 max-w-[75%] text-xl font-bold text-white transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>{getOperatorName()}</h3>
                            <div className={`flex items-center justify-between transition-all duration-300 ${isHovered ? "translate-y-0" : "translate-y-1"}`}>
                                <div className="flex items-center gap-2">
                                    <div className="flex flex-row gap-0">{getRarityStars(getOperatorRarity())}</div>
                                    <div className="flex flex-row items-center gap-1">
                                        {getProfessionIcon(getOperatorProfession())}
                                        <span>{getOperatorProfession()}</span>
                                    </div>
                                </div>
                                <div className="border-white/10 bg-black/10 text-white">{getPhaseIcon()}</div>
                            </div>
                        </div>
                    </div>

                    {/* Operator Stats */}
                    <CardContent className="p-4 pt-0">
                        <div className="grid grid-cols-2 gap-4">
                            {/* Level and Trust */}
                            <div>
                                <div className="flex items-center justify-between">
                                    <span className="text-sm font-medium">Level</span>
                                    <span className="text-sm font-bold">{data.level}</span>
                                </div>
                                <Progress value={levelProgress} className="h-1.5 transition-all duration-1000 ease-out" />
                            </div>

                            <div>
                                <div className="flex items-center justify-between">
                                    <span className="text-sm font-medium">Trust</span>
                                    <span className="text-sm font-bold">{operator?.trust ?? 0}/200</span>
                                </div>
                                <Progress value={trustProgress} className="h-1.5 transition-all duration-1000 ease-out" />
                            </div>
                        </div>

                        <Separator className="my-3" />

                        {/* Stats */}
                        {stats && (
                            <div className="grid grid-cols-2 gap-2 text-sm">
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-500 dark:text-gray-400">HP</span>
                                    <span className="font-medium">{stats.maxHp}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-500 dark:text-gray-400">ATK</span>
                                    <span className="font-medium">{stats.atk}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-500 dark:text-gray-400">DEF</span>
                                    <span className="font-medium">{stats.def}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-500 dark:text-gray-400">RES</span>
                                    <span className="font-medium">{stats.magicResistance}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-500 dark:text-gray-400">Cost</span>
                                    <span className="font-medium">{stats.cost}</span>
                                </div>
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-500 dark:text-gray-400">Block</span>
                                    <span className="font-medium">{stats.blockCnt}</span>
                                </div>
                            </div>
                        )}

                        <Separator className="my-3" />

                        {/* Accordion for Skills, Modules, etc. */}
                        <Accordion type="multiple" className="accordion-item w-full">
                            {/* Potential */}
                            <AccordionItem value="potential" className="accordion-item">
                                <AccordionTrigger className="text-sm font-medium">
                                    <span>Potential</span>
                                </AccordionTrigger>
                                <AccordionContent>
                                    <div className="flex items-center justify-between p-2">
                                        <span className="text-sm">Current Potential</span>
                                        {getPotentialDisplay(data.potentialRank)}
                                    </div>
                                    <div className="p-2 text-xs text-gray-500">
                                        <p>Potential increases operator stats and may unlock additional abilities.</p>
                                    </div>
                                </AccordionContent>
                            </AccordionItem>

                            {/* Skills */}
                            <AccordionItem value="skills" className="accordion-item">
                                <AccordionTrigger className="text-sm font-medium">
                                    <span>Skills</span>
                                </AccordionTrigger>
                                <AccordionContent>
                                    <ScrollArea className="h-[200px]">
                                        {data.skills && data.skills.length > 0 ? (
                                            <div className="space-y-3 p-2">
                                                {data.skills.map((skill, index) => (
                                                    <div key={`skill-${index}`} className={`rounded-md border p-3 ${data.defaultSkillIndex === index ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}>
                                                        <div className="flex items-start justify-between">
                                                            <div className="flex min-w-0 flex-1 items-start gap-2">
                                                                <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skill/skill_icon_${skill.static?.iconId ?? skill.static?.skillId}.png`} width={32} height={32} alt="Skill" className="h-8 w-8 flex-shrink-0" />
                                                                <div className="min-w-0">
                                                                    <div className="font-medium">{skill.static?.levels[0]?.name}</div>
                                                                    <span
                                                                        className="block text-xs leading-tight text-gray-500"
                                                                        dangerouslySetInnerHTML={{
                                                                            __html: descriptionToHtml(skill.static?.levels[skill.specializeLevel]?.description ?? "", skill.static?.levels[skill.specializeLevel]?.blackboard ?? []),
                                                                        }}
                                                                    ></span>
                                                                </div>
                                                            </div>
                                                            <div className="ml-2 flex-shrink-0">{getSkillLevelDisplay(skill.specializeLevel)}</div>
                                                        </div>
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <div className="p-2 text-sm text-gray-500">No skills found.</div>
                                        )}
                                    </ScrollArea>
                                </AccordionContent>
                            </AccordionItem>

                            {/* Modules */}
                            <AccordionItem value="modules" className="accordion-item">
                                <AccordionTrigger className="text-sm font-medium">
                                    <span>Modules</span>
                                </AccordionTrigger>
                                <AccordionContent>
                                    <ScrollArea className="">
                                        {operator?.modules && operator.modules.length > 0 ? (
                                            <div className="space-y-3 p-2">
                                                {operator.modules.map((module, index) => {
                                                    // Use a type guard to ensure module.id is defined
                                                    const moduleId = module.id;
                                                    if (!moduleId) return null;

                                                    const isEquipped = data.currentEquip === moduleId;

                                                    // Safely access the equip property with proper type checking
                                                    let moduleLevel = 0;
                                                    if (data.equip && typeof data.equip === "object" && moduleId in data.equip) {
                                                        const equipData = data.equip[moduleId];
                                                        if (equipData && typeof equipData === "object" && "level" in equipData) {
                                                            moduleLevel = equipData.level;
                                                        }
                                                    }

                                                    if (module.typeName1 === "ORIGINAL" || moduleLevel === 0) return null;

                                                    return (
                                                        <div key={`module-${index}`} className={`rounded-md border p-3 ${isEquipped ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20" : ""}`}>
                                                            <div className="flex items-center justify-between">
                                                                <div className="flex items-center gap-2">
                                                                    <Image src={`https://raw.githubusercontent.com/fexli/ArknightsResource/main/equip/${module.uniEquipIcon}.png`} width={32} height={32} alt="Module" className="h-8 w-8" />
                                                                    <div>
                                                                        <div className="font-medium">{module.uniEquipName}</div>
                                                                        <div className="text-xs text-gray-500">
                                                                            {module.typeName1} {module.typeName2 ? `(${module.typeName2})` : ""}
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                                {getModuleLevelDisplay(moduleLevel)}
                                                            </div>
                                                        </div>
                                                    );
                                                })}
                                            </div>
                                        ) : (
                                            <div className="p-2 text-sm text-gray-500">No modules found.</div>
                                        )}
                                    </ScrollArea>
                                </AccordionContent>
                            </AccordionItem>
                        </Accordion>
                    </CardContent>
                </div>
            </Card>

            {/* Dialog for Character Details */}
            <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                <DialogContent className="">
                    <CharacterDialogueCard data={data} />
                </DialogContent>
            </Dialog>
        </>
    );
}

export default CharacterCard;
