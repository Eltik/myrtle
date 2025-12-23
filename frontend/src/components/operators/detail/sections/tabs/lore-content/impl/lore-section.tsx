"use client";

import { ChevronDown } from "lucide-react";
import { motion } from "motion/react";
import type React from "react";
import { memo, useCallback, useState } from "react";
import { Disclosure, DisclosureContent, DisclosureTrigger } from "~/components/ui/motion-primitives/disclosure";

interface LoreSectionProps {
    title: string;
    icon: React.ElementType;
    children: React.ReactNode;
    defaultOpen?: boolean;
}

/**
 * Collapsible section component for lore entries
 */
export const LoreSection = memo(function LoreSection({ title, icon: Icon, children, defaultOpen = false }: LoreSectionProps) {
    const [isOpen, setIsOpen] = useState(defaultOpen);

    const handleToggle = useCallback((open: boolean) => {
        setIsOpen(open);
    }, []);

    return (
        <Disclosure onOpenChange={handleToggle} open={isOpen} transition={{ type: "spring", stiffness: 300, damping: 30 }}>
            <DisclosureTrigger>
                <div className="flex w-full cursor-pointer items-center justify-between rounded-lg border border-border bg-secondary/30 px-4 py-3 transition-colors hover:bg-secondary/50">
                    <div className="flex items-center gap-3">
                        <Icon className="h-4 w-4 text-primary" />
                        <span className="font-medium text-sm">{title}</span>
                    </div>
                    <motion.div animate={{ rotate: isOpen ? 180 : 0 }} className="will-change-transform" transition={{ type: "spring", stiffness: 300, damping: 30 }}>
                        <ChevronDown className="h-4 w-4 text-muted-foreground" />
                    </motion.div>
                </div>
            </DisclosureTrigger>
            <DisclosureContent>
                <div className="mt-3 rounded-lg border border-border/50 bg-card/30 p-4">{children}</div>
            </DisclosureContent>
        </Disclosure>
    );
});
