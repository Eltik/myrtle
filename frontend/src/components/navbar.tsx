import Link from "next/link";
import { useEffect, useState } from "react";
import { useStore } from "zustand";
import { useLogin } from "~/store/store";
import { type Server, type CodeResponse, type LoginResponse } from "~/types/types";

function Navbar({ children }: { children: React.ReactNode }) {
    const loginData = useStore(useLogin, (state) => (state as unknown as { loginData: LoginResponse | null } | null)?.loginData as LoginResponse | null)

    const [email, setEmail] = useState<string | null>(null);

    const [server, setServer] = useState<Server>("en");

    const [showLogin, setShowLogin] = useState(false);
    const [submitted, setSubmitted] = useState(false);
    const [canLogin, setCanLogin] = useState(false);
    const [hasOTP, setHasOTP] = useState(false);
    
    const [timer, setTimer] = useState(60);

    useEffect(() => {
        console.log(loginData);
    }, []);

    useEffect(() => {
        if (hasOTP || !submitted || timer <= 0) return;

        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");

        let interval: NodeJS.Timeout | null = null;

        const countdown = () => {
            interval = setInterval(() => {
                setTimer(timer - 1)
            }, 1000);
            if (!canLogin) {
                Object.assign(otpButton ?? {}, {
                    innerText: `Resend in ${timer}...`
                });
            }
            if (timer === 1) {
                setTimer(0);
                otpButton?.removeAttribute("disabled");
                otpButton?.classList.remove("disabled:cursor-not-allowed");
                if (!canLogin) {
                    Object.assign(otpButton ?? {}, {
                        innerText: "Resend"
                    });
                }
            }
        };

        countdown();

        return () => clearInterval(interval!);
    }, [timer, submitted]);

    const sendOTP = async() => {
        if (submitted) return;

        const email: HTMLInputElement | null = document.querySelector("#login-email");
        if (!email) return alert("Email element not found. This is an error. Please contact the developer.");

        setEmail(email.value);

        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
        Object.assign(otpButton ?? {}, {
            innerText: `Sending...`
        });
        otpButton?.setAttribute("disabled", "true");
        otpButton?.classList.add("disabled:cursor-not-allowed");

        const data = await (await fetch("/api/code", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                email: email.value,
                server
            })
        })).json() as CodeResponse;
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
                innerText: "Submit"
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
                innerText: "Login"
            });
        } else {
            setCanLogin(false);
            otpButton?.setAttribute("disabled", "true");
            otpButton?.classList.add("disabled:cursor-not-allowed");
        }
    };

    const login = async() => {
        const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
        Object.assign(otpButton ?? {}, {
            innerText: `Sending...`
        });
        otpButton?.setAttribute("disabled", "true");
        otpButton?.classList.add("disabled:cursor-not-allowed");

        const otp: HTMLInputElement | null = document.querySelector("#otp-code");
        // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion
        const curEmail: string | null = hasOTP ? (document.querySelector("#login-email")! as HTMLInputElement)?.value : email;
        if (otp && otp.value.length > 0 && curEmail && curEmail.length > 0) {
            console.log("Logging in...");
            console.log(curEmail);
            console.log(otp);
            const data = await (await fetch("/api/login", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json"
                },
                body: JSON.stringify({
                    email: curEmail,
                    code: otp.value,
                    server
                })
            })).json() as LoginResponse;

            // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
            useLogin.setState({ loginData: data });
            
            setShowLogin(false);
        } else {
            console.log("OTP or email is empty.")
            console.log(otp?.value);
            console.log(email);
        }
    };

    return (
        <>
            <div className={`${showLogin ? "blur-sm" : ""} bg-main-blue-100 transition-all duration-150 w-full h-full`}>
                <header
                    className="sticky top-0 flex h-16 w-full flex-col justify-around bg-main-blue-200/80 p-[0_24px] backdrop-blur-sm"
                    style={{
                        transform: "translateZ(0)",
                        boxShadow: "inset 0 -1px 0 0 #333",
                    }}
                >
                    <nav
                        className="relative flex w-full items-center"
                        style={{
                            flex: "1 1",
                        }}
                    >
                        <div className="flex w-full items-center justify-between md:hidden ">
                            <div className="flex flex-row items-center justify-start gap-1">
                                <Link href={"/"}>
                                    <svg aria-label="Vercel logomark" height="22" role="img" className="w-auto overflow-visible" viewBox="0 0 74 64">
                                        <path d="M37.5896 0.25L74.5396 64.25H0.639648L37.5896 0.25Z" fill="#fff"></path>
                                    </svg>
                                </Link>
                                <Link href={"/"} target="_blank" className="flex text-white">
                                    myrtle.moe
                                </Link>
                            </div>
                            <div className="flex items-center">
                                <button type="button" className="background-none flex justify-center p-0 text-gray-200">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                        <circle cx="11" cy="11" r="8" />
                                        <path d="m21 21-4.3-4.3" />
                                    </svg>
                                </button>
                                <button type="button" className="m-[0_-5px_0_5px] flex cursor-pointer justify-center rounded-md border-none bg-transparent text-gray-200">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                        <line x1="4" x2="20" y1="12" y2="12" />
                                        <line x1="4" x2="20" y1="6" y2="6" />
                                        <line x1="4" x2="20" y1="18" y2="18" />
                                    </svg>
                                </button>
                            </div>
                        </div>
                        <div className="hidden w-full items-center gap-6 md:flex">
                            <div className="flex flex-row items-center justify-start gap-4">
                                <Link href={"/"}>
                                    <svg aria-label="Vercel logomark" height="22" role="img" className="w-auto overflow-visible" viewBox="0 0 74 64">
                                        <path d="M37.5896 0.25L74.5396 64.25H0.639648L37.5896 0.25Z" fill="#fff"></path>
                                    </svg>
                                </Link>
                                <Link href={"/"} target="_blank" className="flex text-white">
                                    myrtle.moe
                                </Link>
                            </div>
                            <Link href={"/players"} className="relative ml-5 text-main-pink-400 transition-all duration-150 ease-in-out hover:text-main-dark-pink-400">
                                Players
                            </Link>
                            <Link href={"/statistics"} className="relative ml-5 text-main-pink-400 transition-all duration-150 ease-in-out hover:text-main-dark-pink-400">
                                Statistics
                            </Link>
                        </div>
                        <div className="hidden items-center gap-2 md:flex">
                            <button type="button" className="flex h-8 cursor-pointer items-center justify-between whitespace-nowrap rounded-md bg-main-blue-500 p-[0_6px_0_8px] text-gray-50 transition-all duration-150 ease-in-out hover:bg-main-blue-500/70">
                                Search players...
                                <kbd
                                    className="ml-4 h-5 rounded-md bg-main-blue-200 p-[0_6px] text-xs text-gray-50"
                                    style={{
                                        lineHeight: "20px",
                                        boxShadow: "0 0 0 1px hsla(0,0%,100%,.145)",
                                    }}
                                >
                                    ⌘K
                                </kbd>
                            </button>
                            <button type="button" className="relative m-0 flex h-8 max-w-full cursor-pointer items-center justify-center rounded-md border-0 bg-main-blue-400 p-[0_12px] font-bold text-gray-50 outline-none transition-all duration-150 ease-in-out hover:bg-main-blue-400/70" onClick={() => {
                                setShowLogin(true);
                            }}>
                                <span className="mr-1 flex min-w-5 items-center justify-center">
                                    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="">
                                        <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
                                        <polyline points="10 17 15 12 10 7" />
                                        <line x1="15" x2="3" y1="12" y2="12" />
                                    </svg>
                                </span>
                                <span className="inline-block overflow-hidden text-ellipsis whitespace-nowrap p-[0_6px]">Login</span>
                            </button>
                        </div>
                    </nav>
                </header>
                <div onClick={() => {
                    setShowLogin(false);
                }}>
                    {children}
                </div>
            </div>
            <div className={`fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-white p-6 shadow-lg duration-200 ${showLogin ? "opacity-100 scale-100" : "opacity-0 pointer-events-none scale-95"} sm:rounded-lg sm:max-w-[425px]`}>
                <div className="flex flex-col space-y-1.5 text-center sm:text-left">
                    <h2 className="text-lg font-semibold leading-none tracking-tight">Login</h2>
                    <p className="text-sm text-main-grey-200">Use your Yostar email to send an OTP message. No login information is stored on the server.</p>
                </div>
                <div className="grid gap-4">
                    {submitted ? 
                    <>
                        <div className="grid gap-2">
                            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">OTP Code</label>
                            {hasOTP ? <input id="login-email" className="flex h-10 w-full rounded-md border border-input bg-white px-3 py-2 text-sm ring-offset-white file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-main-grey-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50" type="email" placeholder="Enter your email..." onKeyUp={checkOTP} /> : null}
                            <input id="otp-code" className="flex h-10 w-full rounded-md border border-input bg-white px-3 py-2 text-sm ring-offset-white file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-main-grey-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50" type="number" placeholder="Enter the OTP code..." onKeyUp={checkOTP} />
                            <span className="text-sm underline cursor-pointer" onClick={() => {
                                setSubmitted(false);
                                setEmail(null);
                                setHasOTP(false);
                                setCanLogin(false);

                                const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
                                otpButton?.removeAttribute("disabled");
                                otpButton?.classList.remove("disabled:cursor-not-allowed");
                                Object.assign(otpButton ?? {}, {
                                    innerText: "Submit"
                                });
                            }}>Back to login</span>
                        </div>
                    </>
                    :
                    <>
                        <div className="grid gap-2">
                            <label className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">Email</label>
                            <input id="login-email" className="flex h-10 w-full rounded-md border border-input bg-white px-3 py-2 text-sm ring-offset-white file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-main-grey-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50" type="email" placeholder="Enter your email..." />
                            <span className="text-sm underline cursor-pointer" onClick={() => {
                                setSubmitted(true);
                                setHasOTP(true);
                                
                                const otpButton: HTMLButtonElement | null = document.querySelector("#otp-button");
                                otpButton?.setAttribute("disabled", "true");
                                otpButton?.classList.add("disabled:cursor-not-allowed");
                                Object.assign(otpButton ?? {}, {
                                    innerText: "Login"
                                });
                            }}>I have an OTP</span>
                        </div>
                    </>
                    }
                    <button id="otp-button" type="button" className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-main-blue-100 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-main-blue-300 text-white hover:bg-main-blue-200/90 h-10 px-4 py-2" onClick={() => {
                        if (canLogin) {
                            void login();
                        } else {
                            void sendOTP();
                        }
                    }}>
                        Submit
                    </button>
                </div>
            </div>
        </>
    );
}

export default Navbar;
