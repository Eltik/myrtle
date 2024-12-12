import type { OperatorSelectorProps } from "~/types/impl/frontend/impl/dps-calculator";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { useMemo, useState } from "react";
import { type Operator, OperatorProfession } from "~/types/impl/api/static/operator";
import { ChevronsUpDown, Search, X } from "lucide-react";
import { Input } from "../ui/input";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuTrigger } from "../ui/dropdown-menu";
import { Button } from "../ui/button";
import { ScrollArea } from "../ui/scroll-area";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "../ui/accordion";
import { Checkbox } from "../ui/checkbox";
import { Label } from "../ui/label";
import { Badge } from "../ui/badge";
import { capitalize, formatProfession, formatSubProfession } from "~/helper";

function OperatorSelector({ operators, selectedOperators, isOpen, onClose, onSelect }: OperatorSelectorProps) {
    const [search, setSearch] = useState("");

    const [selectedClasses, setSelectedClasses] = useState<OperatorProfession[]>(
        Object.keys(OperatorProfession).map((profession) => {
            const value = OperatorProfession[profession as keyof typeof OperatorProfession];
            return value;
        }),
    );
    const [selectedSubclasses, setSelectedSubclasses] = useState<string[]>([]);

    const groupedOperators = useMemo(() => {
        return operators.reduce(
            (acc, operator) => {
                if (!acc[operator.profession]) {
                    acc[operator.profession] = {};
                }
                if (!acc[operator.profession][operator.subProfessionId]) {
                    acc[operator.profession][operator.subProfessionId] = [];
                }

                acc[operator.profession][operator.subProfessionId]?.push(operator);
                return acc;
            },
            {} as Record<OperatorProfession, Record<string, Operator[]>>,
        );
    }, [operators]);

    const filteredOperators = useMemo(() => {
        return Object.entries(groupedOperators).reduce(
            (acc, [className, subclasses]) => {
                if (selectedClasses.includes(className as OperatorProfession)) {
                    acc[className] = Object.entries(subclasses).reduce(
                        (subAcc, [subclassName, ops]) => {
                            if (selectedSubclasses.length === 0 || selectedSubclasses.includes(subclassName)) {
                                subAcc[subclassName] = ops.filter((op) => op.name.toLowerCase().includes(search.toLowerCase()));
                            }
                            return subAcc;
                        },
                        {} as Record<string, Operator[]>,
                    );
                }
                return acc;
            },
            {} as Record<string, Record<string, Operator[]>>,
        );
    }, [groupedOperators, selectedClasses, selectedSubclasses, search]);

    const handleToggleOperator = (operator: Operator) => {
        const newSelectedOperators = selectedOperators.some((op) => op.id === operator.id) ? selectedOperators.filter((op) => op.id !== operator.id) : [...selectedOperators, operator];

        onSelect(newSelectedOperators);
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
                    <div className="mb-4 flex items-center space-x-2">
                        <Search className="h-4 w-4 text-muted-foreground" />
                        <Input placeholder="Search operators..." value={search} onChange={(e) => setSearch(e.target.value)} className="flex-1" />
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
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button variant="outline">
                                    <span className="mr-2 max-w-32 truncate">{selectedSubclasses.length === 0 ? <span className="font-normal">Filter Subclasses</span> : selectedSubclasses.map((v) => formatSubProfession(v)).join(", ")}</span>
                                    <ChevronsUpDown className="ml-2 h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end" className="max-h-72 overflow-y-scroll">
                                {allSubclasses.map((subclass) => (
                                    <DropdownMenuCheckboxItem key={subclass} checked={selectedSubclasses.includes(subclass)} onCheckedChange={() => handleSubclassToggle(subclass)}>
                                        {formatSubProfession(subclass)}
                                    </DropdownMenuCheckboxItem>
                                ))}
                            </DropdownMenuContent>
                        </DropdownMenu>
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
                                                                    {ops.map((operator) => (
                                                                        <div key={operator.id} className="flex items-center space-x-2">
                                                                            <Checkbox id={operator.id} checked={selectedOperators.some((op) => op.id === operator.id)} onCheckedChange={() => handleToggleOperator(operator)} />
                                                                            <Label htmlFor={operator.id} className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
                                                                                {operator.name}
                                                                            </Label>
                                                                        </div>
                                                                    ))}
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
                        {selectedOperators.map((operator) => (
                            <Badge key={operator.id} variant="secondary">
                                {operator.name}
                                <Button variant="ghost" size="sm" className="ml-2 h-4 w-4 p-0" onClick={() => handleToggleOperator(operator)}>
                                    <X className="h-3 w-3" />
                                </Button>
                            </Badge>
                        ))}
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