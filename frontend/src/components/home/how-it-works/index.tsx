"use client";
import { ArrowRight } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { LoginContent } from "~/components/layout/login/impl/login-content";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { Dialog, DialogContent } from "~/components/ui/shadcn/dialog";
import { STEPS } from "./impl/constants";

export function HowItWorksSection() {
    const [loginOpen, setLoginOpen] = useState(false);

    return (
        <section className="relative py-20 md:py-32">
            {/* Background decoration */}
            <div className="pointer-events-none absolute inset-0 overflow-hidden">
                <div className="absolute top-20 left-1/4 h-72 w-72 rounded-full bg-primary/5 blur-3xl" />
                <div className="absolute right-1/4 bottom-20 h-96 w-96 rounded-full bg-primary/5 blur-3xl" />
            </div>

            <div className="container relative mx-auto px-4">
                <InView
                    once
                    transition={{ duration: 0.6, ease: "easeOut" }}
                    variants={{
                        hidden: { opacity: 0, y: 20 },
                        visible: { opacity: 1, y: 0 },
                    }}
                >
                    <div className="mb-16 text-center">
                        <h2 className="mb-4 text-balance font-bold text-3xl md:text-5xl">How it works</h2>
                        <p className="mx-auto max-w-2xl text-balance text-lg text-muted-foreground md:text-xl">Get started in minutes and take your gameplay to the next level.</p>
                    </div>
                </InView>

                <div className="mx-auto max-w-5xl">
                    <div className="grid gap-8 md:grid-cols-2 lg:gap-12">
                        {STEPS.map((step, index) => (
                            <InView
                                key={step.number}
                                once
                                transition={{
                                    duration: 0.6,
                                    ease: "easeOut",
                                    delay: index * 0.15,
                                }}
                                variants={{
                                    hidden: { opacity: 0, x: index % 2 === 0 ? -30 : 30 },
                                    visible: { opacity: 1, x: 0 },
                                }}
                            >
                                <div className="group relative min-h-40 rounded-2xl border border-transparent p-4 transition-all duration-300 ease-out hover:border-border hover:bg-card">
                                    <div className="flex gap-6">
                                        <div className="shrink-0">
                                            <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-primary/10 font-bold text-2xl text-primary transition-all duration-300 group-hover:bg-primary group-hover:text-primary-foreground group-hover:shadow-lg group-hover:shadow-primary/20">{step.number}</div>
                                        </div>
                                        <div className="flex flex-1 flex-col pt-1">
                                            <h3 className="mb-2 font-semibold text-lg transition-all duration-300 ease-out group-hover:mb-1">{step.title}</h3>
                                            <p className="text-muted-foreground text-sm leading-relaxed">{step.description}</p>
                                            <div className="pt-2">
                                                {index === 0 ? (
                                                    <>
                                                        <Button className="translate-y-3 cursor-pointer gap-1.5 opacity-0 transition-all duration-300 ease-out group-hover:translate-y-0 group-hover:opacity-100" onClick={() => setLoginOpen(true)} size="sm">
                                                            {step.action.label}
                                                            <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                                        </Button>
                                                        <Dialog onOpenChange={setLoginOpen} open={loginOpen}>
                                                            <DialogContent className="border-none bg-transparent p-0 shadow-none" showCloseButton={false}>
                                                                <LoginContent onSuccess={() => setLoginOpen(false)} />
                                                            </DialogContent>
                                                        </Dialog>
                                                    </>
                                                ) : (
                                                    <Button asChild className="translate-y-3 gap-1.5 opacity-0 transition-all duration-300 ease-out group-hover:translate-y-0 group-hover:opacity-100" size="sm">
                                                        <Link href={(step.action as { href: string }).href}>
                                                            {step.action.label}
                                                            <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                                                        </Link>
                                                    </Button>
                                                )}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </InView>
                        ))}
                    </div>
                </div>
            </div>
        </section>
    );
}
