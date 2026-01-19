"use client";

import { XIcon } from "lucide-react";
import { AnimatePresence, MotionConfig, motion, type Transition, type Variant } from "motion/react";
import React, { useCallback, useContext, useEffect, useId, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import useClickOutside from "~/hooks/useClickOutside";
import { cn } from "~/lib/utils";

export type MorphingDialogContextType = {
    isOpen: boolean;
    setIsOpen: (value: boolean | ((prev: boolean) => boolean)) => void;
    uniqueId: string;
    triggerRef: React.RefObject<HTMLButtonElement | null>;
};

const MorphingDialogContext = React.createContext<MorphingDialogContextType | null>(null);

function useMorphingDialog() {
    const context = useContext(MorphingDialogContext);
    if (!context) {
        throw new Error("useMorphingDialog must be used within a MorphingDialogProvider");
    }
    return context;
}

export type MorphingDialogProviderProps = {
    children: React.ReactNode;
    transition?: Transition;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
};

function MorphingDialogProvider({ children, transition, open: controlledOpen, onOpenChange }: MorphingDialogProviderProps) {
    const [uncontrolledOpen, setUncontrolledOpen] = useState(false);
    const uniqueId = useId();
    const triggerRef = useRef<HTMLButtonElement>(null);

    const isControlled = controlledOpen !== undefined;
    const isOpen = isControlled ? controlledOpen : uncontrolledOpen;

    const setIsOpen = useCallback(
        (value: boolean | ((prev: boolean) => boolean)) => {
            const newValue = typeof value === "function" ? value(isOpen) : value;
            if (!isControlled) {
                setUncontrolledOpen(newValue);
            }
            onOpenChange?.(newValue);
        },
        [isControlled, isOpen, onOpenChange],
    );

    const contextValue = useMemo(
        () => ({
            isOpen,
            setIsOpen,
            uniqueId,
            triggerRef,
        }),
        [isOpen, setIsOpen, uniqueId],
    );

    return (
        <MorphingDialogContext.Provider value={contextValue}>
            <MotionConfig transition={transition}>{children}</MotionConfig>
        </MorphingDialogContext.Provider>
    );
}

export type MorphingDialogProps = {
    children: React.ReactNode;
    transition?: Transition;
    open?: boolean;
    onOpenChange?: (open: boolean) => void;
};

function MorphingDialog({ children, transition, open, onOpenChange }: MorphingDialogProps) {
    return (
        <MorphingDialogProvider onOpenChange={onOpenChange} open={open}>
            <MotionConfig transition={transition}>{children}</MotionConfig>
        </MorphingDialogProvider>
    );
}

export type MorphingDialogTriggerProps = {
    children: React.ReactNode;
    className?: string;
    style?: React.CSSProperties;
    triggerRef?: React.RefObject<HTMLButtonElement>;
};

function MorphingDialogTrigger({ children, className, style, triggerRef }: MorphingDialogTriggerProps) {
    const { setIsOpen, isOpen, uniqueId } = useMorphingDialog();

    const handleClick = useCallback(() => {
        setIsOpen(!isOpen);
    }, [isOpen, setIsOpen]);

    const handleKeyDown = useCallback(
        (event: React.KeyboardEvent) => {
            if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                setIsOpen(!isOpen);
            }
        },
        [isOpen, setIsOpen],
    );

    return (
        <motion.button
            aria-controls={`motion-ui-morphing-dialog-content-${uniqueId}`}
            aria-expanded={isOpen}
            aria-haspopup="dialog"
            aria-label={`Open dialog ${uniqueId}`}
            className={cn("relative cursor-pointer", className)}
            layoutId={`dialog-${uniqueId}`}
            onClick={handleClick}
            onKeyDown={handleKeyDown}
            ref={triggerRef}
            style={style}
        >
            {children}
        </motion.button>
    );
}

export type MorphingDialogContentProps = {
    children: React.ReactNode;
    className?: string;
    style?: React.CSSProperties;
};

