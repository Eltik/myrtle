"use client";
import { MotionConfig, motion, type Transition, type Variant, type Variants } from "motion/react";
import * as React from "react";
import { createContext, useContext, useEffect, useId, useState } from "react";
import { cn } from "~/lib/utils";

export type DisclosureContextType = {
    open: boolean;
    toggle: () => void;
    variants?: { expanded: Variant; collapsed: Variant };
};

const DisclosureContext = createContext<DisclosureContextType | undefined>(undefined);

export type DisclosureProviderProps = {
    children: React.ReactNode;
    open: boolean;
    onOpenChange?: (open: boolean) => void;
    variants?: { expanded: Variant; collapsed: Variant };
};

function DisclosureProvider({ children, open: openProp, onOpenChange, variants }: DisclosureProviderProps) {
    const [internalOpenValue, setInternalOpenValue] = useState<boolean>(openProp);

    useEffect(() => {
        setInternalOpenValue(openProp);
    }, [openProp]);

    const toggle = () => {
        const newOpen = !internalOpenValue;
        setInternalOpenValue(newOpen);
        if (onOpenChange) {
            onOpenChange(newOpen);
        }
    };

    return (
        <DisclosureContext.Provider
            value={{
                open: internalOpenValue,
                toggle,
                variants,
            }}
        >
            {children}
        </DisclosureContext.Provider>
    );
}

function useDisclosure() {
    const context = useContext(DisclosureContext);
    if (!context) {
        throw new Error("useDisclosure must be used within a DisclosureProvider");
    }
    return context;
}

export type DisclosureProps = {
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    children: React.ReactNode;
    className?: string;
    variants?: { expanded: Variant; collapsed: Variant };
    transition?: Transition;
};

export function Disclosure({ open: openProp = false, onOpenChange, children, className, transition, variants }: DisclosureProps) {
    return (
        <MotionConfig transition={transition}>
            <div className={className}>
                <DisclosureProvider onOpenChange={onOpenChange} open={openProp} variants={variants}>
                    {React.Children.toArray(children)[0]}
                    {React.Children.toArray(children)[1]}
                </DisclosureProvider>
            </div>
        </MotionConfig>
    );
}

export function DisclosureTrigger({ children, className }: { children: React.ReactNode; className?: string }) {
    const { toggle, open } = useDisclosure();

    return (
        <>
            {React.Children.map(children, (child) => {
                if (React.isValidElement(child)) {
                    const childElement = child as React.ReactElement<{
                        className?: string;
                        onClick?: () => void;
                        role?: string;
                        "aria-expanded"?: boolean;
                        tabIndex?: number;
                        onKeyDown?: (e: { key: string; preventDefault: () => void }) => void;
                        "data-state"?: "open" | "closed";
                    }>;
                    return React.cloneElement(childElement, {
                        onClick: toggle,
                        role: "button",
                        "aria-expanded": open,
                        "data-state": open ? "open" : "closed",
                        tabIndex: 0,
                        onKeyDown: (e: { key: string; preventDefault: () => void }) => {
                            if (e.key === "Enter" || e.key === " ") {
                                e.preventDefault();
                                toggle();
                            }
                        },
                        className: cn("group", className, childElement.props.className),
                    });
                }
                return child;
            })}
        </>
    );
}

export function DisclosureContent({ children, className }: { children: React.ReactNode; className?: string }) {
    const { open, variants } = useDisclosure();
    const uniqueId = useId();

    // Use CSS grid technique for smooth height animation without layout shifts
    // grid-template-rows: 0fr -> 1fr animates height without removing from DOM
    const BASE_VARIANTS: Variants = {
        expanded: {
            gridTemplateRows: "1fr",
            opacity: 1,
        },
        collapsed: {
            gridTemplateRows: "0fr",
            opacity: 0,
        },
    };

    const combinedVariants = {
        expanded: { ...BASE_VARIANTS.expanded, ...variants?.expanded },
        collapsed: { ...BASE_VARIANTS.collapsed, ...variants?.collapsed },
    };

    return (
        <motion.div animate={open ? "expanded" : "collapsed"} className={cn("grid", className)} id={uniqueId} initial={false} style={{ willChange: "grid-template-rows, opacity" }} variants={combinedVariants}>
            <div className="overflow-hidden">{children}</div>
        </motion.div>
    );
}

export default {
    Disclosure,
    DisclosureProvider,
    DisclosureTrigger,
    DisclosureContent,
};
