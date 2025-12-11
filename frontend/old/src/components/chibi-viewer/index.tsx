import { ChevronLeft, ChevronRight, Search } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { Button } from "~/components/ui/button";
import { Card, CardContent } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Skeleton } from "~/components/ui/skeleton";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { cn } from "~/lib/utils";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";
import type { Operator } from "~/types/impl/api/static/operator";
import type { AnimationType, FormattedChibis, SkinData } from "~/types/impl/frontend/impl/chibis";
import { ChibiRenderer } from "./impl/renderer";

export function ChibiViewer() {
    const [chibis, setChibis] = useState<FormattedChibis[]>([]);
    const [loading, setLoading] = useState(true);
    const [searchTerm, setSearchTerm] = useState("");
    const [selectedOperator, setSelectedOperator] = useState<FormattedChibis | null>(null);
    const [selectedSkin, setSelectedSkin] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<"info" | "canvas">("canvas");
    const [isListCollapsed, setIsListCollapsed] = useState(false);

    const formatData = useCallback((data: ChibisSimplified[], operatorsList: Operator[]): FormattedChibis[] => {
        // Deduplicate operators by their ID, keeping the first occurrence
        const uniqueOperators = new Set<string>();
        const uniqueData = data.filter((chibi) => {
            const id = chibi.operatorCode;
            if (uniqueOperators.has(id)) {
                return false;
            }
            uniqueOperators.add(id);
            return true;
        });

        // Map the simplified data to the frontend format
        return uniqueData.map((chibi) => {
            const operatorCode = chibi.operatorCode.includes("/") ? (chibi.operatorCode.split("/").pop() ?? chibi.operatorCode) : chibi.operatorCode;

            const formattedChibi: FormattedChibis = {
                name: chibi.name,
                operatorCode,
                path: chibi.path,
                skins: [],
                data: operatorsList.find((data) => data.id === operatorCode),
            };

            const skinsByName = new Map<string, SkinData>();

            for (const skin of chibi.skins) {
                // The backend now sends full skin names like "default"
                const skinName = skin.name;

                const existingSkin = skinsByName.get(skinName);

                const createAnimationData = (animationType: AnimationType | undefined) => ({
                    atlas: animationType?.atlas ?? "",
                    png: animationType?.png ?? "",
                    skel: animationType?.skel ?? "",
                    path: skin.path,
                });

                // Mapping animation types according to our new backend format:
                // BattleFront -> front
                // BattleBack -> back
                // Building -> dorm

                if (skin.animationTypes?.dorm) {
                    Object.assign(existingSkin ?? {}, { dorm: createAnimationData(skin.animationTypes.dorm) });
                }

                if (skin.animationTypes?.front) {
                    Object.assign(existingSkin ?? {}, { front: createAnimationData(skin.animationTypes.front) });
                }

                if (skin.animationTypes?.back) {
                    Object.assign(existingSkin ?? [], { back: createAnimationData(skin.animationTypes.back) });
                }

                if (existingSkin) {
                    skinsByName.set(skinName, existingSkin);
                } else {
                    skinsByName.set(skinName, {
                        name: skinName,
                        dorm: createAnimationData(skin.animationTypes?.dorm),
                        front: createAnimationData(skin.animationTypes?.front),
                        back: createAnimationData(skin.animationTypes?.back),
                    });
                }
            }

            const emptyAnimationData = {
                atlas: "",
                png: "",
                skel: "",
                path: "",
            };

            formattedChibi.skins = Array.from(skinsByName.values()).map((skin) => ({
                name: skin.name,
                dorm: skin.dorm ?? emptyAnimationData,
                front: skin.front ?? emptyAnimationData,
                back: skin.back ?? emptyAnimationData,
            }));

            return formattedChibi;
        });
    }, []);

    useEffect(() => {
        const fetchChibis = async () => {
            try {
                setLoading(true);

                const fetchOperators = async () => {
                    const data = (await (
                        await fetch("/api/static", {
                            method: "POST",
                            headers: {
                                "Content-Type": "application/json",
                            },
                            body: JSON.stringify({
                                type: "operators",
                            }),
                        })
                    ).json()) as {
                        data: Operator[];
                    };

                    return data.data;
                };

                const operatorsList = await fetchOperators();

                const response = await fetch("/api/chibis", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        format: "simplified",
                    }),
                });

                const data = formatData((await response.json()) as ChibisSimplified[], operatorsList);
                setChibis(data);

                // Select the first operator by default
                if (data.length > 0) {
                    const firstOperator = data[0];
                    if (firstOperator) {
                        setSelectedOperator(firstOperator);
                        if (firstOperator.skins && firstOperator.skins.length > 0 && firstOperator.skins[0]?.dorm.path) {
                            setSelectedSkin(firstOperator.skins[0].dorm.path);
                        }
                    }
                }
            } catch (error) {
                console.error("Error fetching chibis:", error);
            } finally {
                setLoading(false);
            }
        };

        void fetchChibis();
    }, [formatData]);

    const filteredChibis = chibis.filter((chibi) => chibi.name.toLowerCase().includes(searchTerm.toLowerCase()) || chibi.operatorCode.toLowerCase().includes(searchTerm.toLowerCase()));

    const handleOperatorSelect = (operator: FormattedChibis) => {
        setSelectedOperator(operator);
        if (operator.skins && operator.skins.length > 0 && operator.skins[0]?.front.path) {
            setSelectedSkin(operator.skins[0].front.path);
        } else {
            setSelectedSkin(null);
        }
    };

    return (
        <div className="container mx-auto py-8">
            <div className="flex flex-col md:flex-row">
                {/* Operator List */}
                <div className={cn("transition-all duration-300 ease-in-out", isListCollapsed ? "w-[60px]" : "w-full md:w-1/3")}>
                    <div className="sticky top-4 flex">
                        <div className={cn("flex flex-col gap-4 transition-all duration-300 ease-in-out", isListCollapsed ? "w-0 overflow-hidden opacity-0" : "w-full opacity-100")}>
                            <div className="relative">
                                <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
                                <Input className="pl-9" onChange={(e) => setSearchTerm(e.target.value)} placeholder="Search operators..." value={searchTerm} />
                            </div>
                            <div className="h-[70vh] overflow-y-auto rounded-lg border bg-card">
                                {loading ? (
                                    Array.from({ length: 10 }).map((_, i) => (
                                        <div className="border-b p-3" key={i}>
                                            <Skeleton className="h-12 w-full" />
                                        </div>
                                    ))
                                ) : filteredChibis.length > 0 ? (
                                    <>
                                        <div className="sticky top-0 border-b bg-card/95 p-2 backdrop-blur supports-[backdrop-filter]:bg-card/75">
                                            <p className="text-muted-foreground text-sm">
                                                {filteredChibis.length} operator{filteredChibis.length !== 1 ? "s" : ""} found
                                            </p>
                                        </div>
                                        <div className="divide-y">
                                            {filteredChibis.map((chibi) => (
                                                <Button
                                                    className={cn("h-auto w-full justify-start p-3 transition-colors", selectedOperator?.operatorCode === chibi.operatorCode ? "bg-primary/10 hover:bg-primary/20" : "hover:bg-accent")}
                                                    key={chibi.operatorCode}
                                                    onClick={() => handleOperatorSelect(chibi)}
                                                    variant={selectedOperator?.operatorCode === chibi.operatorCode ? "default" : "ghost"}
                                                >
                                                    <div className="text-left">
                                                        <div className="font-medium text-primary">{chibi.data?.name ?? chibi.name}</div>
                                                        <div className="line-clamp-1 truncate text-muted-foreground text-sm">{chibi.operatorCode}</div>
                                                    </div>
                                                </Button>
                                            ))}
                                        </div>
                                    </>
                                ) : (
                                    <div className="flex h-full items-center justify-center p-8 text-center text-muted-foreground">No operators found</div>
                                )}
                            </div>
                        </div>
                        <Button className={`h-auto shrink-0 ${isListCollapsed ? "rounded-r-none" : "rounded-l-none"}`} onClick={() => setIsListCollapsed(!isListCollapsed)} size="icon" variant="ghost">
                            {isListCollapsed ? <ChevronRight className="h-4 w-4" /> : <ChevronLeft className="h-4 w-4" />}
                        </Button>
                    </div>
                </div>

                {/* Chibi Viewer */}
                <div className="w-full md:w-2/3">
                    <Card>
                        <CardContent className="p-6">
                            {selectedOperator ? (
                                <div className="space-y-6">
                                    <div className="flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
                                        <div>
                                            <h2 className="font-bold text-2xl">{selectedOperator.data?.name ?? selectedOperator.name}</h2>
                                            <p className="text-muted-foreground">{selectedOperator.operatorCode}</p>
                                        </div>

                                        <div className="flex flex-wrap gap-2">
                                            {/* Skin selector */}
                                            {selectedOperator.skins && selectedOperator.skins.length > 0 && (
                                                <div className="flex flex-wrap gap-2">
                                                    {selectedOperator.skins.map((skin, index) => (
                                                        <Button key={index} onClick={() => setSelectedSkin(skin.front.path)} size="sm" variant={selectedSkin === skin.dorm.path ? "default" : "outline"}>
                                                            Skin {index + 1}
                                                        </Button>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
                                    </div>

                                    <Tabs onValueChange={(value) => setActiveTab(value as "info" | "canvas")} value={activeTab}>
                                        <TabsList className="mb-4">
                                            <TabsTrigger value="canvas">Canvas Renderer</TabsTrigger>
                                            <TabsTrigger value="info">Spine Info</TabsTrigger>
                                        </TabsList>

                                        <TabsContent className="mt-0" value="canvas">
                                            <ChibiRenderer selectedOperator={selectedOperator} selectedSkin={selectedSkin} />
                                        </TabsContent>

                                        <TabsContent className="mt-0" value="info">
                                            {selectedSkin && selectedOperator.skins?.find((skin) => skin.dorm.path === selectedSkin) && (
                                                <div className="space-y-2 text-sm">
                                                    <div className="font-medium">Spine Data Files:</div>
                                                    <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                                                        <div className="rounded-lg border p-3">
                                                            <div className="font-medium">Atlas:</div>
                                                            <div className="truncate text-muted-foreground text-xs">{selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin)?.dorm.atlas ?? "N/A"}</div>
                                                        </div>
                                                        <div className="rounded-lg border p-3">
                                                            <div className="font-medium">Skeleton:</div>
                                                            <div className="truncate text-muted-foreground text-xs">{selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin)?.dorm.skel ?? "N/A"}</div>
                                                        </div>
                                                        <div className="rounded-lg border p-3">
                                                            <div className="font-medium">Image:</div>
                                                            <div className="truncate text-muted-foreground text-xs">{selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin)?.dorm.png ?? "N/A"}</div>
                                                        </div>
                                                    </div>
                                                </div>
                                            )}
                                        </TabsContent>
                                    </Tabs>
                                </div>
                            ) : (
                                <div className="p-8 text-center text-muted-foreground">{loading ? "Loading..." : "Select an operator to view their chibi"}</div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
}
