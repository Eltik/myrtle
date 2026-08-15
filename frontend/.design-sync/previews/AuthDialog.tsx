import { AuthDialog, Button } from "frontend";
import { type ReactNode, useEffect } from "react";

// The Yostar login modal. `UserMenu` mounts it behind a "Login" button and
// `MobileNav` behind a drawer row; both pass only a `trigger`. The dialog also
// accepts a controlled `open`, which is how these stories photograph it.
//
// Step 2 (the six-slot OTP field) is only reachable after the send-code mutation
// resolves, and that mutation is a stubbed server function in a preview — so the
// email + server step is the state a card can honestly show.

const noop = () => {};

// An open Base UI dialog auto-focuses its first tabbable control, which lands the
// brand-red focus ring on the empty email input and reads as a validation error.
// Two frames after mount, blur it (the dialog stays open). The same helper types
// the email for the story that needs a filled form: the field is bound to the
// auth store through `onChange`, so it has to be driven as a real input event.
function Stage({ email, children }: { email?: string; children: ReactNode }) {
    useEffect(() => {
        let f2 = 0;
        let f3 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                if (email) {
                    const input = document.querySelector<HTMLInputElement>('input[type="email"]');
                    const setValue = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value")?.set;
                    if (input && setValue) {
                        setValue.call(input, email);
                        input.dispatchEvent(new Event("input", { bubbles: true }));
                    }
                }
                f3 = requestAnimationFrame(() => (document.activeElement as HTMLElement | null)?.blur());
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
            cancelAnimationFrame(f3);
        };
    }, [email]);
    return <div className="min-h-[520px] w-full">{children}</div>;
}

/** Step 1, untouched: "Send Code" stays disabled until an email is entered. */
export const Open = () => (
    <Stage>
        <AuthDialog open onOpenChange={noop} trigger={<Button>Login</Button>} />
    </Stage>
);

/** Step 1 with a Yostar address filled in — the submit button becomes enabled. */
export const EmailEntered = () => (
    <Stage email="doctor@rhodes.island">
        <AuthDialog open onOpenChange={noop} trigger={<Button>Login</Button>} />
    </Stage>
);

/** Closed: how the header mounts it — the dialog is just its trigger until opened. */
export const FromHeaderButton = () => (
    <div className="flex w-fit items-center gap-2 rounded-lg border border-border bg-background px-2 py-1.5">
        <span className="px-1 font-sans text-muted-foreground text-sm">Not signed in</span>
        <AuthDialog trigger={<Button>Login</Button>} />
    </div>
);
