import { useState, useEffect, useCallback } from "react";
import { Card, CardContent } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Button } from "~/components/ui/button";
import { Skeleton } from "~/components/ui/skeleton";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";
import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import { type Operator } from "~/types/impl/api/static/operator";
import { ChibiRenderer } from "./impl/renderer";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";

export function ChibiViewer() {
    const [operators, setOperators] = useState<Operator[]>([]);

    const [chibis, setChibis] = useState<FormattedChibis[]>([]);
    const [loading, setLoading] = useState(true);
    const [searchTerm, setSearchTerm] = useState("");
    const [selectedOperator, setSelectedOperator] = useState<FormattedChibis | null>(null);
    const [selectedSkin, setSelectedSkin] = useState<string | null>(null);
    const [activeTab, setActiveTab] = useState<"info" | "canvas">("canvas");

    const formatData = useCallback((data: ChibisSimplified[]): FormattedChibis[] => {
        // Deduplicate operators by their ID, keeping the first occurrence
        const uniqueData = data.reduce((acc, curr) => {
            const operatorCode = curr.operatorCode.includes("/") ? (curr.operatorCode.split("/").pop() ?? curr.operatorCode) : curr.operatorCode;

            if (
                !acc.some((item) => {
                    const itemCode = item.operatorCode.includes("/") ? (item.operatorCode.split("/").pop() ?? item.operatorCode) : item.operatorCode;
                    return itemCode === operatorCode;
                })
            ) {
                acc.push(curr);
            }
            return acc;
        }, [] as ChibisSimplified[]);

        return uniqueData.map((chibi) => {
            // Get the last part of the operator code if it contains a slash
            const operatorCode = chibi.operatorCode.includes("/") ? (chibi.operatorCode.split("/").pop() ?? chibi.operatorCode) : chibi.operatorCode;

            const formattedChibi: FormattedChibis = {
                name: chibi.name,
                operatorCode: operatorCode,
                path: chibi.path,
                skins: [],
                data: operators.find((data) => data.id === operatorCode),
            };

            // Group skins by name to combine their animation types
            const skinsByName = new Map<
                string,
                {
                    name: string;
                    dorm?: { atlas: string; png: string; skel: string; path: string };
                    front?: { atlas: string; png: string; skel: string; path: string };
                    back?: { atlas: string; png: string; skel: string; path: string };
                }
            >();

            for (const skin of chibi.skins) {
                const skinName = skin.name.startsWith("build_") ? (skin.name.split("build_")[1]?.split("/")[0] ?? chibi.name) : skin.name;

                const skinData = skinsByName.get(skinName) ?? {
                    name: skinName,
                };

                if (skin.animationTypes?.dorm) {
                    skinData.dorm = {
                        atlas: skin.animationTypes.dorm.atlas ?? "",
                        png: skin.animationTypes.dorm.png ?? "",
                        skel: skin.animationTypes.dorm.skel ?? "",
                        path: skin.path,
                    };
                }

                if (skin.animationTypes?.front) {
                    skinData.front = {
                        atlas: skin.animationTypes.front.atlas ?? "",
                        png: skin.animationTypes.front.png ?? "",
                        skel: skin.animationTypes.front.skel ?? "",
                        path: skin.path,
                    };
                }

                if (skin.animationTypes?.back) {
                    skinData.back = {
                        atlas: skin.animationTypes.back.atlas ?? "",
                        png: skin.animationTypes.back.png ?? "",
                        skel: skin.animationTypes.back.skel ?? "",
                        path: skin.path,
                    };
                }

                skinsByName.set(skinName, skinData);
            }

            // Convert grouped skins to final format
            formattedChibi.skins = Array.from(skinsByName.values()).map((skin) => ({
                name: skin.name,
                dorm: skin.dorm ?? {
                    atlas: "",
                    png: "",
                    skel: "",
                    path: "",
                },
                front: skin.front ?? {
                    atlas: "",
                    png: "",
                    skel: "",
                    path: "",
                },
                back: skin.back ?? {
                    atlas: "",
                    png: "",
                    skel: "",
                    path: "",
                },
            }));

            return formattedChibi;
        });
        // eslint-disable-next-line react-hooks/exhaustive-deps
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

                    console.log(data);

                    setOperators(data.data);
                };

                await fetchOperators();

                const response = await fetch("/api/chibis", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        format: "simplified",
                    }),
                });

                const data = formatData((await response.json()) as ChibisSimplified[]);
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
        if (operator.skins && operator.skins.length > 0 && operator.skins[0]?.dorm.path) {
            setSelectedSkin(operator.skins[0].dorm.path);
        } else {
            setSelectedSkin(null);
        }
    };

    // GitHub raw content URL base for the repository
    const repoBaseUrl = "https://raw.githubusercontent.com/fexli/ArknightsResource/main/";

    return (
        <div className="container mx-auto py-8">
            <div className="flex flex-col gap-8 md:flex-row">
                {/* Operator List */}
                <div className="w-full space-y-4 md:w-1/3">
                    <div className="sticky top-4">
                        <Input placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="mb-4" />

                        <div className="h-[70vh] overflow-y-auto rounded-lg border bg-card">
                            {loading ? (
                                Array.from({ length: 10 }).map((_, i) => (
                                    <div key={i} className="border-b p-3">
                                        <Skeleton className="h-12 w-full" />
                                    </div>
                                ))
                            ) : filteredChibis.length > 0 ? (
                                <div className="divide-y">
                                    {filteredChibis.map((chibi) => (
                                        <Button key={chibi.operatorCode} variant={selectedOperator?.operatorCode === chibi.operatorCode ? "default" : "ghost"} className="h-auto w-full justify-start p-3" onClick={() => handleOperatorSelect(chibi)}>
                                            <div className="text-left">
                                                <div className="font-medium">{chibi.data?.name ?? chibi.name}</div>
                                                <div className="line-clamp-1 truncate text-sm text-muted-foreground">{chibi.operatorCode}</div>
                                            </div>
                                        </Button>
                                    ))}
                                </div>
                            ) : (
                                <div className="p-8 text-center text-muted-foreground">No operators found</div>
                            )}
                        </div>
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
                                            <h2 className="text-2xl font-bold">{selectedOperator.data?.name ?? selectedOperator.name}</h2>
                                            <p className="text-muted-foreground">{selectedOperator.operatorCode}</p>
                                        </div>

                                        <div className="flex flex-wrap gap-2">
                                            {/* Skin selector */}
                                            {selectedOperator.skins && selectedOperator.skins.length > 0 && (
                                                <div className="flex flex-wrap gap-2">
                                                    {selectedOperator.skins.map((skin, index) => (
                                                        <Button key={index} variant={selectedSkin === skin.dorm.path ? "default" : "outline"} onClick={() => setSelectedSkin(skin.dorm.path)} size="sm">
                                                            Skin {index + 1}
                                                        </Button>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
                                    </div>

                                    <Tabs value={activeTab} onValueChange={(value) => setActiveTab(value as "info" | "canvas")}>
                                        <TabsList className="mb-4">
                                            <TabsTrigger value="canvas">Canvas Renderer</TabsTrigger>
                                            <TabsTrigger value="info">Spine Info</TabsTrigger>
                                        </TabsList>

                                        <TabsContent value="canvas" className="mt-0">
                                            <ChibiRenderer selectedOperator={selectedOperator} selectedSkin={selectedSkin} repoBaseUrl={repoBaseUrl} />
                                        </TabsContent>

                                        <TabsContent value="info" className="mt-0">
                                            {selectedSkin && selectedOperator.skins?.find((skin) => skin.dorm.path === selectedSkin) && (
                                                <div className="space-y-2 text-sm">
                                                    <div className="font-medium">Spine Data Files:</div>
                                                    <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                                                        <div className="rounded-lg border p-3">
                                                            <div className="font-medium">Atlas:</div>
                                                            <div className="truncate text-xs text-muted-foreground">{selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin)?.dorm.atlas ?? "N/A"}</div>
                                                        </div>
                                                        <div className="rounded-lg border p-3">
                                                            <div className="font-medium">Skeleton:</div>
                                                            <div className="truncate text-xs text-muted-foreground">{selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin)?.dorm.skel ?? "N/A"}</div>
                                                        </div>
                                                        <div className="rounded-lg border p-3">
                                                            <div className="font-medium">Image:</div>
                                                            <div className="truncate text-xs text-muted-foreground">{selectedOperator.skins.find((skin) => skin.dorm.path === selectedSkin)?.dorm.png ?? "N/A"}</div>
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
