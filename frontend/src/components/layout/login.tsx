"use client";

import { ChevronDown, Loader2, User, X } from "lucide-react";
import { useRef, useState } from "react";
import { toast } from "sonner";
import { useAuth } from "~/hooks/use-auth";
import { cn } from "~/lib/utils";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "../ui/motion-primitives/accordion";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "../ui/motion-primitives/morphing-dialog";
import { Button } from "../ui/shadcn/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "../ui/shadcn/card";
import { Input } from "../ui/shadcn/input";
import { InputOTP, InputOTPGroup, InputOTPSlot } from "../ui/shadcn/input-otp";
import { Label } from "../ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/shadcn/select";

type AKServer = "en" | "jp" | "kr" | "cn" | "bili" | "tw";

interface LoginProps {
    variant?: "default" | "header";
}

interface SendCodeResponse {
    success: boolean;
    error?: string;
}

interface LoginResponse {
    success: boolean;
    error?: string;
}

const SERVER_OPTIONS: { value: AKServer; label: string }[] = [
    { value: "en", label: "Global (EN)" },
    { value: "jp", label: "Japan (JP)" },
    { value: "kr", label: "Korea (KR)" },
    { value: "cn", label: "China (CN)" },
    { value: "bili", label: "Bilibili" },
    { value: "tw", label: "Taiwan (TW)" },
];

function LoginTriggerButton({ variant = "default" }: { variant?: "default" | "header" }) {
    if (variant === "header") {
        return (
            <div className="flex h-8 items-center gap-2 rounded-md border border-border bg-transparent px-3 text-foreground text-sm transition-colors hover:bg-secondary">
                <User className="h-3.5 w-3.5" />
                <span className="font-medium">Login</span>
            </div>
        );
    }

    return (
        <div className="flex h-10 w-full items-center justify-center gap-2 rounded-md border border-border bg-transparent text-foreground transition-colors hover:bg-secondary">
            <User className="h-4 w-4" />
            <span className="font-medium text-sm">Login</span>
        </div>
    );
}