function MorphingDialogContent({ children, className, style }: MorphingDialogContentProps) {
    const { setIsOpen, isOpen, uniqueId, triggerRef } = useMorphingDialog();
    const containerRef = useRef<HTMLDivElement>(null);
    const [firstFocusableElement, setFirstFocusableElement] = useState<HTMLElement | null>(null);
    const [lastFocusableElement, setLastFocusableElement] = useState<HTMLElement | null>(null);

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            if (event.key === "Escape") {
                setIsOpen(false);
            }
            if (event.key === "Tab") {
                if (!firstFocusableElement || !lastFocusableElement) return;

                if (event.shiftKey) {
                    if (document.activeElement === firstFocusableElement) {
                        event.preventDefault();
                        lastFocusableElement.focus();
                    }
                } else {
                    if (document.activeElement === lastFocusableElement) {
                        event.preventDefault();
                        firstFocusableElement.focus();
                    }
                }
            }
        };

        document.addEventListener("keydown", handleKeyDown);

        return () => {
            document.removeEventListener("keydown", handleKeyDown);
        };
    }, [setIsOpen, firstFocusableElement, lastFocusableElement]);

    useEffect(() => {
        if (isOpen) {
            document.body.classList.add("overflow-hidden");
            const focusableElements = containerRef.current?.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
            if (focusableElements && focusableElements.length > 0) {
                setFirstFocusableElement(focusableElements[0] as HTMLElement);
                setLastFocusableElement(focusableElements[focusableElements.length - 1] as HTMLElement);
                (focusableElements[0] as HTMLElement).focus();
            }
        } else {
            document.body.classList.remove("overflow-hidden");
            triggerRef.current?.focus();
        }
    }, [isOpen, triggerRef]);

    useClickOutside(containerRef, (event) => {
        // Ignore clicks inside Radix UI portals (Select, DropdownMenu, Popover, etc.)
        const target = event.target as HTMLElement;

        // Check for Radix UI portal content (various data attributes and roles)
        const isInsideRadixPortal =
            target.closest("[data-radix-popper-content-wrapper]") ||
            target.closest("[data-radix-select-viewport]") ||
            target.closest("[data-radix-menu-content]") ||
            target.closest("[data-radix-dialog-content]") ||
            target.closest("[data-radix-popover-content]") ||
            target.closest("[data-radix-dropdown-menu-content]") ||
            target.closest("[data-radix-collection-item]") ||
            target.closest("[data-slot='dropdown-menu-content']") ||
            target.closest("[data-slot='dropdown-menu-checkbox-item']") ||
            target.closest("[data-slot='select-content']") ||
            target.closest("[role='listbox']") ||
            target.closest("[role='dialog']") ||
            target.closest("[role='menu']") ||
            target.closest("[role='menuitem']") ||
            target.closest("[role='menuitemcheckbox']") ||
            target.closest("[role='option']");

        if (isInsideRadixPortal) {
            return;
        }
        if (isOpen) {
            setIsOpen(false);
        }
    });

    return (
        <motion.div aria-describedby={`motion-ui-morphing-dialog-description-${uniqueId}`} aria-labelledby={`motion-ui-morphing-dialog-title-${uniqueId}`} aria-modal="true" className={cn("overflow-hidden", className)} layoutId={`dialog-${uniqueId}`} ref={containerRef} role="dialog" style={style}>
            {children}
        </motion.div>
    );
}

export type MorphingDialogContainerProps = {
    children: React.ReactNode;
    className?: string;
    style?: React.CSSProperties;
};

