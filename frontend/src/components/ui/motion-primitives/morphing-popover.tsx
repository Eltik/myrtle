"use client";

import { AnimatePresence, MotionConfig, motion, type Transition, type Variants } from "motion/react";
import { createContext, type RefObject, useContext, useEffect, useRef, useState } from "react";
import { cn } from "~/lib/utils";

const TRANSITION: Transition = {
    type: "spring" as const,
    bounce: 0.1,
    duration: 0.4,
};

type MorphingPopoverContextValue = {
    isOpen: boolean;
    open: () => void;
    close: () => void;
    toggle: () => void;
    triggerRef: RefObject<HTMLDivElement | null>;
    variants?: Variants;
};

const MorphingPopoverContext = createContext<MorphingPopoverContextValue | null>(null);

function usePopoverLogic({ defaultOpen = false, open: controlledOpen, onOpenChange }: { defaultOpen?: boolean; open?: boolean; onOpenChange?: (open: boolean) => void } = {}) {
    const [uncontrolledOpen, setUncontrolledOpen] = useState(defaultOpen);
    const triggerRef = useRef<HTMLDivElement>(null);

    const isOpen = controlledOpen ?? uncontrolledOpen;

    const open = () => {
        if (controlledOpen === undefined) {
            setUncontrolledOpen(true);
        }
        onOpenChange?.(true);
    };

    const close = () => {
        if (controlledOpen === undefined) {
            setUncontrolledOpen(false);
        }
        onOpenChange?.(false);
    };

    const toggle = () => {
        if (isOpen) {
            close();
        } else {
            open();
        }
    };

    return { isOpen, open, close, toggle, triggerRef };
}

export type MorphingPopoverProps = {
    children: React.ReactNode;
    transition?: Transition;
    defaultOpen?: boolean;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
    variants?: Variants;
    className?: string;
} & React.ComponentProps<"div">;

function MorphingPopover({ children, transition = TRANSITION, defaultOpen, open, onOpenChange, variants, className, ...props }: MorphingPopoverProps) {
    const popoverLogic = usePopoverLogic({ defaultOpen, open, onOpenChange });

    return (
        <MorphingPopoverContext.Provider value={{ ...popoverLogic, variants }}>
            <MotionConfig transition={transition}>
                <div className={cn("relative", className)} {...props}>
                    {children}
                </div>
            </MotionConfig>
        </MorphingPopoverContext.Provider>
    );
}

export type MorphingPopoverTriggerProps = {
    children: React.ReactNode;
    className?: string;
};

function MorphingPopoverTrigger({ children, className }: MorphingPopoverTriggerProps) {
    const context = useContext(MorphingPopoverContext);
    if (!context) {
        throw new Error("MorphingPopoverTrigger must be used within MorphingPopover");
    }

    return (
        // biome-ignore lint/a11y/useSemanticElements: This wraps arbitrary children which may include buttons, using a semantic button could cause nested button issues
        <div
            className={cn("cursor-pointer", className)}
            onClick={context.toggle}
            onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    context.toggle();
                }
            }}
            ref={context.triggerRef}
            role="button"
            tabIndex={0}
        >
            {children}
        </div>
    );
}

export type MorphingPopoverContentProps = {
    children: React.ReactNode;
    className?: string;
} & React.ComponentProps<typeof motion.div>;

function MorphingPopoverContent({ children, className, ...props }: MorphingPopoverContentProps) {
    const context = useContext(MorphingPopoverContext);
    if (!context) throw new Error("MorphingPopoverContent must be used within MorphingPopover");

    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!context.isOpen) return;

        const handleClickOutside = (event: MouseEvent | TouchEvent) => {
            const target = event.target as HTMLElement;

            // Don't close if clicking inside the content
            if (ref.current?.contains(target)) {
                return;
            }

            // Don't close if clicking the trigger (toggle will handle it)
            if (context.triggerRef.current?.contains(target)) {
                return;
            }

            // Don't close if clicking inside Radix UI portals (Select, DropdownMenu, Popover, etc.)
            if (target.closest("[data-radix-popper-content-wrapper]") || target.closest("[data-radix-select-viewport]") || target.closest("[data-radix-menu-content]") || target.closest("[role='listbox']")) {
                return;
            }

            context.close();
        };

        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === "Escape") context.close();
        };

        // Use mousedown to match click timing
        document.addEventListener("mousedown", handleClickOutside);
        document.addEventListener("touchstart", handleClickOutside);
        document.addEventListener("keydown", handleKeyDown);

        return () => {
            document.removeEventListener("mousedown", handleClickOutside);
            document.removeEventListener("touchstart", handleClickOutside);
            document.removeEventListener("keydown", handleKeyDown);
        };
    }, [context.isOpen, context.close, context.triggerRef]);

    return (
        <AnimatePresence>
            {context.isOpen && (
                <motion.div
                    {...props}
                    animate={{ opacity: 1, scale: 1, y: 0 }}
                    aria-modal="true"
                    className={cn("absolute top-full right-0 z-50 mt-2 overflow-hidden rounded-lg border border-border bg-card shadow-lg", className)}
                    exit={{ opacity: 0, scale: 0.95, y: -10 }}
                    initial={{ opacity: 0, scale: 0.95, y: -10 }}
                    ref={ref}
                    role="dialog"
                    variants={context.variants}
                >
                    {children}
                </motion.div>
            )}
        </AnimatePresence>
    );
}

export { MorphingPopover, MorphingPopoverTrigger, MorphingPopoverContent };
