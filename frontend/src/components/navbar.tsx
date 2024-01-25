import Link from "next/link";
import React, { useEffect, useState } from "react";
import { useStore } from "zustand";
import ClientOnly from "~/lib/ClientOnly";
import { useLogin, usePlayer } from "~/store/store";
import { type Server, type CodeResponse, type LoginResponse, type PlayerData, type LoginData } from "~/types/types";
import { NavigationMenu, NavigationMenuContent, NavigationMenuItem, NavigationMenuLink, NavigationMenuList, NavigationMenuTrigger, navigationMenuTriggerStyle } from "./ui/navigation-menu";
import { Button } from "./ui/button";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import { cn } from "~/lib/utils";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "./ui/dialog";
import { Input } from "./ui/input";
import { ToastAction } from "./ui/toast";
import { useToast } from "./ui/use-toast";
import { CommandDialog, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList } from "./ui/command";
import { Menu, TrendingUp, UploadCloud, Users } from "lucide-react";
import { ThemeToggle } from "./theme-toggle";
import { DropdownMenu, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuPortal, DropdownMenuSeparator, DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger } from "./ui/dropdown-menu";

function Navbar() {
    const [showNavbar, setShowNavbar] = useState(false);

    const loginData = useStore(useLogin, (state) => (state as { loginData: LoginData })?.loginData);
    const playerData = useStore(usePlayer, (state) => (state as { playerData: PlayerData })?.playerData);

    const [email, setEmail] = useState<string | null>(null);

    const [server, setServer] = useState<Server>("en");

    const [submitted, setSubmitted] = useState(false);
    const [canLogin, setCanLogin] = useState(false);
    const [hasOTP, setHasOTP] = useState(false);

    const [timer, setTimer] = useState(60);

    const { toast } = useToast();

    const [searchModel, setSearchModel] = useState(false);
    useEffect(() => {
        const down = (e: KeyboardEvent) => {
            if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
                e.preventDefault();
                setSearchModel((open) => !open);
            }
        };

        document.addEventListener("keydown", down);
        return () => document.removeEventListener("keydown", down);
    }, []);

    useEffect(() => {
        if (hasOTP || !submitted || timer <= 0) return;

        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");

        let interval: NodeJS.Timeout | null = null;

        const countdown = () => {
            interval = setInterval(() => {
                setTimer(timer - 1);
            }, 1000);
            if (!canLogin) {
                Object.assign(otpButton ?? {}, {
                    innerText: `Resend in ${timer}...`,
                });
            }
            if (timer === 1) {
                setTimer(0);
                otpButton?.removeAttribute("disabled");
                otpButton?.classList.remove("disabled:cursor-not-allowed");
                if (!canLogin) {
                    Object.assign(otpButton ?? {}, {
                        innerText: "Resend",
                    });
                }
            }
        };

        countdown();

        return () => clearInterval(interval!);
    }, [timer, submitted]);

    const sendOTP = async () => {
        if (submitted) return;

        const email: HTMLInputElement | null = document.querySelector("#login-email");
        if (!email) return alert("Email element not found. This is an error. Please contact the developer.");

        setEmail(email.value);

        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
        Object.assign(otpButton ?? {}, {
            innerText: `Sending...`,
        });
        otpButton?.setAttribute("disabled", "true");
        otpButton?.classList.add("disabled:cursor-not-allowed");

        try {
            const data = (await (
                await fetch("/api/code", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        email: email.value,
                        server,
                    }),
                })
            ).json()) as CodeResponse;
            if (data.result === 0) {
                console.log("Successfully sent OTP.", data);
                if (timer === 0) setTimer(60);
                setSubmitted(true);
            } else {
                console.error("Failed to send OTP.", data);
                alert("Failed to send OTP. Please try again later.");
                setSubmitted(false);
                setEmail(null);
                setTimer(0);
                otpButton?.removeAttribute("disabled");
                otpButton?.classList.remove("disabled:cursor-not-allowed");
                Object.assign(otpButton ?? {}, {
                    innerText: "Submit",
                });
            }
        } catch (e) {
            toast({
                title: "Uh oh! There was an error!",
                description: "The server was unable to send an OTP to your email. Please try again later.",
                action: (
                    <ToastAction
                        altText="Goto schedule to undo"
                        onClick={() => {
                            void sendOTP();
                        }}
                    >
                        Try again
                    </ToastAction>
                ),
            });
        }
    };

    const checkOTP = (e: React.KeyboardEvent<HTMLInputElement>) => {
        const input = e.currentTarget.value;
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
        const curEmail: string | null = hasOTP ? (document.querySelector("#login-email")! as HTMLInputElement)?.value : email;

        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
        if (input.length > 0 && curEmail && curEmail.length > 0) {
            setCanLogin(true);
            otpButton?.removeAttribute("disabled");
            otpButton?.classList.remove("disabled:cursor-not-allowed");

            Object.assign(otpButton ?? {}, {
                innerText: "Login",
            });
        } else {
            setCanLogin(false);
            otpButton?.setAttribute("disabled", "true");
            otpButton?.classList.add("disabled:cursor-not-allowed");
        }
    };

    const login = async () => {
        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
        Object.assign(otpButton ?? {}, {
            innerText: `Logging in...`,
        });
        otpButton?.setAttribute("disabled", "true");
        otpButton?.classList.add("disabled:cursor-not-allowed");

        const otp: HTMLInputElement | null = document.querySelector("#otp-code");
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
        const curEmail: string | null = hasOTP ? (document.querySelector("#login-email")! as HTMLInputElement)?.value : email;
        if (otp && otp.value.length > 0 && curEmail && curEmail.length > 0) {
            console.log("Logging in...");
            try {
                const data = (await (
                    await fetch("/api/login", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            email: curEmail,
                            code: otp.value,
                            server,
                        }),
                    })
                ).json()) as LoginResponse;
                Object.assign(data, {
                    email: curEmail,
                });

                useLogin.setState({ loginData: data });

                toast({
                    title: "Logged in",
                    description: `Successfully logged into account ${data.uid}. Fetching player data and refreshing the page...`,
                });

                await fetchPlayerData(data as LoginData);
            } catch (e) {
                toast({
                    title: "Uh oh! There was an error!",
                    description: "The server was unable to login to your account. Please try again later.",
                    action: (
                        <ToastAction
                            altText="Goto schedule to undo"
                            onClick={() => {
                                void login();
                            }}
                        >
                            Try again
                        </ToastAction>
                    ),
                });
            }
        } else {
            toast({
                title: "Error!",
                description: "Your OTP code or email is empty. Please try again.",
            });
        }
    };

    const fetchPlayerData = async (loginData: LoginData) => {
        try {
            const playerData = (await (
                await fetch("/api/player", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        uid: loginData.uid,
                        email: loginData.email,
                        secret: loginData.secret,
                        seqnum: loginData.seqnum++,
                        server,
                    }),
                })
            ).json()) as PlayerData;

            // Inccrease seqnum
            useLogin.setState({ loginData: { ...loginData, seqnum: loginData.seqnum++ } });

            if ((playerData as unknown as { statusCode: number }).statusCode === 401) {
                // Clear login data
                useLogin.setState({ loginData: null });
            } else {
                console.log("Fetched player data successfully.");
                usePlayer.setState({ playerData });
                window.location.reload();
            }
        } catch (e) {
            toast({
                title: "Uh oh! There was an error!",
                description: "The server was unable to fetch player information from your account. Please refresh and try again later.",
            });
        }
    };

    const leaderboardComponents: { title: string; href: string; description: string }[] = [
        {
            title: "Level",
            href: "/leaderboard?type=level&sort=desc",
            description: "View the top Arknights players sorted by their level.",
        },
        {
            title: "Operator Trust",
            href: "/leaderboard?type=trust&sort=desc&operator=amiya",
            description: "Sort Arknights players based on the trust level of a specific operator.",
        },
        {
            title: "Operator Mastery",
            href: "/leaderboard?type=mastery&sort=desc&operator=amiya",
            description: "See what the best players are doing in terms of module level, operator level, and skill masteries.",
        },
        {
            title: "Story Progress",
            href: "/leaderboard?type=story&sort=desc",
            description: "Track your friends and other players, new or veteran, on their story progress.",
        },
        {
            title: "Annihilation",
            href: "/leaderboard?type=annihilation&sort=desc",
            description: "See how well you and other players are doing in the weekly Annihilation mode.",
        },
    ];

    const profileComponents: { title: string; href: string; description: string }[] = [
        {
            title: "Profile",
            href: "/profile",
            description: "Display and share your current operators as well as your materials and resources.",
        },
        {
            title: "Management",
            href: "/manage",
            description: "Account management and connections across various platforms.",
        },
        {
            title: "Settings",
            href: "/settings",
            description: "Change your profile settings and preferences on myrtle.moe.",
        },
        {
            title: "Logout",
            href: "/logout",
            description: "Logout of your current Arknights account.",
        },
    ];

    return (
        <ClientOnly>
            <Dialog>
                <div className="sticky top-0 z-50 w-full border-b-2 border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
                    <NavigationMenu className="container flex h-14 max-w-screen-2xl justify-between">
                        <NavigationMenuList>
                            <NavigationMenuItem className="block md:hidden">
                                <DropdownMenu>
                                    <DropdownMenuTrigger asChild className="cursor-pointer rounded-md transition-all duration-150 ease-in-out">
                                        <Menu className="h-8 w-8" onClick={() => setShowNavbar((show) => !show)} />
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent className="ml-5 w-56">
                                        <DropdownMenuGroup>
                                            <Link href={"/"}>
                                                <DropdownMenuItem className="cursor-pointer">
                                                    <span>Home</span>
                                                </DropdownMenuItem>
                                            </Link>
                                            <Link href={"/planner"}>
                                                <DropdownMenuItem className="cursor-pointer">
                                                    <span>Planner</span>
                                                </DropdownMenuItem>
                                            </Link>
                                            <DropdownMenuSub>
                                                <DropdownMenuSubTrigger className="cursor-pointer">
                                                    <span>Leaderboard</span>
                                                </DropdownMenuSubTrigger>
                                                <DropdownMenuPortal>
                                                    <DropdownMenuSubContent>
                                                        {leaderboardComponents.map((component, index) => (
                                                            <Link href={component.href} key={`leaderboard-component-mobile-${index}`}>
                                                                <DropdownMenuItem className="cursor-pointer">
                                                                    <span>{component.title}</span>
                                                                </DropdownMenuItem>
                                                            </Link>
                                                        ))}
                                                    </DropdownMenuSubContent>
                                                </DropdownMenuPortal>
                                            </DropdownMenuSub>
                                            {playerData.status?.uid ? (
                                                <DropdownMenuSub>
                                                    <DropdownMenuSubTrigger className="cursor-pointer">
                                                        <span>Your Account</span>
                                                    </DropdownMenuSubTrigger>
                                                    <DropdownMenuPortal>
                                                        <DropdownMenuSubContent>
                                                            {profileComponents.map((component, index) => (
                                                                <Link href={component.href} key={`profile-component-mobile-${index}`}>
                                                                    <DropdownMenuItem className="cursor-pointer">
                                                                        <span>{component.title}</span>
                                                                    </DropdownMenuItem>
                                                                </Link>
                                                            ))}
                                                        </DropdownMenuSubContent>
                                                    </DropdownMenuPortal>
                                                </DropdownMenuSub>
                                            ) : (
                                                <DropdownMenuItem
                                                    className="cursor-pointer"
                                                    onClick={() => {
                                                        if (loginData?.uid) {
                                                            console.log("Already have login data...");
                                                            void fetchPlayerData(loginData);
                                                        }
                                                    }}
                                                >
                                                    <DialogTrigger asChild>
                                                        <span>Login</span>
                                                    </DialogTrigger>
                                                </DropdownMenuItem>
                                            )}
                                        </DropdownMenuGroup>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </NavigationMenuItem>
                            <div className={"hidden md:flex"}>
                                <NavigationMenuItem>
                                    <Link href={"/"} legacyBehavior passHref>
                                        <NavigationMenuLink className={navigationMenuTriggerStyle()}>Home</NavigationMenuLink>
                                    </Link>
                                </NavigationMenuItem>
                                <NavigationMenuItem>
                                    <Link href={"/planner"} legacyBehavior passHref>
                                        <NavigationMenuLink className={navigationMenuTriggerStyle()}>Planner</NavigationMenuLink>
                                    </Link>
                                </NavigationMenuItem>
                                <NavigationMenuItem>
                                    <NavigationMenuTrigger>Leaderboard</NavigationMenuTrigger>
                                    <NavigationMenuContent>
                                        <ul className="grid w-[400px] gap-3 p-4 md:w-[500px] md:grid-cols-2 lg:w-[600px]">
                                            {leaderboardComponents.map((component, index) => (
                                                <ListItem key={`leaderboard-component-${index}`} title={component.title} href={component.href}>
                                                    {component.description}
                                                </ListItem>
                                            ))}
                                        </ul>
                                    </NavigationMenuContent>
                                </NavigationMenuItem>
                                {playerData.status?.uid ? (
                                    <NavigationMenuItem>
                                        <NavigationMenuTrigger>
                                            <Avatar className="bg-muted">
                                                <AvatarImage
                                                    src={
                                                        playerData.status?.avatarId
                                                            ? playerData.status.avatar.type === "ASSISTANT"
                                                                ? `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/avatar/${
                                                                      Object.values(playerData.troop.chars)
                                                                          .find((item) => item.skin === playerData.status?.avatar.id ?? "")
                                                                          ?.charId.replaceAll("#", "_") ?? ""
                                                                  }.png`
                                                                : ""
                                                            : "https://static.wikia.nocookie.net/mrfz/images/4/46/Symbol_profile.png/revision/latest?cb=20220418145951"
                                                    }
                                                    alt="@shadcn"
                                                />
                                                <AvatarFallback>{playerData.status?.nickName?.slice(0, 1) ?? "E"}</AvatarFallback>
                                            </Avatar>
                                        </NavigationMenuTrigger>
                                        <NavigationMenuContent>
                                            <ul className="grid w-[400px] gap-3 p-4 md:w-[500px] md:grid-cols-2 lg:w-[600px]">
                                                {profileComponents.map((component, index) => (
                                                    <ListItem key={`profile-component-${index}`} title={component.title} href={component.href}>
                                                        {component.description}
                                                    </ListItem>
                                                ))}
                                            </ul>
                                        </NavigationMenuContent>
                                    </NavigationMenuItem>
                                ) : (
                                    <NavigationMenuItem
                                        className="cursor-pointer"
                                        onClick={() => {
                                            if (loginData?.uid) {
                                                console.log("Already have login data...");
                                                void fetchPlayerData(loginData);
                                            }
                                        }}
                                    >
                                        <DialogTrigger asChild>
                                            <Button variant="outline">Login</Button>
                                        </DialogTrigger>
                                    </NavigationMenuItem>
                                )}
                            </div>
                        </NavigationMenuList>
                        <div className="flex flex-row gap-3">
                            <div className="flex flex-row items-center justify-center">
                                <Input
                                    type="search"
                                    placeholder="Search for players..."
                                    className="cursor-pointer transition-all duration-150 hover:bg-accent"
                                    onClick={() => {
                                        setSearchModel(true);
                                    }}
                                />
                            </div>
                            <ThemeToggle />
                        </div>
                    </NavigationMenu>
                </div>
                <CommandDialog open={searchModel} onOpenChange={setSearchModel}>
                    <CommandInput placeholder="Type a command or search..." />
                    <CommandList>
                        <CommandEmpty>No players found.</CommandEmpty>
                        <CommandGroup heading="Suggestions">
                            <CommandItem className="cursor-pointer">
                                <TrendingUp className="mr-2 h-4 w-4" />
                                <span>Highest Level</span>
                            </CommandItem>
                            <CommandItem className="cursor-pointer">
                                <Users className="mr-2 h-4 w-4" />
                                <span>Most Operators</span>
                            </CommandItem>
                            <CommandItem className="cursor-pointer">
                                <UploadCloud className="mr-2 h-4 w-4" />
                                <span>New Players</span>
                            </CommandItem>
                        </CommandGroup>
                    </CommandList>
                </CommandDialog>
                <DialogContent className="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle>Login</DialogTitle>
                        <DialogDescription>Use your Yostar email to send an OTP message. No login information is stored on the server.</DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4">
                        {submitted ? (
                            <>
                                <div className="grid gap-2">
                                    <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">OTP Code</label>
                                    {hasOTP ? <Input id="login-email" type="email" placeholder="Enter your email..." onKeyUp={checkOTP} /> : null}
                                    <Input id="otp-code" type="number" placeholder="Enter the OTP code..." onKeyUp={checkOTP} />
                                    <span
                                        className="cursor-pointer text-sm underline"
                                        onClick={() => {
                                            setSubmitted(false);
                                            setEmail(null);
                                            setHasOTP(false);
                                            setCanLogin(false);

                                            const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
                                            otpButton?.removeAttribute("disabled");
                                            otpButton?.classList.remove("disabled:cursor-not-allowed");
                                            Object.assign(otpButton ?? {}, {
                                                innerText: "Submit",
                                            });
                                        }}
                                    >
                                        Back to login
                                    </span>
                                </div>
                            </>
                        ) : (
                            <>
                                <div className="grid gap-2">
                                    <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">Email</label>
                                    <Input id="login-email" type="email" placeholder="Enter your email..." />
                                    <span
                                        className="cursor-pointer text-sm underline"
                                        onClick={() => {
                                            setSubmitted(true);
                                            setHasOTP(true);

                                            const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
                                            otpButton?.setAttribute("disabled", "true");
                                            otpButton?.classList.add("disabled:cursor-not-allowed");
                                            Object.assign(otpButton ?? {}, {
                                                innerText: "Login",
                                            });
                                        }}
                                    >
                                        I have an OTP
                                    </span>
                                </div>
                            </>
                        )}
                        <Button
                            id="otp-button"
                            onClick={() => {
                                if (canLogin) {
                                    void login();
                                } else {
                                    void sendOTP();
                                }
                            }}
                        >
                            Submit
                        </Button>
                    </div>
                </DialogContent>
            </Dialog>
        </ClientOnly>
    );
}

// eslint-disable-next-line react/display-name
const ListItem = React.forwardRef<React.ElementRef<"a">, React.ComponentPropsWithoutRef<"a">>(({ className, title, children, ...props }, ref) => {
    return (
        <li>
            <NavigationMenuLink asChild>
                <a ref={ref} className={cn("block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground", className)} {...props}>
                    <div className="text-sm font-medium leading-none">{title}</div>
                    <p className="line-clamp-2 text-sm leading-snug text-muted-foreground">{children}</p>
                </a>
            </NavigationMenuLink>
        </li>
    );
});

export default Navbar;