function MorphingDialogContainer({ children }: MorphingDialogContainerProps) {
    const { isOpen, setIsOpen, uniqueId } = useMorphingDialog();
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
        return () => setMounted(false);
    }, []);

    const handleBackdropClick = useCallback(() => {
        setIsOpen(false);
    }, [setIsOpen]);

    // Explicitly handle focus for inputs on mobile - the pointer-events manipulation can interfere with native focus
    const handleContentTouchEnd = useCallback((e: React.TouchEvent) => {
        const target = e.target as HTMLElement;
        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
            // Use setTimeout to ensure focus happens after all event processing
            setTimeout(() => target.focus(), 0);
        }
    }, []);

    if (!mounted) return null;

    return createPortal(
        <AnimatePresence initial={false} mode="sync">
            {isOpen && (
                <>
                    {/* Backdrop - clickable to close */}
                    <motion.div animate={{ opacity: 1 }} className="fixed inset-0 z-[100] bg-black/60 backdrop-blur-sm" exit={{ opacity: 0 }} initial={{ opacity: 0 }} key={`backdrop-${uniqueId}`} onClick={handleBackdropClick} />
                    {/* Content container - pointer-events-none so clicks pass through to backdrop, but children have pointer-events-auto */}
                    <div className="pointer-events-none fixed inset-0 z-[101] flex items-center justify-center">
                        <div className="pointer-events-auto" onTouchEnd={handleContentTouchEnd} style={{ touchAction: "auto" }}>
                            {children}
                        </div>
                    </div>
                </>
            )}
        </AnimatePresence>,
        document.body,
    );
}

export type MorphingDialogTitleProps = {
    children: React.ReactNode;
    className?: string;
    style?: React.CSSProperties;
};

function MorphingDialogTitle({ children, className, style }: MorphingDialogTitleProps) {
    const { uniqueId } = useMorphingDialog();

    return (
        <motion.div className={className} layout layoutId={`dialog-title-container-${uniqueId}`} style={style}>
            {children}
        </motion.div>
    );
}

export type MorphingDialogSubtitleProps = {
    children: React.ReactNode;
    className?: string;
    style?: React.CSSProperties;
};

function MorphingDialogSubtitle({ children, className, style }: MorphingDialogSubtitleProps) {
    const { uniqueId } = useMorphingDialog();

    return (
        <motion.div className={className} layoutId={`dialog-subtitle-container-${uniqueId}`} style={style}>
            {children}
        </motion.div>
    );
}

export type MorphingDialogDescriptionProps = {
    children: React.ReactNode;
    className?: string;
    disableLayoutAnimation?: boolean;
    variants?: {
        initial: Variant;
        animate: Variant;
        exit: Variant;
    };
};

function MorphingDialogDescription({ children, className, variants, disableLayoutAnimation }: MorphingDialogDescriptionProps) {
    const { uniqueId } = useMorphingDialog();

    return (
        <motion.div animate="animate" className={className} exit="exit" id={`dialog-description-${uniqueId}`} initial="initial" key={`dialog-description-${uniqueId}`} layoutId={disableLayoutAnimation ? undefined : `dialog-description-content-${uniqueId}`} variants={variants}>
            {children}
        </motion.div>
    );
}

export type MorphingDialogImageProps = {
    src: string;
    alt: string;
    className?: string;
    style?: React.CSSProperties;
};

function MorphingDialogImage({ src, alt, className, style }: MorphingDialogImageProps) {
    const { uniqueId } = useMorphingDialog();

    // biome-ignore lint/performance/noImgElement: motion.img required for layout animation
    return <motion.img alt={alt} className={cn(className)} layoutId={`dialog-img-${uniqueId}`} src={src} style={style} />;
}

export type MorphingDialogCloseProps = {
    children?: React.ReactNode;
    className?: string;
    variants?: {
        initial: Variant;
        animate: Variant;
        exit: Variant;
    };
};

function MorphingDialogClose({ children, className, variants }: MorphingDialogCloseProps) {
    const { setIsOpen, uniqueId } = useMorphingDialog();

    const handleClose = useCallback(() => {
        setIsOpen(false);
    }, [setIsOpen]);

    return (
        <motion.button animate="animate" aria-label="Close dialog" className={cn("absolute top-6 right-6", className)} exit="exit" initial="initial" key={`dialog-close-${uniqueId}`} onClick={handleClose} type="button" variants={variants}>
            {children || <XIcon size={24} />}
        </motion.button>
    );
}

export { MorphingDialog, MorphingDialogTrigger, MorphingDialogContainer, MorphingDialogContent, MorphingDialogClose, MorphingDialogTitle, MorphingDialogSubtitle, MorphingDialogDescription, MorphingDialogImage };
