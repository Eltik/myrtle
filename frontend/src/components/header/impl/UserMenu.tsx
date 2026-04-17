import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle, DialogTrigger } from "#/components/ui/dialog";
import { Field, FieldLabel } from "#/components/ui/field";
import { Form } from "#/components/ui/form";
import { Input } from "#/components/ui/input";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { OTPField, OTPFieldInput } from "#/components/ui/otp-field";
import { Select, SelectItem, SelectLabel, SelectPopup, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Spinner } from "#/components/ui/spinner";
import { toastManager } from "#/components/ui/toast";
import type { AKServer } from "#/lib/auth/login";
import { sendCodeFn } from "#/lib/auth/server";
import { getAvatarSkinId } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { useMutation } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ChevronDown, Cog, LayoutList, LogOut } from "lucide-react";
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

export default function UserMenu({ user, loading, login, logout }: { user: IUserProfile | null; loading: boolean; login: (data: { email: string; code: string; server: "en" | "jp" | "kr" | "cn" | "bili" | "tw" }) => Promise<IUserProfile>; logout: () => Promise<void> }) {
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
            // close dialog / reset form as needed
        },
        onError: (err) =>
            toastManager.add({
                id: "login-error",
                title: "Login failed",
                description: err.message,
                type: "error",
            }),
    });

    if (loading) {
        return <Spinner />;
    }

    if (user !== null) {
        return (
            <DropdownMenu>
                <DropdownMenuTrigger>
                    <Button className="flex h-8 items-center gap-2 rounded-md border border-border bg-transparent px-2 text-foreground text-sm transition-colors hover:bg-secondary" variant="ghost">
                        <Avatar className="h-5 w-5">
                            <AvatarImage alt="User avatar" src={getAvatarSkinId(user)} />
                            <AvatarFallback className="text-[0.625rem]">{(user.nickname ?? "Doctor").slice(0, 1) ?? "E"}</AvatarFallback>
                        </Avatar>
                        <span className="max-w-24 truncate font-medium">{user.nickname ?? "Doctor"}</span>
                        <ChevronDown className="h-3 w-3" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48">
                    <div className="px-2 pb-1.5">
                        <Link className="font-medium text-sm hover:underline" to={`/user/${user.uid}`}>
                            {user.nickname ?? "Doctor"}
                        </Link>
                        <p className="text-muted-foreground text-xs">Level {user.level}</p>
                    </div>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem className="cursor-pointer">
                        <Link to="/my/tier-lists" className="flex flex-row gap-2 items-center">
                            <LayoutList className="h-4 w-4 text-muted-foreground" />
                            My Tier Lists
                        </Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem className="cursor-pointer">
                        <Link to="/my/settings" className="flex flex-row gap-2 items-center">
                            <Cog className="h-4 w-4 text-muted-foreground" />
                            Settings
                        </Link>
                    </DropdownMenuItem>
                    <DropdownMenuItem className="text-primary transition-colors focus:text-primary/80 pl-3 cursor-pointer" onClick={logout}>
                        <LogOut className="mr-2 h-4 w-4" />
                        Logout
                    </DropdownMenuItem>
                </DropdownMenuContent>
            </DropdownMenu>
        );
    }

    return (
        <Dialog>
            <DialogTrigger render={<Button />}>Login</DialogTrigger>
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
                            <Input placeholder="doctor@rhodes.island" type="email" value={email} onChange={(e) => setEmail(e.target.value)} />
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
                                    {cooldown > 0 ? (
                                        <p className="text-muted-foreground text-xs">Resend available in {cooldown}s</p>
                                    ) : (
                                        <Button type="button" variant="ghost" disabled={sendOTP.isPending} onClick={() => sendOTP.mutate({ email, server })}>
                                            Resend code
                                        </Button>
                                    )}
                                </>
                            )}
                        </div>
                    </DialogFooter>
                </Form>
            </DialogPopup>
        </Dialog>
    );
}
