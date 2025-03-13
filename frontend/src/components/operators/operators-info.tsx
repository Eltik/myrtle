import type { Operator } from "~/types/impl/api/static/operator";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbSeparator } from "../ui/breadcrumb";
import Image from "next/image";
import { ScrollArea, ScrollBar } from "../ui/scroll-area";
import { useState } from "react";
import { Button } from "../ui/button";
import InfoContent from "./components/info-content";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";
import SkillsTalentsContent from "./components/skills-talents-content";
import LevelUpContent from "./components/level-cost-content";
import SkinsContent from "./components/skins-content";
import AudioContent from "./components/audio-content";

type TabType = "info" | "skills" | "levelup" | "skins" | "audio";

function OperatorsInfo({ operator }: { operator: Operator }) {
    const [activeTab, setActiveTab] = useState<TabType>("info");

    const tabs: { type: TabType; label: string }[] = [
        { type: "info", label: "Information" },
        { type: "skills", label: "Skills & Talents" },
        { type: "levelup", label: "Level-Up Cost" },
        { type: "skins", label: "Skins" },
        { type: "audio", label: "Audio/SFX" },
    ];

    return (
        <>
            <div className="grid-areas-operatorsInfo grid h-full w-full rounded-md">
                <div className="flex justify-center grid-in-top-fold">
                    <div className="relative grid h-[800px] w-full grid-areas-operatorBanner">
                        <div
                            className="absolute z-10 h-full w-full"
                            style={{
                                background: "linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, hsl(var(--background)) 100%)",
                            }}
                        />
                        <div
                            className="absolute z-10 h-full w-full"
                            style={{
                                background: "linear-gradient(180deg, rgba(0, 0, 0, 0) 0%, hsl(var(--background)) 100%)",
                            }}
                        />
                        <div className="absolute z-10 h-full w-full" />
                        <span className="relative m-0 box-border inline-block max-w-full overflow-hidden border-0 p-0 grid-in-banner">
                            <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id ?? ""}_${operator.phases.length > 2 ? "2b" : "1b"}.png`} alt={operator.name} layout="fill" objectFit="cover" className="absolute bottom-0 left-0 right-0 top-0 m-auto box-border block h-0 max-h-full min-h-full w-0 min-w-full max-w-full object-cover p-0" />
                        </span>
                    </div>
                </div>
                <div className="container relative z-20 m-auto w-screen grid-in-top-fold">
                    <header className="mb-2 flex h-64 flex-col-reverse p-[24px_24px_0]">
                        <div className="flex flex-col">
                            <Breadcrumb>
                                <BreadcrumbList>
                                    <BreadcrumbItem>
                                        <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
                                    </BreadcrumbItem>
                                    <BreadcrumbSeparator />
                                    <BreadcrumbItem>
                                        <BreadcrumbLink href={`/operators?id=${operator.id}`}>{operator.name}</BreadcrumbLink>
                                    </BreadcrumbItem>
                                </BreadcrumbList>
                            </Breadcrumb>
                            <h1 className="text-4xl font-bold md:text-6xl">{operator.name}</h1>
                        </div>
                    </header>
                    <div className="px-1 md:container md:px-0">
                        <div className="flex w-full flex-col flex-wrap md:flex-row md:flex-nowrap md:p-[0_24px]">
                            <div className="w-full flex-1 md:hidden">
                                <ScrollArea className="w-full whitespace-nowrap rounded-md">
                                    <Tabs defaultValue="info" className="w-full" onValueChange={(value) => setActiveTab(value as TabType)}>
                                        <TabsList className="inline-flex h-10 w-full items-center justify-center rounded-md bg-muted p-1 text-muted-foreground">
                                            {tabs.map((tab) => (
                                                <TabsTrigger key={tab.type} value={tab.type} className="inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-sm">
                                                    {tab.label}
                                                </TabsTrigger>
                                            ))}
                                        </TabsList>
                                    </Tabs>
                                    <ScrollBar orientation="horizontal" className="h-0" />
                                </ScrollArea>
                            </div>
                            <div className="mr-4 mt-2 hidden w-48 flex-col space-y-1 md:flex">
                                {tabs.map((tab) => (
                                    <Button key={tab.type} variant={activeTab === tab.type ? "default" : "outline"} className="justify-start" onClick={() => setActiveTab(tab.type)}>
                                        {tab.label}
                                    </Button>
                                ))}
                            </div>
                            <div className="flex-1 rounded-md border bg-card/50 backdrop-blur-sm">
                                <ScrollArea className="w-full rounded-md">
                                    <div>
                                        {activeTab === "info" && <InfoContent operator={operator} />}
                                        {activeTab === "skills" && <SkillsTalentsContent operator={operator} />}
                                        {activeTab === "levelup" && <LevelUpContent operator={operator} />}
                                        {activeTab === "skins" && <SkinsContent operator={operator} />}
                                        {activeTab === "audio" && <AudioContent operator={operator} />}
                                    </div>
                                </ScrollArea>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </>
    );
}

export default OperatorsInfo;
