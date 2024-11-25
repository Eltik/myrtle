import type { Operator } from "~/types/impl/api/static/operator";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbSeparator } from "../ui/breadcrumb";
import Image from "next/image";
import { ScrollArea } from "../ui/scroll-area";
import { useState } from "react";
import { Button } from "../ui/button";
import InfoContent from "./components/info-content";

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

    return (<>
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
                    <span className="relative m-0 box-border inline-block max-w-full overflow-hidden border-0 p-0 grid-in-banner">
                        <Image
                            src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id ?? ""}_${operator.phases.length > 1 ? "2b" : "1b"}.png`}
                            alt={operator.name}
                            className="absolute bottom-0 left-0 right-0 top-0 m-auto box-border block h-0 max-h-full min-h-full w-0 min-w-full max-w-full object-cover p-0"
                            fill
                            sizes="100vw"
                            style={{
                                objectFit: "cover"
                            }} />
                    </span>
                </div>
            </div>
            <div className="container relative z-20 m-auto w-screen grid-in-top-fold">
                <header className="flex h-64 flex-col-reverse p-[24px_24px_0]">
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
                        <h1 className="text-6xl font-bold">{operator.name}</h1>
                    </div>
                </header>
                <div className="container">
                    <div className="m-[24px_0_0] flex p-[0_24px]">
                        <div className="mr-4 flex w-48 flex-col space-y-2">
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
                                    {activeTab === "skills" && <SkillsContent operator={operator} />}
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
    </>);
}

function SkillsContent({ operator }: { operator: Operator }) {
    return <div>Skills & Talents content for {operator.name}</div>;
}

function LevelUpContent({ operator }: { operator: Operator }) {
    return <div>Level-Up Cost content for {operator.name}</div>;
}

function SkinsContent({ operator }: { operator: Operator }) {
    return <div>Skins content for {operator.name}</div>;
}

function AudioContent({ operator }: { operator: Operator }) {
    return <div>Audio/SFX content for {operator.name}</div>;
}

export default OperatorsInfo;
