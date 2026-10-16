"use client";

import { AnimatePresence, motion, usePresence, useReducedMotion } from "motion/react";
import type { ComponentProps, ReactElement } from "react";
import { Fragment } from "react";
import { cn } from "#/lib/utils";

export type ChatTypingIndicatorProps = Omit<ComponentProps<"div">, "children"> & {
    label: string;
    visible?: boolean;
};

export function ChatTypingIndicator({ label, visible = true, className, ...props }: ChatTypingIndicatorProps): ReactElement {
    return (
        <AnimatePresence>
            {visible && (
                <Fragment key="typing">
                    <ChatTypingIndicatorContent label={label} className={className} {...props} />
                </Fragment>
            )}
        </AnimatePresence>
    );
}

function ChatTypingIndicatorContent({ label, className, ...props }: Omit<ChatTypingIndicatorProps, "visible">): ReactElement {
    const reduceMotion = useReducedMotion();
    const [isPresent, safeToRemove] = usePresence();
    return (
        <motion.div
            className="overflow-hidden"
            initial={{ height: 0, opacity: 0 }}
            animate={isPresent ? { height: "auto", opacity: 1 } : { height: 0, opacity: 0 }}
            transition={{
                duration: reduceMotion ? 0 : 0.2,
                ease: [0.22, 1, 0.36, 1],
            }}
            onAnimationComplete={() => {
                if (!isPresent) safeToRemove?.();
            }}
        >
            {/* biome-ignore lint/a11y/useSemanticElements: coss registry markup kept verbatim; a live status region, not form output */}
            <div role="status" data-slot="chat-typing-indicator" className={cn("flex items-center gap-2.5", className)} {...props}>
                <motion.div
                    aria-hidden="true"
                    className="flex origin-bottom-left items-center gap-1 rounded-2xl rounded-es-md bg-muted/72 px-3 py-3.5 rtl:origin-bottom-right"
                    initial={{ transform: reduceMotion ? "scale(1)" : "scale(0.92)" }}
                    animate={{
                        transform: isPresent || reduceMotion ? "scale(1)" : "scale(0.96)",
                    }}
                    transition={{
                        duration: reduceMotion ? 0 : 0.2,
                        ease: [0.22, 1, 0.36, 1],
                    }}
                >
                    {[0, 1, 2].map((dot) => (
                        <motion.span
                            key={dot}
                            className="block size-1.5 rounded-full bg-muted-foreground"
                            initial={{
                                transform: "translateY(0px) scale(0.8)",
                                opacity: 0.4,
                            }}
                            animate={
                                reduceMotion
                                    ? { transform: "translateY(0px) scale(1)", opacity: 1 }
                                    : {
                                          transform: ["translateY(0px) scale(0.8)", "translateY(-2px) scale(1.1)", "translateY(0px) scale(0.8)", "translateY(0px) scale(0.8)"],
                                          opacity: [0.4, 1, 0.4, 0.4],
                                      }
                            }
                            transition={
                                reduceMotion
                                    ? { duration: 0 }
                                    : {
                                          duration: 1.4,
                                          times: [0, 0.22, 0.44, 1],
                                          delay: dot * 0.16,
                                          repeat: Number.POSITIVE_INFINITY,
                                          ease: "easeInOut",
                                      }
                            }
                        />
                    ))}
                </motion.div>
                <span className="min-w-0 text-muted-foreground text-xs">{label}</span>
            </div>
        </motion.div>
    );
}
