import { X } from "lucide-react";

interface FilterTagProps {
    label: string;
    onRemove: () => void;
}

export function FilterTag({ label, onRemove }: FilterTagProps) {
    return (
        <span className="inline-flex items-center gap-1 rounded-md bg-primary/20 px-2 py-0.5 text-foreground text-xs">
            {label}
            <button className="hover:text-destructive" onClick={onRemove} type="button">
                <X className="h-3 w-3" />
            </button>
        </span>
    );
}
