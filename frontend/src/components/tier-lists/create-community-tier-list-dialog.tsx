"use client";

import { useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "~/components/ui/shadcn/dialog";
import { Input } from "~/components/ui/shadcn/input";
import { Label } from "~/components/ui/shadcn/label";
import { Textarea } from "~/components/ui/shadcn/textarea";

interface CreateCommunityTierListDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onCreate: (data: { name: string; slug: string; description: string }) => Promise<void>;
}

export function CreateCommunityTierListDialog({ open, onOpenChange, onCreate }: CreateCommunityTierListDialogProps) {
    const [name, setName] = useState("");
    const [slug, setSlug] = useState("");
    const [description, setDescription] = useState("");
    const [creating, setCreating] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const generateSlug = (name: string) => {
        return name
            .toLowerCase()
            .replace(/[^a-z0-9\s-]/g, "")
            .replace(/\s+/g, "-")
            .replace(/-+/g, "-")
            .trim();
    };

    const handleNameChange = (value: string) => {
        setName(value);
        // Auto-generate slug if slug hasn't been manually edited
        if (!slug || slug === generateSlug(name)) {
            setSlug(generateSlug(value));
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError(null);

        if (!name.trim()) {
            setError("Name is required");
            return;
        }
        if (!slug.trim()) {
            setError("Slug is required");
            return;
        }
        if (!/^[a-z0-9-]+$/.test(slug)) {
            setError("Slug can only contain lowercase letters, numbers, and hyphens");
            return;
        }

        setCreating(true);
        try {
            await onCreate({ name: name.trim(), slug: slug.trim(), description: description.trim() });
            // Reset form
            setName("");
            setSlug("");
            setDescription("");
            onOpenChange(false);
        } catch (err) {
            setError(err instanceof Error ? err.message : "Failed to create tier list");
        } finally {
            setCreating(false);
        }
    };

    const handleClose = () => {
        setName("");
        setSlug("");
        setDescription("");
        setError(null);
        onOpenChange(false);
    };

    return (
        <Dialog onOpenChange={handleClose} open={open}>
            <DialogContent className="sm:max-w-[500px]">
                <DialogHeader>
                    <DialogTitle>Create Community Tier List</DialogTitle>
                    <DialogDescription>Create your own tier list to share your operator rankings with the community. You can add tiers and operators after creation.</DialogDescription>
                </DialogHeader>

                <form className="space-y-4" onSubmit={handleSubmit}>
                    <div className="space-y-2">
                        <Label htmlFor="create-name">Name *</Label>
                        <Input id="create-name" onChange={(e) => handleNameChange(e.target.value)} placeholder="e.g., My Operator Rankings" value={name} />
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="create-slug">Slug *</Label>
                        <Input id="create-slug" onChange={(e) => setSlug(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ""))} placeholder="e.g., my-operator-rankings" value={slug} />
                        <p className="text-muted-foreground text-xs">URL-friendly identifier. Only lowercase letters, numbers, and hyphens.</p>
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="create-description">Description</Label>
                        <Textarea id="create-description" onChange={(e) => setDescription(e.target.value)} placeholder="Optional description for your tier list..." rows={3} value={description} />
                    </div>

                    {error && <div className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-destructive text-sm">{error}</div>}

                    <DialogFooter>
                        <Button disabled={creating} onClick={handleClose} type="button" variant="outline">
                            Cancel
                        </Button>
                        <Button disabled={creating || !name.trim() || !slug.trim()} type="submit">
                            {creating ? "Creating..." : "Create Tier List"}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogContent>
        </Dialog>
    );
}
