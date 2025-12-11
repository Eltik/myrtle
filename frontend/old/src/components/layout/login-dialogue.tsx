import { Button } from "../ui/button";
import { DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "../ui/dialog";
import { Label } from "~/components/ui/label";
import { Input } from "../ui/input";
import { useState } from "react";
import { Loader2 } from "lucide-react";
import { useToast } from "~/hooks/use-toast";
import { useCookies } from "react-cookie";
import { usePlayer } from "~/store";
import type { AKServer, StoredUser, User } from "~/types/impl/api";
import type { SendCodeResponse } from "~/types/impl/api/impl/send-code";
import type { LoginResponse } from "~/types/impl/api/impl/login";
import type { RefreshResponse } from "~/types/impl/api/impl/refresh";
import type { PlayerResponse } from "~/types/impl/api/impl/player";

export function LoginDialogue() {
    const [email, setEmail] = useState("");
    const [otp, setOTP] = useState("");
    const [isOtpSent, setIsOtpSent] = useState(false);
    const [isLoading, setIsLoading] = useState(false);
    const [cooldown, setCooldown] = useState(0);
    const { toast } = useToast();

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const [_, setCookie] = useCookies(["login"]);

    const sendOtp = async () => {
        if (email.length === 0) {
            toast({
                title: "Error: Invalid email",
                description: "Please enter a valid email address.",
            });
            return;
        }
        setIsLoading(true);
        try {
            const data = (await (
                await fetch("/api/sendCode", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        email,
                        server: "en" as AKServer,
                    }),
                })
            ).json()) as SendCodeResponse;

            if (!data.success || data.message) {
                throw new Error(data.message);
            }

            setIsOtpSent(true);
            setCooldown(60);
            const interval = setInterval(() => {
                setCooldown((prev) => {
                    if (prev <= 1) {
                        clearInterval(interval);
                        return 0;
                    }
                    return prev - 1;
                });
            }, 1000);
        } catch (err) {
            console.error(err);
            toast({
                title: "Error: Failed to send OTP",
                description: "Please check your email and try again.",
            });
        } finally {
            setIsLoading(false);
        }
    };

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setIsLoading(true);
        try {
            // Simulating an API call to verify OTP
            const login = (await (
                await fetch("/api/login", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        email,
                        code: otp,
                        server: "en" as AKServer,
                    }),
                })
            ).json()) as LoginResponse;

            if (login.error || login.message) {
                console.log(login);
                throw new Error(login.message);
            }

            setCookie("login", login, {
                maxAge: 60 * 60 * 24,
                path: "/",
            });

            console.log(`Successfully logged in as ${login.uid}.`);

            toast({
                title: "Success: Logged in",
                description: "You have successfully logged in. Fetching player data...",
            });

            try {
                const playerData = (await (
                    await fetch("/api/refresh", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            uid: login.uid,
                            secret: login.secret,
                            seqnum: (login.seqnum ?? 0) + 1,
                            server: "en" as AKServer,
                        }),
                    })
                ).json()) as RefreshResponse;

                if (playerData.error || playerData.message) {
                    console.log(playerData);
                    throw new Error(playerData.message);
                }

                const storedData: StoredUser = {
                    avatar: playerData.avatar,
                    status: playerData.status,
                };

                usePlayer.setState({ playerData: storedData });

                setIsLoading(false);

                toast({
                    title: "Success: Fetched player data",
                    description: "You have successfully logged in.",
                });
            } catch (e) {
                console.error(e);
                toast({
                    title: "Sorry! An error occurred. Please wait...",
                    description: "Sorry, it takes a little while to fetch and format player data. Please wait a moment. If this message persists after 10 seconds, please try logging in again in 1 minute.",
                });

                await new Promise((resolve) => setTimeout(resolve, 10000));

                const playerData = (await (
                    await fetch("/api/refresh", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            uid: login.uid,
                            server: "en" as AKServer,
                        }),
                    })
                ).json()) as PlayerResponse;

                if (playerData.error || playerData.message || playerData.length === 0) {
                    console.log(playerData);
                    throw new Error(playerData.message);
                }

                const storedData: StoredUser = {
                    avatar: playerData[0]?.data.avatar ?? {},
                    status: playerData[0]?.data.status as unknown as User["status"],
                };

                usePlayer.setState({ playerData: storedData });

                setIsLoading(false);

                toast({
                    title: "Success: Fetched player data",
                    description: "You have successfully logged in.",
                });
            }
        } catch (err) {
            console.error(err);
            toast({
                title: "Error: Failed to login",
                description: "Please check your email and OTP and try again.",
            });
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
                <DialogTitle>Login</DialogTitle>
                <DialogDescription>
                    Use your YoStar email to send a code to your email. No login information is stored on the server. myrtle.moe keeps you logged in for one day max as a security measure.
                    <br />
                    <br />
                    <b>WARNING:</b> Logging in will log you out of all current Arknights sessions. Make sure you are not logged in on any other devices.
                </DialogDescription>
            </DialogHeader>
            <form onSubmit={handleSubmit}>
                <div className="grid w-full items-center gap-4">
                    <div className="flex flex-col space-y-1.5">
                        <Label htmlFor="email">Email</Label>
                        <Input id="email" value={email} type="email" placeholder="doctor@rhodes.island" onChange={(e) => setEmail(e.target.value)} required />
                    </div>
                    {isOtpSent && (
                        <div className="flex flex-col space-y-1.5">
                            <Label htmlFor="otp">Code</Label>
                            <Input id="otp" type="number" placeholder="Enter OTP..." value={otp} onChange={(e) => setOTP(String(e.target.value))} required />
                        </div>
                    )}
                </div>
                <DialogFooter className="mt-4 flex flex-col items-start space-y-2">
                    <div className="flex flex-row items-center gap-4">
                        <button
                            type="button"
                            className="hover:underline"
                            onClick={() => {
                                setIsOtpSent(false);
                                setCooldown(0);
                                setOTP("");
                            }}
                        >
                            Retry
                        </button>
                        <Button type="submit" className="w-full" onClick={isOtpSent ? handleSubmit : sendOtp} disabled={isLoading || (isLoading && cooldown > 0) || (isOtpSent && otp.length === 0)}>
                            {isLoading ? (
                                <>
                                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                                    Please wait...
                                </>
                            ) : isOtpSent ? (
                                "Login"
                            ) : cooldown > 0 ? (
                                `Resend OTP (${cooldown}s)`
                            ) : (
                                "Send OTP"
                            )}
                        </Button>
                    </div>
                </DialogFooter>
            </form>
        </DialogContent>
    );
}
