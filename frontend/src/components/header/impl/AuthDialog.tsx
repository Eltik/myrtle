import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { Field, FieldLabel } from "#/components/ui/field";
import { Form } from "#/components/ui/form";
import { Input } from "#/components/ui/input";
import { OTPField, OTPFieldInput } from "#/components/ui/otp-field";
import { Select, SelectItem, SelectPopup, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import { toastManager } from "#/components/ui/toast";
import type { AKServer } from "#/lib/auth/login";
import { sendCodeFn } from "#/lib/auth/server";
import { useAuth } from "#/hooks/use-auth";
import { useMutation } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";

export const SERVER_OPTIONS: { value: AKServer; label: string; disabled: boolean }[] = [
    { value: "en", label: "Global (EN)", disabled: false },
    { value: "jp", label: "Japan (JP)", disabled: false },
    { value: "kr", label: "Korea (KR)", disabled: false },
    { value: "cn", label: "China (CN)", disabled: true },
    { value: "bili", label: "Bilibili", disabled: true },
    { value: "tw", label: "Taiwan (TW)", disabled: true },
];

const OTP_LENGTH = 6;
const OTP_SLOT_KEYS = Array.from({ length: OTP_LENGTH }, (_, i) => `otp-slot-${i}`);

interface AuthDialogProps {
    trigger?: React.ReactNode;
    onOpenChange?: (open: boolean) => void;
    open?: boolean;
}

export function AuthDialog({ trigger, onOpenChange, open: openProp }: AuthDialogProps) {
    const { login } = useAuth();
    const [internalOpen, setInternalOpen] = useState(false);
    const isOpen = openProp !== undefined ? openProp : internalOpen;
    const setOpen = onOpenChange !== undefined ? onOpenChange : setInternalOpen;

    const [email, setEmail] = useState("");
    const [otp, setOTP] = useState("");
    const [server, setServer] = useState<AKServer>("en");
    const [isOTPSent, setIsOTPSent] = useState(false);
    const [cooldown, setCooldown] = useState(0);
    const cooldownRef = useRef<NodeJS.Timeout | null>(null);

    useEffect(() => {
        return () => {
            if (cooldownRef.current) clearInterval(cooldownRef.current);
        };
    }, []);

    const resetForm = () => {
        setIsOTPSent(false);
        setOTP("");
        if (cooldownRef.current) {
            clearInterval(cooldownRef.current);
            cooldownRef.current = null;
        }
        setCooldown(0);
    };

    const handleOpenChange = (open: boolean) => {
        setOpen(open);
        if (!open) {
            // Optional: reset form on close if desired,
            // but usually better to keep it if they accidentally closed it
        }
    };

    const sendOTP = useMutation({
        mutationFn: (vars: { email: string; server: AKServer }) => sendCodeFn({ data: vars }),
        onSuccess: () => {
            setIsOTPSent(true);
            setCooldown(60);

            toastManager.add({
                id: "otp-success",
                title: "Sent code",
                description: "Sent OTP code to your email.",
                type: "success",
            });

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
        },
        onError: (err) =>
            toastManager.add({
                id: "otp-error",
                title: "Error",
                description: `There was an error sending an OTP:\n${err.message}`,
                type: "error",
            }),
    });

    const loginMut = useMutation({
        mutationFn: (vars: { email: string; code: string; server: AKServer }) => login(vars),
        onSuccess: () => {
            if (cooldownRef.current) {
                clearInterval(cooldownRef.current);
                cooldownRef.current = null;
            }
            toastManager.add({
                id: "login-success",
                title: "Logged in successfully.",
                type: "success",
            });
            handleOpenChange(false);
            resetForm();
        },
        onError: (err) =>
            toastManager.add({
                id: "login-error",
                title: "Login failed",
                description: err.message,
                type: "error",
            }),
    });

    return (
        <Dialog open={isOpen} onOpenChange={handleOpenChange}>
            {trigger && <DialogTrigger render={trigger} />}
            <DialogPopup className="sm:max-w-sm">
                <Form
                    className="contents"
                    onSubmit={(e) => {
                        e.preventDefault();
                        if (isOTPSent) {
                            loginMut.mutate({ email, code: otp, server });
                        } else {
                            sendOTP.mutate({ email, server });
                        }
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>Login</DialogTitle>
                        <DialogDescription>Use your YoStar email to send an OTP code. No login information is stored on the server.</DialogDescription>
                    </DialogHeader>
                    <DialogPanel className="grid gap-4">
                        <Field>
                            <FieldLabel>Email</FieldLabel>
                            <Input placeholder="doctor@rhodes.island" type="email" value={email} onChange={(e) => setEmail(e.target.value)} disabled={isOTPSent || sendOTP.isPending} />
                        </Field>
                        {isOTPSent ? (
                            <Field>
                                <FieldLabel>Code</FieldLabel>
                                <OTPField aria-label="One-time password" length={OTP_LENGTH} onValueChange={setOTP} size="lg">
                                    {OTP_SLOT_KEYS.map((slotKey, index) => (
                                        <OTPFieldInput key={slotKey} aria-label={`Character ${index + 1} of ${OTP_LENGTH}`} />
                                    ))}
                                </OTPField>
                            </Field>
                        ) : null}
                        <Field>
                            <Select
                                items={SERVER_OPTIONS}
                                aria-label="Select server"
                                defaultValue="en"
                                value={server}
                                onValueChange={(v) => {
                                    if (v) setServer(v as AKServer);
                                }}
                                disabled={isOTPSent || sendOTP.isPending}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectPopup>
                                    {SERVER_OPTIONS.map((item) => (
                                        <SelectItem key={item.value} value={item} disabled={item.disabled}>
                                            {item.label}
                                        </SelectItem>
                                    ))}
                                </SelectPopup>
                            </Select>
                        </Field>
                    </DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button variant="ghost" />}>Cancel</DialogClose>
                        <div className="flex flex-col gap-2">
                            {!isOTPSent ? (
                                <Button type="submit" variant="outline" disabled={sendOTP.isPending || loginMut.isPending || email.length === 0}>
                                    {sendOTP.isPending || loginMut.isPending ? (
                                        <>
                                            <Spinner className="mr-2 h-4 w-4" />
                                            Sending...
                                        </>
                                    ) : (
                                        "Send Code"
                                    )}
                                </Button>
                            ) : (
                                <>
                                    <Button type="submit" variant="outline" disabled={loginMut.isPending || otp.length !== 6}>
                                        {loginMut.isPending ? (
                                            <>
                                                <Spinner className="mr-2 h-4 w-4" />
                                                Logging in...
                                            </>
                                        ) : (
                                            "Login"
                                        )}
                                    </Button>
                                    <div className="flex items-center gap-2">
                                        {cooldown > 0 ? (
                                            <p className="text-muted-foreground text-xs">Resend available in {cooldown}s</p>
                                        ) : (
                                            <Button type="button" variant="ghost" className="h-auto p-0 text-xs" disabled={sendOTP.isPending} onClick={() => sendOTP.mutate({ email, server })}>
                                                Resend code
                                            </Button>
                                        )}
                                        <Button type="button" variant="ghost" className="h-auto p-0 text-xs" onClick={resetForm}>
                                            Change email
                                        </Button>
                                    </div>
                                </>
                            )}
                        </div>
                    </DialogFooter>
                </Form>
            </DialogPopup>
        </Dialog>
    );
}