function LoginContent({ onSuccess }: { onSuccess?: () => void }) {
    const { login } = useAuth();
    const [email, setEmail] = useState("");
    const [otp, setOTP] = useState("");
    const [server, setServer] = useState<AKServer>("en");
    const [isOtpSent, setIsOtpSent] = useState(false);
    const [isSendingOtp, setIsSendingOtp] = useState(false);
    const [isLoggingIn, setIsLoggingIn] = useState(false);
    const [cooldown, setCooldown] = useState(0);
    const cooldownRef = useRef<NodeJS.Timeout | null>(null);

    async function sendOTP() {
        if (email.length === 0) {
            toast.error("Email cannot be blank.", {
                description: "Please enter a valid email address.",
            });
            return;
        }

        setIsSendingOtp(true);
        try {
            const response = await fetch("/api/auth/send-code", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({ email, server }),
            });

            const data: SendCodeResponse = await response.json();

            if (!data.success) {
                throw new Error(data.error ?? "Failed to send OTP");
            }

            toast.success("OTP sent!", {
                description: "Check your email for the 6-digit code.",
            });

            setIsOtpSent(true);
            setCooldown(60);

            // Clear any existing interval
            if (cooldownRef.current) {
                clearInterval(cooldownRef.current);
            }

            cooldownRef.current = setInterval(() => {
                setCooldown((prev) => {
                    if (prev <= 1) {
                        if (cooldownRef.current) {
                            clearInterval(cooldownRef.current);
                            cooldownRef.current = null;
                        }
                        return 0;
                    }
                    return prev - 1;
                });
            }, 1000);
        } catch (err) {
            console.error(err);
            toast.error("Failed to send OTP", {
                description: err instanceof Error ? err.message : "Please check the address you entered and try again.",
            });
        } finally {
            setIsSendingOtp(false);
        }
    }

    async function handleLogin() {
        if (otp.length !== 6) {
            toast.error("Invalid OTP", {
                description: "Please enter the 6-digit code from your email.",
            });
            return;
        }

        setIsLoggingIn(true);
        try {
            const result: LoginResponse = await login(email, otp, server);

            if (!result.success) {
                throw new Error(result.error ?? "Login failed");
            }

            toast.success("Login successful!", {
                description: "Welcome back, Doctor.",
            });

            // Clear the cooldown interval
            if (cooldownRef.current) {
                clearInterval(cooldownRef.current);
                cooldownRef.current = null;
            }

            // Close the dialog
            onSuccess?.();

            window.location.reload();
        } catch (err) {
            console.error(err);
            toast.error("Login failed", {
                description: err instanceof Error ? err.message : "Invalid code or session expired. Please try again.",
            });
        } finally {
            setIsLoggingIn(false);
        }
    }

    const handleSendOtpSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        sendOTP();
    };

    const handleLoginSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        handleLogin();
    };

    const isLoading = isSendingOtp || isLoggingIn;

    return (
        <Card className="w-full max-w-md">
            <CardHeader>
                <CardTitle>Login</CardTitle>
                <CardDescription>
                    Use your YoStar email to send an OTP code. No login information is stored on the server.
                    <Accordion className="mt-3 flex w-full flex-col divide-y divide-zinc-700 rounded-md border border-border px-2" transition={{ duration: 0.2, ease: "easeInOut" }}>
                        <AccordionItem className="py-2" value="how-it-works">
                            <AccordionTrigger className="w-full text-left text-zinc-50">
                                <div className="flex items-center justify-between">
                                    <div>How it Works</div>
                                    <ChevronDown className="group-data-expanded:-rotate-180 h-4 w-4 text-zinc-50 transition-transform duration-200" />
                                </div>
                            </AccordionTrigger>
                            <AccordionContent className="pt-4 text-muted-foreground text-sm leading-relaxed">
                                <div className="max-h-64 overflow-y-auto pr-2">
                                    <p className="mb-3">Myrtle follows the same authentication steps used by the game client when you sign in.</p>
                                    <ol className="mb-3 list-decimal space-y-2 pl-4">
                                        <li>You enter your email, which is sent to our servers. They request the game servers to email you a 6-digit verification code.</li>
                                        <li>After you submit the code on this page, our servers send your email and code to the game servers to obtain an access token.</li>
                                        <li>That access token is then immediately used to request your account information from the game servers.</li>
                                        <li>
                                            The retrieved account data, along with the access token, is sent back to the client (your browser) for processing.
                                            <ul className="mt-1.5 list-disc space-y-1 pl-4 text-muted-foreground/80">
                                                <li>If you choose to do so, the access token is securely saved in your browser's storage; otherwise, it is discarded.</li>
                                                <li>At the same time, the remaining selected data is converted into the site's internal format and uploaded to the database.</li>
                                            </ul>
                                        </li>
                                    </ol>
                                    <div className="rounded-md border border-amber-500/20 bg-amber-500/5 p-3">
                                        <p className="mb-1.5 font-medium text-amber-500/90">Notice</p>
                                        <p className="text-muted-foreground/90">
                                            myrtle.moe is not affiliated with Yostar or Hypergryph and is not an officially sanctioned tool. We are not responsible for any actions taken by the Arknights publishers as a result of logging in using this approach. Proceed at your own discretion.
                                        </p>
                                        <p className="mt-2 text-muted-foreground/80">To date, no such actions have occurred, and we therefore believe it is reasonable to make this tool available to users. Ultimately, the choice to use it is yours.</p>
                                    </div>
                                </div>
                            </AccordionContent>
                        </AccordionItem>
                    </Accordion>
                </CardDescription>
            </CardHeader>
            <CardContent>
                {!isOtpSent ? (
                    <form onSubmit={handleSendOtpSubmit}>
                        <div className="flex flex-col gap-4">
                            <div className="grid gap-2">
                                <Label htmlFor="email">Email</Label>
                                <Input disabled={isLoading} id="email" onChange={(e) => setEmail(e.target.value)} placeholder="doctor@rhodes.island" required type="email" value={email} />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="server">Server</Label>
                                <Select disabled={isLoading} onValueChange={(value) => setServer(value as AKServer)} value={server}>
                                    <SelectTrigger id="server">
                                        <SelectValue placeholder="Select server" />
                                    </SelectTrigger>
                                    <SelectContent className="z-70">
                                        {SERVER_OPTIONS.map((option) => (
                                            <SelectItem key={option.value} value={option.value}>
                                                {option.label}
                                            </SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>
                    </form>
                ) : (
                    <form onSubmit={handleLoginSubmit}>
                        <div className="flex flex-col gap-4">
                            <div className="grid gap-2">
                                <Label>Email</Label>
                                <p className="text-muted-foreground text-sm">{email}</p>
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="otp">Verification Code</Label>
                                <InputOTP disabled={isLoading} maxLength={6} onChange={setOTP} value={otp}>
                                    <InputOTPGroup className="w-full">
                                        <InputOTPSlot className="flex-1" index={0} />
                                        <InputOTPSlot className="flex-1" index={1} />
                                        <InputOTPSlot className="flex-1" index={2} />
                                        <InputOTPSlot className="flex-1" index={3} />
                                        <InputOTPSlot className="flex-1" index={4} />
                                        <InputOTPSlot className="flex-1" index={5} />
                                    </InputOTPGroup>
                                </InputOTP>
                                <p className="text-muted-foreground text-xs">Enter the 6-digit code sent to your email.</p>
                            </div>
                        </div>
                    </form>
                )}
            </CardContent>
            <CardFooter className="flex flex-col gap-2">
                {!isOtpSent ? (
                    <Button className="w-full" disabled={isLoading || email.length === 0} onClick={sendOTP} type="submit" variant="outline">
                        {isSendingOtp ? (
                            <>
                                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                Sending...
                            </>
                        ) : (
                            "Send OTP"
                        )}
                    </Button>
                ) : (
                    <>
                        <Button className="w-full" disabled={isLoading || otp.length !== 6} onClick={handleLogin} type="submit">
                            {isLoggingIn ? (
                                <>
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                    Logging in...
                                </>
                            ) : (
                                "Login"
                            )}
                        </Button>
                        <Button
                            className="w-full text-muted-foreground"
                            disabled={isLoading}
                            onClick={() => {
                                setIsOtpSent(false);
                                setOTP("");
                            }}
                            type="button"
                            variant="ghost"
                        >
                            Use different email
                        </Button>
                        {cooldown > 0 ? (
                            <p className="text-center text-muted-foreground text-xs">Resend available in {cooldown}s</p>
                        ) : (
                            <Button className="h-auto p-0 text-muted-foreground text-xs" disabled={isSendingOtp} onClick={sendOTP} type="button" variant="link">
                                Resend OTP
                            </Button>
                        )}
                    </>
                )}
            </CardFooter>
        </Card>
    );
}

export function Login({ variant = "default" }: LoginProps) {
    const [open, setOpen] = useState(false);

    return (
        <MorphingDialog
            onOpenChange={setOpen}
            open={open}
            transition={{
                type: "spring",
                bounce: 0.1,
                duration: 0.4,
            }}
        >
            <MorphingDialogTrigger className={cn("inline-flex", variant === "default" && "w-full")}>
                <LoginTriggerButton variant={variant} />
            </MorphingDialogTrigger>
            <MorphingDialogContainer>
                <MorphingDialogContent className="relative w-[calc(100vw-2rem)] max-w-md rounded-xl shadow-2xl backdrop-blur-sm">
                    <MorphingDialogClose className="absolute top-3 right-3 z-10 flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground">
                        <X className="h-4 w-4" />
                    </MorphingDialogClose>
                    <LoginContent onSuccess={() => setOpen(false)} />
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
    );
}
