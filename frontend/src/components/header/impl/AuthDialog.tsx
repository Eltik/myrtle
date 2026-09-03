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
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type AKServer, formatServerForPicker, SERVERS } from "#/lib/auth/login";
import { sendBiliSmsFn, sendCodeCnFn, sendCodeFn } from "#/lib/auth/server";
import { authActions, authStore } from "#/lib/auth/store";

export const SERVER_OPTIONS: { value: AKServer; label: string; disabled: boolean }[] = SERVERS.map((s) => ({
    value: s.code,
    label: formatServerForPicker(s.code),
    disabled: s.loginDisabled,
}));

const OTP_LENGTH = 6;
const OTP_SLOT_KEYS = Array.from({ length: OTP_LENGTH }, (_, i) => `otp-slot-${i}`);

const YOSTAR_SERVERS: readonly AKServer[] = ["en", "jp", "kr"];

interface IAuthDialogProps {
    trigger?: DialogTriggerProps["render"];
    onOpenChange?: (open: boolean) => void;
    open?: boolean;
}

export function AuthDialog({ trigger, onOpenChange, open: openProp }: IAuthDialogProps) {
    const { login, loginBilibili, loginBilibiliSms, loginCn } = useAuth();
    const [internalOpen, setInternalOpen] = useState(false);
    const isOpen = openProp !== undefined ? openProp : internalOpen;
    const setOpen = onOpenChange !== undefined ? onOpenChange : setInternalOpen;

    const email = useStore(authStore, (s) => s.login.email);
    const otp = useStore(authStore, (s) => s.login.otp);
    const server = useStore(authStore, (s) => s.login.server);
    const isOTPSent = useStore(authStore, (s) => s.login.isOTPSent);
    const cooldownUntil = useStore(authStore, (s) => s.login.cooldownUntil);

    const biliUsername = useStore(authStore, (s) => s.login.biliUsername);
    const biliPassword = useStore(authStore, (s) => s.login.biliPassword);
    const biliPhone = useStore(authStore, (s) => s.login.biliPhone);
    const biliSmsCode = useStore(authStore, (s) => s.login.biliSmsCode);
    const biliUseSms = useStore(authStore, (s) => s.login.biliUseSms);
    const isBiliCodeSent = useStore(authStore, (s) => s.login.isBiliCodeSent);
    const biliCooldownUntil = useStore(authStore, (s) => s.login.biliCooldownUntil);

    const cnPhone = useStore(authStore, (s) => s.login.cnPhone);
    const cnPassword = useStore(authStore, (s) => s.login.cnPassword);
    const cnSmsCode = useStore(authStore, (s) => s.login.cnSmsCode);
    const cnUseSms = useStore(authStore, (s) => s.login.cnUseSms);
    const isCnCodeSent = useStore(authStore, (s) => s.login.isCnCodeSent);
    const cnCooldownUntil = useStore(authStore, (s) => s.login.cnCooldownUntil);

    const isYostar = YOSTAR_SERVERS.includes(server);
    const isBili = server === "bili";
    const isCn = server === "cn";

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

    const [cnCooldown, setCnCooldown] = useState(() => Math.max(0, Math.ceil((cnCooldownUntil - Date.now()) / 1000)));
    useEffect(() => {
        if (cnCooldownUntil <= Date.now()) {
            setCnCooldown(0);
            return;
        }
        const tick = () => setCnCooldown(Math.max(0, Math.ceil((cnCooldownUntil - Date.now()) / 1000)));
        tick();
        const id = setInterval(tick, 1000);
        return () => clearInterval(id);
    }, [cnCooldownUntil]);

    const [biliCooldown, setBiliCooldown] = useState(() => Math.max(0, Math.ceil((biliCooldownUntil - Date.now()) / 1000)));
    useEffect(() => {
        if (biliCooldownUntil <= Date.now()) {
            setBiliCooldown(0);
            return;
        }
        const tick = () => setBiliCooldown(Math.max(0, Math.ceil((biliCooldownUntil - Date.now()) / 1000)));
        tick();
        const id = setInterval(tick, 1000);
        return () => clearInterval(id);
    }, [biliCooldownUntil]);

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

    const sendCnSms = useMutation({
        mutationFn: (vars: { phone: string }) => sendCodeCnFn({ data: vars }),
        onSuccess: () => {
            authActions.markCnCodeSent(60);
            toastManager.add({
                id: "cn-otp-success",
                title: "Sent code",
                description: "Sent SMS code to your phone.",
                type: "success",
            });
        },
        onError: (err) =>
            toastManager.add({
                id: "cn-otp-error",
                title: "Error",
                description: `There was an error sending a code:\n${err.message}`,
                type: "error",
            }),
    });

    const sendBiliSms = useMutation({
        mutationFn: (vars: { phone: string }) => sendBiliSmsFn({ data: vars }),
        onSuccess: () => {
            authActions.markBiliCodeSent(60);
            toastManager.add({
                id: "bili-otp-success",
                title: "Sent code",
                description: "Sent SMS code to your phone.",
                type: "success",
            });
        },
        onError: (err) =>
            toastManager.add({
                id: "bili-otp-error",
                title: "Error",
                description: `There was an error sending a code:\n${err.message}`,
                type: "error",
            }),
    });

    const onLoginSuccess = () => {
        toastManager.add({
            id: "login-success",
            title: "Logged in successfully.",
            type: "success",
        });
        setOpen(false);
        authActions.resetLoginForm();
    };
    const onLoginError = (err: Error) =>
        toastManager.add({
            id: "login-error",
            title: "Login failed",
            description: err.message,
            type: "error",
        });

    const loginMut = useMutation({
        mutationFn: (vars: { email: string; code: string; server: AKServer }) => login(vars),
        onSuccess: onLoginSuccess,
        onError: onLoginError,
    });

    const loginBiliMut = useMutation({
        mutationFn: (vars: { username: string; password: string }) => loginBilibili(vars),
        onSuccess: onLoginSuccess,
        onError: onLoginError,
    });

    const loginBiliSmsMut = useMutation({
        mutationFn: (vars: { phone: string; code: string }) => loginBilibiliSms(vars),
        onSuccess: onLoginSuccess,
        onError: onLoginError,
    });

    const loginCnMut = useMutation({
        mutationFn: (vars: { phone: string; password?: string; code?: string }) => loginCn(vars),
        onSuccess: onLoginSuccess,
        onError: onLoginError,
    });

    let submitLabel: string;
    let submitPending: boolean;
    let submitDisabled: boolean;

    if (isBili) {
        if (biliUseSms && !isBiliCodeSent) {
            submitPending = sendBiliSms.isPending;
            submitLabel = submitPending ? "Sending..." : "Send Code";
            submitDisabled = submitPending || biliPhone.length === 0;
        } else if (biliUseSms) {
            submitPending = loginBiliSmsMut.isPending;
            submitLabel = submitPending ? "Logging in..." : "Login";
            submitDisabled = submitPending || biliPhone.length === 0 || biliSmsCode.length === 0;
        } else {
            submitPending = loginBiliMut.isPending;
            submitLabel = submitPending ? "Logging in..." : "Login";
            submitDisabled = submitPending || biliUsername.length === 0 || biliPassword.length === 0;
        }
    } else if (isCn) {
        if (cnUseSms && !isCnCodeSent) {
            submitPending = sendCnSms.isPending;
            submitLabel = submitPending ? "Sending..." : "Send Code";
            submitDisabled = submitPending || cnPhone.length === 0;
        } else {
            submitPending = loginCnMut.isPending;
            submitLabel = submitPending ? "Logging in..." : "Login";
            submitDisabled = submitPending || cnPhone.length === 0 || (cnUseSms ? cnSmsCode.length === 0 : cnPassword.length === 0);
        }
    } else if (isOTPSent) {
        submitPending = loginMut.isPending;
        submitLabel = submitPending ? "Logging in..." : "Login";
        submitDisabled = submitPending || otp.length !== 6;
    } else {
        submitPending = sendOTP.isPending || loginMut.isPending;
        submitLabel = submitPending ? "Sending..." : "Send Code";
        submitDisabled = submitPending || email.length === 0;
    }

    const description = isBili
        ? "Log in with your Bilibili account. No login information is stored on the server."
        : isCn
          ? "Experimental: logs in with your Hypergryph (CN) account through the same passport flow the Skland app uses. Untested against a real account, so it may fail even with correct credentials. No login information is stored on the server."
          : "Use your YoStar email to send an OTP code. No login information is stored on the server.";

    return (
        <Dialog open={isOpen} onOpenChange={setOpen}>
            {trigger && <DialogTrigger render={trigger} />}
            <DialogPopup className="sm:max-w-sm">
                <Form
                    className="contents"
                    onSubmit={(e) => {
                        e.preventDefault();
                        if (isBili) {
                            if (biliUseSms) {
                                if (isBiliCodeSent) {
                                    loginBiliSmsMut.mutate({ phone: biliPhone, code: biliSmsCode });
                                } else {
                                    sendBiliSms.mutate({ phone: biliPhone });
                                }
                            } else {
                                loginBiliMut.mutate({ username: biliUsername, password: biliPassword });
                            }
                            return;
                        }
                        if (isCn) {
                            if (cnUseSms) {
                                if (isCnCodeSent) {
                                    loginCnMut.mutate({ phone: cnPhone, code: cnSmsCode });
                                } else {
                                    sendCnSms.mutate({ phone: cnPhone });
                                }
                            } else {
                                loginCnMut.mutate({ phone: cnPhone, password: cnPassword });
                            }
                            return;
                        }
                        if (isOTPSent) {
                            loginMut.mutate({ email, code: otp, server });
                        } else {
                            sendOTP.mutate({ email, server });
                        }
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>Login</DialogTitle>
                        <DialogDescription>{description}</DialogDescription>
                    </DialogHeader>
                    <DialogPanel className="grid gap-4">
                        {isYostar ? (
                            <>
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
                            </>
                        ) : null}
                        {isBili ? (
                            <Tabs
                                value={biliUseSms ? "sms" : "password"}
                                onValueChange={(v) => {
                                    authActions.setBiliUseSms(v === "sms");
                                    authActions.resetBiliCode();
                                }}
                            >
                                <TabsList>
                                    <TabsTab value="password">Password</TabsTab>
                                    <TabsTab value="sms">SMS code</TabsTab>
                                </TabsList>
                                <TabsPanel value="password">
                                    <Field>
                                        <FieldLabel>Username</FieldLabel>
                                        <Input placeholder="Bilibili username, phone, or email" value={biliUsername} onChange={(e) => authActions.setBiliUsername(e.target.value)} disabled={loginBiliMut.isPending} />
                                    </Field>
                                    <Field>
                                        <FieldLabel>Password</FieldLabel>
                                        <Input type="password" value={biliPassword} onChange={(e) => authActions.setBiliPassword(e.target.value)} disabled={loginBiliMut.isPending} />
                                    </Field>
                                </TabsPanel>
                                <TabsPanel value="sms">
                                    <p className="text-muted-foreground text-xs">Experimental: this endpoint hasn't been confirmed against a real response, so it may fail even with a correct code.</p>
                                    <Field>
                                        <FieldLabel>Phone</FieldLabel>
                                        <Input placeholder="Bilibili account phone number" value={biliPhone} onChange={(e) => authActions.setBiliPhone(e.target.value)} disabled={loginBiliSmsMut.isPending || isBiliCodeSent} />
                                    </Field>
                                    {isBiliCodeSent ? (
                                        <Field>
                                            <FieldLabel>Code</FieldLabel>
                                            <Input value={biliSmsCode} onChange={(e) => authActions.setBiliSmsCode(e.target.value)} disabled={loginBiliSmsMut.isPending} />
                                        </Field>
                                    ) : null}
                                </TabsPanel>
                            </Tabs>
                        ) : null}
                        {isCn ? (
                            <>
                                <Field>
                                    <FieldLabel>Phone</FieldLabel>
                                    <Input placeholder="Hypergryph account phone number" value={cnPhone} onChange={(e) => authActions.setCnPhone(e.target.value)} disabled={loginCnMut.isPending || (cnUseSms && isCnCodeSent)} />
                                </Field>
                                <Tabs
                                    value={cnUseSms ? "sms" : "password"}
                                    onValueChange={(v) => {
                                        authActions.setCnUseSms(v === "sms");
                                        authActions.resetCnCode();
                                    }}
                                >
                                    <TabsList>
                                        <TabsTab value="password">Password</TabsTab>
                                        <TabsTab value="sms">SMS code</TabsTab>
                                    </TabsList>
                                    <TabsPanel value="password">
                                        <Field>
                                            <FieldLabel>Password</FieldLabel>
                                            <Input type="password" value={cnPassword} onChange={(e) => authActions.setCnPassword(e.target.value)} disabled={loginCnMut.isPending} />
                                        </Field>
                                    </TabsPanel>
                                    <TabsPanel value="sms">
                                        {isCnCodeSent ? (
                                            <Field>
                                                <FieldLabel>Code</FieldLabel>
                                                <Input value={cnSmsCode} onChange={(e) => authActions.setCnSmsCode(e.target.value)} disabled={loginCnMut.isPending} />
                                            </Field>
                                        ) : null}
                                    </TabsPanel>
                                </Tabs>
                            </>
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
                                disabled={isOTPSent || sendOTP.isPending || (isCn && isCnCodeSent) || (isBili && biliUseSms && isBiliCodeSent)}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectPopup>
                                    {SERVER_OPTIONS.map((item) => (
                                        <SelectItem key={item.value} value={item.value} disabled={item.disabled}>
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
                            <Button type="submit" variant="outline" disabled={submitDisabled}>
                                {submitPending ? (
                                    <>
                                        <Spinner className="mr-2 h-4 w-4" />
                                        {submitLabel}
                                    </>
                                ) : (
                                    submitLabel
                                )}
                            </Button>
                            {isYostar && isOTPSent ? (
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
                            ) : null}
                            {isCn && cnUseSms && isCnCodeSent ? (
                                <div className="flex items-center gap-2">
                                    {cnCooldown > 0 ? (
                                        <p className="text-muted-foreground text-xs">Resend available in {cnCooldown}s</p>
                                    ) : (
                                        <Button type="button" variant="ghost" className="h-auto p-0 text-xs" disabled={sendCnSms.isPending} onClick={() => sendCnSms.mutate({ phone: cnPhone })}>
                                            Resend code
                                        </Button>
                                    )}
                                    <Button type="button" variant="ghost" className="h-auto p-0 text-xs" onClick={authActions.resetCnCode}>
                                        Change phone
                                    </Button>
                                </div>
                            ) : null}
                            {isBili && biliUseSms && isBiliCodeSent ? (
                                <div className="flex items-center gap-2">
                                    {biliCooldown > 0 ? (
                                        <p className="text-muted-foreground text-xs">Resend available in {biliCooldown}s</p>
                                    ) : (
                                        <Button type="button" variant="ghost" className="h-auto p-0 text-xs" disabled={sendBiliSms.isPending} onClick={() => sendBiliSms.mutate({ phone: biliPhone })}>
                                            Resend code
                                        </Button>
                                    )}
                                    <Button type="button" variant="ghost" className="h-auto p-0 text-xs" onClick={authActions.resetBiliCode}>
                                        Change phone
                                    </Button>
                                </div>
                            ) : null}
                        </div>
                    </DialogFooter>
                </Form>
            </DialogPopup>
        </Dialog>
    );
}
