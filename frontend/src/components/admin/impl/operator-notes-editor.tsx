"use client";

import { Loader2, Save, Search, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";
import type { AdminRole } from "~/lib/permissions";
import { canCreateTierList } from "~/lib/permissions";
import type { OperatorFromList } from "~/types/api";
import type { OperatorNote } from "~/types/api/impl/operator-notes";
import { Badge } from "../../ui/shadcn/badge";
import { Button } from "../../ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../ui/shadcn/card";
import { Input } from "../../ui/shadcn/input";
import { Textarea } from "../../ui/shadcn/textarea";

interface OperatorNotesEditorProps {
    role: AdminRole;
}

export function OperatorNotesEditor({ role }: OperatorNotesEditorProps) {
    const [operators, setOperators] = useState<OperatorFromList[]>([]);
    const [existingNotes, setExistingNotes] = useState<Record<string, OperatorNote>>({});
    const [operatorsLoading, setOperatorsLoading] = useState(true);
    const [searchQuery, setSearchQuery] = useState("");
    const [selectedOperatorId, setSelectedOperatorId] = useState<string | null>(null);

    // Form state
    const [summary, setSummary] = useState("");
    const [pros, setPros] = useState("");
    const [cons, setCons] = useState("");
    const [notes, setNotes] = useState("");
    const [trivia, setTrivia] = useState("");
    const [tags, setTags] = useState<string[]>([]);
    const [tagInput, setTagInput] = useState("");
    const [saving, setSaving] = useState(false);
    const [loadingNotes, setLoadingNotes] = useState(false);

    const canEdit = canCreateTierList(role);

    // Fetch operators list
    useEffect(() => {
        async function fetchOperators() {
            setOperatorsLoading(true);
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({
                        type: "operators",
                        limit: 1000,
                        fields: ["id", "name", "rarity", "profession"],
                    }),
                });
                const json = await response.json();
                const ops = json.data || json.operators;
                if (ops && Array.isArray(ops)) {
                    setOperators(ops);
                }
            } catch (error) {
                console.error("Failed to fetch operators:", error);
            } finally {
                setOperatorsLoading(false);
            }
        }

        async function fetchExistingNotes() {
            try {
                const response = await fetch("/api/operator-notes");
                if (response.ok) {
                    const data: OperatorNote[] = await response.json();
                    const notesMap: Record<string, OperatorNote> = {};
                    for (const note of data) {
                        notesMap[note.operator_id] = note;
                    }
                    setExistingNotes(notesMap);
                }
            } catch (error) {
                console.error("Failed to fetch existing notes:", error);
            }
        }

        fetchOperators();
        fetchExistingNotes();
    }, []);

    const filteredOperators = useMemo(() => {
        if (!searchQuery.trim()) return operators;
        const query = searchQuery.toLowerCase();
        return operators.filter((op) => op.name?.toLowerCase().includes(query) || op.id?.toLowerCase().includes(query));
    }, [operators, searchQuery]);

    const selectedOperator = useMemo(() => {
        if (!selectedOperatorId) return null;
        return operators.find((op) => op.id === selectedOperatorId) ?? null;
    }, [operators, selectedOperatorId]);

    // Load notes when selecting an operator
    const selectOperator = useCallback(
        async (operatorId: string) => {
            setSelectedOperatorId(operatorId);
            setLoadingNotes(true);

            // Check if we already have notes cached
            const cached = existingNotes[operatorId];
            if (cached) {
                setSummary(cached.summary ?? "");
                setPros(cached.pros ?? "");
                setCons(cached.cons ?? "");
                setNotes(cached.notes ?? "");
                setTrivia(cached.trivia ?? "");
                setTags(cached.tags ?? []);
                setLoadingNotes(false);
                return;
            }

            // Try fetching from API
            try {
                const response = await fetch(`/api/operator-notes/${operatorId}`);
                if (response.ok) {
                    const data: OperatorNote = await response.json();
                    setSummary(data.summary ?? "");
                    setPros(data.pros ?? "");
                    setCons(data.cons ?? "");
                    setNotes(data.notes ?? "");
                    setTrivia(data.trivia ?? "");
                    setTags(data.tags ?? []);
                } else {
                    // No existing notes — clear form
                    setSummary("");
                    setPros("");
                    setCons("");
                    setNotes("");
                    setTrivia("");
                    setTags([]);
                }
            } catch {
                setSummary("");
                setPros("");
                setCons("");
                setNotes("");
                setTrivia("");
                setTags([]);
            } finally {
                setLoadingNotes(false);
            }
        },
        [existingNotes],
    );

    const addTag = useCallback(() => {
        const trimmed = tagInput.trim();
        if (trimmed && !tags.includes(trimmed)) {
            setTags((prev) => [...prev, trimmed]);
            setTagInput("");
        }
    }, [tagInput, tags]);

    const removeTag = useCallback((tag: string) => {
        setTags((prev) => prev.filter((t) => t !== tag));
    }, []);

    const handleSave = useCallback(async () => {
        if (!selectedOperatorId) return;

        setSaving(true);
        try {
            const response = await fetch(`/api/operator-notes/${selectedOperatorId}/update`, {
                method: "PUT",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    pros,
                    cons,
                    notes,
                    trivia,
                    summary,
                    tags,
                }),
            });

            if (!response.ok) {
                const data = await response.json().catch(() => ({}));
                throw new Error((data as { error?: string }).error || "Failed to save operator notes");
            }

            const savedData: OperatorNote = await response.json();
            setExistingNotes((prev) => ({ ...prev, [selectedOperatorId]: savedData }));
            toast.success("Operator notes saved successfully");
        } catch (err) {
            toast.error(err instanceof Error ? err.message : "Failed to save operator notes");
        } finally {
            setSaving(false);
        }
    }, [selectedOperatorId, pros, cons, notes, trivia, summary, tags]);

    if (!canEdit) {
        return null;
    }

    return (
        <Card>
            <CardHeader>
                <CardTitle>Operator Notes</CardTitle>
                <CardDescription>Add or edit notes, pros/cons, and trivia for operators</CardDescription>
            </CardHeader>
            <CardContent>
                <div className="grid gap-6 lg:grid-cols-[300px_1fr]">
                    {/* Operator List */}
                    <div className="space-y-3">
                        <div className="relative">
                            <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                            <Input className="pl-9" onChange={(e) => setSearchQuery(e.target.value)} placeholder="Search operators..." value={searchQuery} />
                        </div>

                        <div className="max-h-[500px] space-y-1 overflow-y-auto rounded-md border border-border p-2">
                            {operatorsLoading ? (
                                <div className="flex items-center justify-center py-8">
                                    <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                                </div>
                            ) : (
                                filteredOperators.map((op) => (
                                    <button className={`flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors hover:bg-secondary/50 ${selectedOperatorId === op.id ? "bg-secondary" : ""}`} key={op.id} onClick={() => op.id && selectOperator(op.id)} type="button">
                                        <span className="truncate">{op.name}</span>
                                        {op.id && existingNotes[op.id] && <Badge variant="secondary">Has notes</Badge>}
                                    </button>
                                ))
                            )}
                        </div>
                    </div>

                    {/* Editor Form */}
                    <div className="space-y-4">
                        {!selectedOperator ? (
                            <div className="flex min-h-[300px] items-center justify-center text-muted-foreground text-sm">Select an operator to edit notes</div>
                        ) : loadingNotes ? (
                            <div className="flex min-h-[300px] items-center justify-center">
                                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                            </div>
                        ) : (
                            <>
                                <h3 className="font-semibold text-lg">{selectedOperator.name}</h3>

                                {/* Summary */}
                                <div className="space-y-1.5">
                                    <label className="font-medium text-sm" htmlFor="notes-summary">
                                        Summary <span className="text-muted-foreground">({summary.length}/500)</span>
                                    </label>
                                    <Input id="notes-summary" maxLength={500} onChange={(e) => setSummary(e.target.value)} placeholder="Brief summary of the operator..." value={summary} />
                                </div>

                                {/* Pros */}
                                <div className="space-y-1.5">
                                    <label className="font-medium text-sm" htmlFor="notes-pros">
                                        Pros
                                    </label>
                                    <Textarea id="notes-pros" onChange={(e) => setPros(e.target.value)} placeholder="Strengths and advantages..." rows={3} value={pros} />
                                </div>

                                {/* Cons */}
                                <div className="space-y-1.5">
                                    <label className="font-medium text-sm" htmlFor="notes-cons">
                                        Cons
                                    </label>
                                    <Textarea id="notes-cons" onChange={(e) => setCons(e.target.value)} placeholder="Weaknesses and disadvantages..." rows={3} value={cons} />
                                </div>

                                {/* Notes */}
                                <div className="space-y-1.5">
                                    <label className="font-medium text-sm" htmlFor="notes-notes">
                                        Notes
                                    </label>
                                    <Textarea id="notes-notes" onChange={(e) => setNotes(e.target.value)} placeholder="General usage notes..." rows={4} value={notes} />
                                </div>

                                {/* Trivia */}
                                <div className="space-y-1.5">
                                    <label className="font-medium text-sm" htmlFor="notes-trivia">
                                        Trivia
                                    </label>
                                    <Textarea id="notes-trivia" onChange={(e) => setTrivia(e.target.value)} placeholder="Fun facts and trivia..." rows={3} value={trivia} />
                                </div>

                                {/* Tags */}
                                <div className="space-y-1.5">
                                    <label className="font-medium text-sm" htmlFor="notes-tag-input">
                                        Tags
                                    </label>
                                    <div className="flex gap-2">
                                        <Input
                                            id="notes-tag-input"
                                            onChange={(e) => setTagInput(e.target.value)}
                                            onKeyDown={(e) => {
                                                if (e.key === "Enter") {
                                                    e.preventDefault();
                                                    addTag();
                                                }
                                            }}
                                            placeholder="Add a tag..."
                                            value={tagInput}
                                        />
                                        <Button onClick={addTag} size="sm" type="button" variant="outline">
                                            Add
                                        </Button>
                                    </div>
                                    {tags.length > 0 && (
                                        <div className="flex flex-wrap gap-2 pt-2">
                                            {tags.map((tag) => (
                                                <Badge className="gap-1" key={tag} variant="secondary">
                                                    {tag}
                                                    <button className="ml-1 rounded-full hover:text-destructive" onClick={() => removeTag(tag)} type="button">
                                                        <X className="h-3 w-3" />
                                                    </button>
                                                </Badge>
                                            ))}
                                        </div>
                                    )}
                                </div>

                                {/* Save Button */}
                                <Button className="w-full" disabled={saving} onClick={handleSave}>
                                    {saving ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Save className="mr-2 h-4 w-4" />}
                                    {saving ? "Saving..." : "Save Notes"}
                                </Button>
                            </>
                        )}
                    </div>
                </div>
            </CardContent>
        </Card>
    );
}
