import { X } from "lucide-react";
import { motion } from "motion/react";

interface FilterPillProps {
    label: string;
    onRemove: () => void;
}

export function FilterPill({ label, onRemove }: FilterPillProps) {
    return (
        <motion.div animate={{ opacity: 1, scale: 1 }} className="flex items-center gap-1 rounded-full bg-primary/10 py-1 pr-1 pl-3 text-sm" exit={{ opacity: 0, scale: 0.9 }} initial={{ opacity: 0, scale: 0.9 }} layout>
            <span className="text-primary">{label}</span>
            <button
                className="flex h-5 w-5 items-center justify-center rounded-full transition-colors hover:bg-primary/20"
                onClick={(e) => {
                    e.preventDefault();
                    onRemove();
                }}
                type="button"
            >
                <X className="h-3 w-3 text-primary" />
            </button>
        </motion.div>
    );
}
