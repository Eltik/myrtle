import type { OperatorSelectorProps } from "~/types/impl/frontend/impl/dps-calculator";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { useMemo, useState } from "react";
import { type Operator, OperatorProfession } from "~/types/impl/api/static/operator";
import { ChevronsUpDown, Search, X } from "lucide-react";
import { Input } from "../ui/input";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuTrigger } from "../ui/dropdown-menu";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "../ui/hover-card";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "../ui/accordion";
import { Checkbox } from "../ui/checkbox";
import { Label } from "../ui/label";
import { Badge } from "../ui/badge";
import { capitalize, formatProfession, formatSubProfession } from "~/helper";
import type { DPSOperator } from "~/types/impl/api/impl/dps-calculator";

type OperatorOrDPSOperator = Operator | DPSOperator;

// Define SelectedOperator locally if not imported, just for prop type check
interface SelectedOperator extends Operator {
    instanceId: string;
    displayName: string;
}
type OperatorSelectorInput = Operator | DPSOperator | SelectedOperator;

function OperatorSelector({ operators, selectedOperators, isOpen, onClose, onSelect }: OperatorSelectorProps) {
    const [search, setSearch] = useState("");
    const [searchSubclass, setSearchSubclass] = useState("");

    const [selectedClasses, setSelectedClasses] = useState<OperatorProfession[]>(
        Object.keys(OperatorProfession).map((profession) => {
            const value = OperatorProfession[profession as keyof typeof OperatorProfession];
            return value;
        }),
    );
    const [selectedSubclasses, setSelectedSubclasses] = useState<string[]>([]);

    // Helper function to safely access operator properties
    const getOperatorProperty = (operator: OperatorSelectorInput, property: string): string | null => {
        // Check if it's a SelectedOperator (which includes Operator properties)
        if ("instanceId" in operator && "displayName" in operator) {
            return (operator[property as keyof SelectedOperator] as string) || null;
        }
        // If the operator is a DPSOperator (has operatorData.data structure)
        if ("operatorData" in operator && operator.operatorData?.data) {
            return (operator.operatorData.data[property as keyof Operator] as string) || null;
        }
        // If the operator is a regular Operator
        return ((operator as Operator)[property as keyof Operator] as string) || null;
    };

    const groupedOperators = useMemo(() => {
        return operators.reduce(
            (acc, operator) => {
                const profession = getOperatorProperty(operator, "profession") as OperatorProfession;
                const subProfessionId = getOperatorProperty(operator, "subProfessionId");

                if (!profession || !subProfessionId) return acc;

                if (!acc[profession]) {
                    acc[profession] = {};
                }
                if (!acc[profession][subProfessionId]) {
                    acc[profession][subProfessionId] = [];
                }

                acc[profession][subProfessionId]?.push(operator);
                return acc;
            },
            {} as Record<OperatorProfession, Record<string, OperatorOrDPSOperator[]>>,
        );
    }, [operators]);

    const filteredOperators = useMemo(() => {
        return Object.entries(groupedOperators).reduce(
            (acc, [className, subclasses]) => {
                if (selectedClasses.includes(className as OperatorProfession)) {
                    acc[className] = Object.entries(subclasses).reduce(
                        (subAcc, [subclassName, ops]) => {
                            const formattedSubclassName = formatSubProfession(subclassName).toLowerCase();
                            const passesSubclassFilter = selectedSubclasses.length === 0 || selectedSubclasses.includes(subclassName);
                            const passesSearchFilter = searchSubclass === "" || formattedSubclassName.includes(searchSubclass.toLowerCase());

                            if (passesSubclassFilter && passesSearchFilter) {
                                subAcc[subclassName] = ops.filter((op) => {
                                    const name = getOperatorProperty(op, "name");
                                    return name?.toLowerCase().includes(search.toLowerCase());
                                });
                            }
                            return subAcc;
                        },
                        {} as Record<string, OperatorOrDPSOperator[]>,
                    );
                }
                return acc;
            },
            {} as Record<string, Record<string, OperatorOrDPSOperator[]>>,
        );
    }, [groupedOperators, selectedClasses, selectedSubclasses, search, searchSubclass]);

    const handleToggleOperator = (operator: OperatorOrDPSOperator) => {
        const operatorId = getOperatorProperty(operator, "id");
        const isSelected = selectedOperators.some((op) => getOperatorProperty(op, "id") === operatorId);

        let newSelection: Operator[]; // Ensure we always call onSelect with Operator[]

        // Helper to safely extract Operator from various possible input types
        const getBaseOperator = (opInput: OperatorSelectorInput | undefined): Operator | null => {
            if (!opInput) return null; // Handle undefined input

            // If it's SelectedOperator, extract the base Operator part
            if ("instanceId" in opInput && "displayName" in opInput) {
                // eslint-disable-next-line @typescript-eslint/no-unused-vars
                const { instanceId, displayName, ...baseOp } = opInput; // Ensure no 'as SelectedOperator' here
                return baseOp;
            }
            if ("operatorData" in opInput && opInput.operatorData?.data) {
                return opInput.operatorData.data;
            }
            console.warn("Could not extract base Operator from:", opInput); // Add warning for unexpected types
            return null; // Return null if extraction fails
        };

        if (isSelected) {
            newSelection = selectedOperators
                .filter((op) => getOperatorProperty(op, "id") !== operatorId)
                .map((op) => getBaseOperator(op)) // Use helper
                .filter((op): op is Operator => op !== null); // Filter out nulls and assert type
        } else {
            const operatorToAdd = getBaseOperator(operator); // Use helper

            if (!operatorToAdd) {
                console.error("Failed to add operator, could not extract base data from:", operator);
                return; // Don't proceed if we couldn't get the base operator
            }

            const currentOperators = selectedOperators
                .map((op) => getBaseOperator(op)) // Use helper
                .filter((op): op is Operator => op !== null); // Filter out nulls and assert type

            newSelection = [...currentOperators, operatorToAdd];
        }
        onSelect(newSelection); // Consistently call onSelect with Operator[]
    };

    const handleClassToggle = (className: OperatorProfession) => {
        setSelectedClasses((prev) => (prev.includes(className) ? prev.filter((c) => c !== className) : [...prev, className]));
    };

    const handleSubclassToggle = (subclass: string) => {
        setSelectedSubclasses((prev) => (prev.includes(subclass) ? prev.filter((s) => s !== subclass) : [...prev, subclass]));
    };

    const allSubclasses = useMemo(() => {
        return Array.from(new Set(Object.values(groupedOperators).flatMap((subclasses) => Object.keys(subclasses))));
    }, [groupedOperators]);

    const filteredAllSubclasses = useMemo(() => {
        if (!searchSubclass) {
            return allSubclasses;
        }
        return allSubclasses.filter((subclass) => formatSubProfession(subclass).toLowerCase().includes(searchSubclass.toLowerCase()));
    }, [allSubclasses, searchSubclass]);

    const hasOperators = useMemo(() => {
        return Object.values(filteredOperators).some((subclasses) => Object.values(subclasses).some((ops) => ops.length > 0));
    }, [filteredOperators]);

    return (
        <>
            <Dialog open={isOpen} onOpenChange={onClose}>
                <DialogContent className="sm:max-w-[800px]">
                    <DialogHeader>
                        <DialogTitle>Select Operators</DialogTitle>
                    </DialogHeader>
                    <div className="mb-4 flex flex-col items-center gap-2 space-x-2 sm:flex-row sm:gap-0">
                        <div className="flex items-center space-x-2">
                            <Search className="h-4 w-4 text-muted-foreground" />
                            <Input placeholder="Search operators..." value={search} onChange={(e) => setSearch(e.target.value)} className="flex-1" />
                        </div>
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button variant="outline" className="ml-auto">
                                    <span className="mr-2 max-w-24 truncate">{selectedClasses.length === 0 ? <span className="font-normal">Filter Classes</span> : selectedClasses.map((v) => formatProfession(v)).join(", ")}</span>
                                    <ChevronsUpDown className="ml-2 h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                                {Object.keys(OperatorProfession).map((className) => {
                                    const value = OperatorProfession[className as keyof typeof OperatorProfession];
                                    return (
                                        <DropdownMenuCheckboxItem key={className} checked={selectedClasses.includes(value)} onCheckedChange={() => handleClassToggle(value)}>
                                            {capitalize(className)}
                                        </DropdownMenuCheckboxItem>
                                    );
                                })}
                            </DropdownMenuContent>
                        </DropdownMenu>
                        <HoverCard>
                            <HoverCardTrigger asChild>
                                <Button variant="outline">
                                    <span className="mr-2 max-w-32 truncate">{selectedSubclasses.length === 0 ? <span className="font-normal">Filter Subclasses</span> : selectedSubclasses.map((v) => formatSubProfession(v)).join(", ")}</span>
                                    <ChevronsUpDown className="ml-2 h-4 w-4" />
                                </Button>
                            </HoverCardTrigger>
                            <HoverCardContent align="end" className="w-[300px] p-2">
                                <div className="mb-2 flex items-center space-x-2">
                                    <Search className="h-4 w-4 text-muted-foreground" />
                                    <Input placeholder="Search subclasses..." value={searchSubclass} onChange={(e) => setSearchSubclass(e.target.value)} className="flex-1" />
                                </div>
                                <ScrollArea className="h-48">
                                    {filteredAllSubclasses.map((subclass) => (
                                        <div key={subclass} className="flex items-center space-x-2 p-1">
                                            <Checkbox id={`subclass-${subclass}`} checked={selectedSubclasses.includes(subclass)} onCheckedChange={() => handleSubclassToggle(subclass)} />
                                            <Label htmlFor={`subclass-${subclass}`} className="text-sm font-normal">
                                                {formatSubProfession(subclass)}
                                            </Label>
                                        </div>
                                    ))}
                                    {filteredAllSubclasses.length === 0 && <p className="p-2 text-center text-sm text-muted-foreground">No subclasses found.</p>}
                                </ScrollArea>
                            </HoverCardContent>
                        </HoverCard>
                    </div>
                    <ScrollArea className="h-[300px] pr-4">
                        {!hasOperators && (
                            <div className="flex h-full items-center justify-center">
                                <p className="text-muted-foreground">No operators found</p>
                            </div>
                        )}
                        {hasOperators && (
                            <Accordion type="multiple" className="w-full">
                                {Object.entries(filteredOperators).map(([className, subclasses]) => {
                                    const hasOperators = Object.values(subclasses).some((ops) => ops.length > 0);
                                    return hasOperators ? (
                                        <AccordionItem value={className} key={className}>
                                            <AccordionTrigger>{formatProfession(className)}</AccordionTrigger>
                                            <AccordionContent className="rounded-md border p-4">
                                                {Object.entries(subclasses).map(
                                                    ([subclassName, ops]) =>
                                                        ops.length > 0 && (
                                                            <div key={subclassName} className="mb-4">
                                                                <h4 className="mb-2 font-semibold">{formatSubProfession(subclassName)}</h4>
                                                                <div className="grid grid-cols-2 gap-2">
                                                                    {ops.map((operator) => {
                                                                        const id = getOperatorProperty(operator, "id");
                                                                        const name = getOperatorProperty(operator, "name");
                                                                        return (
                                                                            <div key={id ?? name} className="flex items-center space-x-2">
                                                                                <Checkbox id={id ?? ""} checked={selectedOperators.some((op) => getOperatorProperty(op, "id") === id)} onCheckedChange={() => handleToggleOperator(operator)} />
                                                                                <Label htmlFor={id ?? ""} className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                                                                    {name}
                                                                                </Label>
                                                                            </div>
                                                                        );
                                                                    })}
                                                                </div>
                                                            </div>
                                                        ),
                                                )}
                                            </AccordionContent>
                                        </AccordionItem>
                                    ) : null;
                                })}
                            </Accordion>
                        )}
                    </ScrollArea>
                    <div className="mt-4 flex flex-wrap gap-2">
                        {selectedOperators.map((operator) => {
                            const id = getOperatorProperty(operator, "id");
                            const name = getOperatorProperty(operator, "name");
                            const instanceId = getOperatorProperty(operator, "instanceId");
                            return (
                                <Badge key={instanceId ?? id} variant="secondary">
                                    {name}
                                    <Button variant="ghost" size="sm" className="ml-2 h-4 w-4 p-0" onClick={() => handleToggleOperator(operator)}>
                                        <X className="h-3 w-3" />
                                    </Button>
                                </Badge>
                            );
                        })}
                    </div>
                    <div className="mt-4 flex justify-end">
                        <Button onClick={onClose}>Done</Button>
                    </div>
                </DialogContent>
            </Dialog>
        </>
    );
}

export default OperatorSelector;
