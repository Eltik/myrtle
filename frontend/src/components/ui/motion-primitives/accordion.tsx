"use client";
import { MotionConfig, motion, type Transition, type Variant, type Variants } from "motion/react";
import React, { createContext, type ReactNode, useContext, useState } from "react";
import { cn } from "~/lib/utils";

export type AccordionContextType = {
    expandedValue: React.Key | null;
    toggleItem: (value: React.Key) => void;
    variants?: { expanded: Variant; collapsed: Variant };
};

const AccordionContext = createContext<AccordionContextType | undefined>(undefined);

function useAccordion() {
    const context = useContext(AccordionContext);
    if (!context) {
        throw new Error("useAccordion must be used within an AccordionProvider");
    }
    return context;
}

export type AccordionProviderProps = {
    children: ReactNode;
    variants?: { expanded: Variant; collapsed: Variant };
    expandedValue?: React.Key | null;
    onValueChange?: (value: React.Key | null) => void;
};

function AccordionProvider({ children, variants, expandedValue: externalExpandedValue, onValueChange }: AccordionProviderProps) {
    const [internalExpandedValue, setInternalExpandedValue] = useState<React.Key | null>(null);

    const expandedValue = externalExpandedValue !== undefined ? externalExpandedValue : internalExpandedValue;

    const toggleItem = (value: React.Key) => {
        const newValue = expandedValue === value ? null : value;
        if (onValueChange) {
            onValueChange(newValue);
        } else {
            setInternalExpandedValue(newValue);
        }
    };

    return <AccordionContext.Provider value={{ expandedValue, toggleItem, variants }}>{children}</AccordionContext.Provider>;
}

export type AccordionProps = {
    children: ReactNode;
    className?: string;
    transition?: Transition;
    variants?: { expanded: Variant; collapsed: Variant };
    expandedValue?: React.Key | null;
    onValueChange?: (value: React.Key | null) => void;
};

function Accordion({ children, className, transition, variants, expandedValue, onValueChange }: AccordionProps) {
    return (
        <MotionConfig transition={transition}>
            <div className={cn("relative", className)} data-orientation="vertical">
                <AccordionProvider expandedValue={expandedValue} onValueChange={onValueChange} variants={variants}>
                    {children}
                </AccordionProvider>
            </div>
        </MotionConfig>
    );
}

export type AccordionItemProps = {
    value: React.Key;
    children: ReactNode;
    className?: string;
};

function AccordionItem({ value, children, className }: AccordionItemProps) {
    const { expandedValue } = useAccordion();
    const isExpanded = value === expandedValue;

    return (
        <div className={cn("overflow-hidden", className)} {...(isExpanded ? { "data-expanded": "" } : { "data-closed": "" })}>
            {React.Children.map(children, (child) => {
                if (React.isValidElement(child)) {
                    return React.cloneElement(child as React.ReactElement<{ value?: React.Key; expanded?: boolean }>, {
                        value,
                        expanded: isExpanded,
                    });
                }
                return child;
            })}
        </div>
    );
}

export type AccordionTriggerProps = {
    children: ReactNode;
    className?: string;
};

function AccordionTrigger({ children, className, ...props }: AccordionTriggerProps) {
    const { toggleItem, expandedValue } = useAccordion();
    const value = (props as { value?: React.Key }).value;
    const isExpanded = value === expandedValue;

    return (
        <button aria-expanded={isExpanded} className={cn("group", className)} onClick={() => value !== undefined && toggleItem(value)} type="button" {...(isExpanded ? { "data-expanded": "" } : { "data-closed": "" })}>
            {children}
        </button>
    );
}

export type AccordionContentProps = {
    children: ReactNode;
    className?: string;
};

function AccordionContent({ children, className, ...props }: AccordionContentProps) {
    const { expandedValue, variants } = useAccordion();
    const value = (props as { value?: React.Key }).value;
    const isExpanded = value === expandedValue;

    // Use CSS grid technique for smooth height animation without layout shifts
    // grid-template-rows: 0fr -> 1fr animates height without removing from DOM
    const BASE_VARIANTS: Variants = {
        expanded: { gridTemplateRows: "1fr", opacity: 1 },
        collapsed: { gridTemplateRows: "0fr", opacity: 0 },
    };

    const combinedVariants = {
        expanded: { ...BASE_VARIANTS.expanded, ...variants?.expanded },
        collapsed: { ...BASE_VARIANTS.collapsed, ...variants?.collapsed },
    };

    return (
        <motion.div animate={isExpanded ? "expanded" : "collapsed"} className={cn("grid", className)} initial={false} style={{ willChange: "grid-template-rows, opacity" }} variants={combinedVariants}>
            <div className="overflow-hidden">{children}</div>
        </motion.div>
    );
}

export { Accordion, AccordionItem, AccordionTrigger, AccordionContent };
