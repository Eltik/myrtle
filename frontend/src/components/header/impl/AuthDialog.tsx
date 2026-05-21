import type { DialogTriggerProps } from "@base-ui/react";
import { useMutation } from "@tanstack/react-query";
import { useStore } from "@tanstack/react-store";
import { useEffect, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { Field, FieldLabel } from "#/components/ui/field";
import { Form } from "#/components/ui/form";
import { Input } from "#/components/ui/input";
import { OTPField, OTPFieldInput } from "#/components/ui/otp-field";
import { Select, SelectItem, SelectPopup, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type AKServer, formatServerForPicker, SERVERS } from "#/lib/auth/login";
import { sendCodeFn } from "#/lib/auth/server";
import { authActions, authStore } from "#/lib/auth/store";

export const SERVER_OPTIONS: { value: AKServer; label: string; disabled: boolean }[] = SERVERS.map((s) => ({
    value: s.code,
    label: formatServerForPicker(s.code),
    disabled: s.loginDisabled,
}));

const OTP_LENGTH = 6;
const OTP_SLOT_KEYS = Array.from({ length: OTP_LENGTH }, (_, i) => `otp-slot-${i}`);

interface IAuthDialogProps {
    trigger?: DialogTriggerProps["render"];
    onOpenChange?: (open: boolean) => void;
    open?: boolean;
}

export function AuthDialog({ trigger, onOpenChange, open: openProp }: IAuthDialogProps) {
    const { login } = useAuth();
    const [internalOpen, setInternalOpen] = useState(false);
    const isOpen = openProp !== undefined ? openProp : internalOpen;
    const setOpen = onOpenChange !== undefined ? onOpenChange : setInternalOpen;

    const email = useStore(authStore, (s) => s.login.email);
    const otp = useStore(authStore, (s) => s.login.otp);
    const server = useStore(authStore, (s) => s.login.server);
    const isOTPSent = useStore(authStore, (s) => s.login.isOTPSent);
    const cooldownUntil = useStore(authStore, (s) => s.login.cooldownUntil);

    const [cooldown, setCooldown] = useState(() => Math.max(0, Math.ceil((cooldownUntil - Date.now()) / 1000)));

    useEffect(() => {
        if (cooldownUntil <= Date.now()) {
            setCooldown(0);
            return;
        }
        const tick = () => setCooldown(Math.max(0, Math.ceil((cooldownUntil - Date.now()) / 1000)));
        tick();
        const id = setInterval(tick, 1000);
        return () => clearInterval(id);
    }, [cooldownUntil]);

    const sendOTP = useMutation({
        mutationFn: (vars: { email: string; server: AKServer }) => sendCodeFn({ data: vars }),
        onSuccess: () => {
            authActions.markOTPSent(60);
            toastManager.add({
                id: "otp-success",
                title: "Sent code",
                description: "Sent OTP code to your email.",
                type: "success",
            });
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
            toastManager.add({
                id: "login-success",
                title: "Logged in successfully.",
                type: "success",
            });
            setOpen(false);
            authActions.resetLoginForm();
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
        <Dialog open={isOpen} onOpenChange={setOpen}>
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
                            <Input placeholder="doctor@rhodes.island" type="email" value={email} onChange={(e) => authActions.setLoginEmail(e.target.value)} disabled={isOTPSent || sendOTP.isPending} />
                        </Field>
                        {isOTPSent ? (
                            <Field>
                                <FieldLabel>Code</FieldLabel>
                                <OTPField aria-label="One-time password" length={OTP_LENGTH} value={otp} onValueChange={authActions.setLoginOTP} size="lg">
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
                                    if (v) authActions.setLoginServer(v as AKServer);
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
                                        <Button type="button" variant="ghost" className="h-auto p-0 text-xs" onClick={authActions.resetLoginOTP}>
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
