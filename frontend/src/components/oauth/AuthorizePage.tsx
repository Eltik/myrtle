import { useMutation, useQuery } from "@tanstack/react-query";
import { ArrowLeftRightIcon, CheckIcon, LogInIcon, XIcon } from "lucide-react";
import { type ReactNode, useEffect } from "react";
import { usePlayerName } from "#/components/header/impl/use-player-name";
import { Button } from "#/components/ui/button";
import { Card } from "#/components/ui/card";
import { useErrorMessage } from "#/components/ui/error-message";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Spinner } from "#/components/ui/spinner";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type AuthorizeRequest, checkAuthorizationFn, decideAuthorizationFn } from "#/lib/api/oauth";
import { authActions } from "#/lib/auth/store";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { AuthorizeCheck } from "#/types/generated/AuthorizeCheck";
import type { messages } from "./AuthorizePage.messages";

type Consent = Extract<AuthorizeCheck, { status: "consent" }>;

/**
 * The consent page: where another site sends a player to confirm who they
 * are. Signed out, it asks them to sign in. Signed in, it shows who is asking
 * and what they would learn, and sends the browser back with the answer.
 *
 * `request` is the other site's authorization request, unpacked by the route.
 */
export function AuthorizePage({ request }: { request: AuthorizeRequest }) {
    const t: TypedT<typeof messages> = useT("oauth");
    const describeError = useErrorMessage();
    const { user } = useAuth();

    const check = useQuery({
        // Keyed on the account: signing out and in as someone else asks again.
        queryKey: ["oauth", "authorize", user?.id ?? null, request],
        queryFn: () => checkAuthorizationFn({ data: request }),
        enabled: user !== null,
        retry: false,
        staleTime: 0,
        gcTime: 0,
        refetchOnWindowFocus: false,
    });

    const decide = useMutation({
        mutationFn: (approve: boolean) => decideAuthorizationFn({ data: { ...request, approve } }),
        onSuccess: ({ redirect_to }) => window.location.assign(redirect_to),
        onError: (err: unknown) => toastManager.add({ id: `oauth-decide-${Date.now()}`, title: t("authorize.decideFailed"), description: describeError(err), type: "error" }),
    });

    // The request is wrong in a way its sender is told about, not the player.
    const errorRedirect = check.data?.status === "redirect" ? check.data.redirect_to : null;
    useEffect(() => {
        if (errorRedirect) window.location.replace(errorRedirect);
    }, [errorRedirect]);

    let body: ReactNode;
    if (!user) {
        body = (
            <Notice title={t("authorize.signIn.title")} description={t("authorize.signIn.body")}>
                <Button onClick={() => authActions.openLoginDialog()}>
                    <LogInIcon aria-hidden="true" />
                    {t("authorize.signIn.action")}
                </Button>
            </Notice>
        );
    } else if (check.data?.status === "invalid") {
        // Nowhere trustworthy to send the browser, so the reason is shown here,
        // in the backend's own words: it is the other site's developer who
        // needs it.
        body = (
            <Notice title={t("authorize.invalid.title")} description={t("authorize.invalid.body")}>
                <code className="block w-full break-words rounded-lg border border-border bg-muted/50 px-3 py-2 text-left font-mono text-[12.5px] text-foreground/85">{check.data.reason}</code>
            </Notice>
        );
    } else if (check.isError) {
        body = (
            <Notice title={t("authorize.failed.title")} description={describeError(check.error)}>
                <Button variant="outline" onClick={() => check.refetch()} loading={check.isFetching}>
                    {t("authorize.failed.retry")}
                </Button>
            </Notice>
        );
    } else if (check.data?.status === "consent") {
        // Stays busy after a decision: the browser is on its way out.
        body = <ConsentCard consent={check.data} busy={decide.isPending || decide.isSuccess} onDecide={(approve) => decide.mutate(approve)} />;
    } else {
        body = (
            <output className="flex items-center justify-center gap-2.5 py-16 text-[14px] text-muted-foreground">
                <Spinner className="size-4" />
                {errorRedirect ? t("authorize.redirecting") : t("authorize.loading")}
            </output>
        );
    }

    return (
        // A `<div>`: the site shell already wraps every page in the one `<main>`.
        <div className="page-gutter flex min-h-[calc(100svh-var(--site-header-height))] flex-col items-center justify-center py-10 [--page-max:460px]">{body}</div>
    );
}

function Notice({ title, description, children }: { title: string; description: string; children: ReactNode }) {
    return (
        <Card className="w-full">
            <div className="flex flex-col items-start gap-4 p-6">
                <div className="flex flex-col gap-1.5">
                    <h1 className="m-0 font-(--font-heading) font-semibold text-[20px] text-foreground leading-tight tracking-[-0.01em]">{title}</h1>
                    <p className="m-0 text-[14px] text-muted-foreground leading-[1.55]">{description}</p>
                </div>
                {children}
            </div>
        </Card>
    );
}

function ConsentCard({ consent, busy, onDecide }: { consent: Consent; busy: boolean; onDecide: (approve: boolean) => void }) {
    const t: TypedT<typeof messages> = useT("oauth");
    const rt: TypedRichT<typeof messages> = useRichT("oauth");
    const playerName = usePlayerName();
    const { user, logout } = useAuth();
    const { app } = consent;

    // Literal keys, so the extractor sees each one used. A scope this page has
    // no wording for is listed by its raw name.
    const sees = consent.scopes.flatMap((scope) => (scope === "identify" ? [t("authorize.scope.identify.name"), t("authorize.scope.identify.account"), t("authorize.scope.identify.avatar")] : [scope]));
    const cannot = [t("authorize.cannot.private"), t("authorize.cannot.change"), t("authorize.cannot.game")];
    const owner = `${playerName(app.owner.nickname)}${app.owner.nick_number ? `#${app.owner.nick_number}` : ""}`;
    const me = user ? `${playerName(user.nickname)}${user.nick_number ? `#${user.nick_number}` : ""}` : "";

    return (
        <Card className="w-full">
            <div className="flex flex-col gap-5 p-6">
                <div className="flex items-center justify-center gap-3" aria-hidden="true">
                    <span className="flex size-14 items-center justify-center rounded-2xl border border-border bg-muted font-(--font-heading) font-bold text-[22px] text-foreground">{app.name.charAt(0).toUpperCase()}</span>
                    <ArrowLeftRightIcon className="size-4 text-muted-foreground" />
                    <span className="relative flex size-14 items-center justify-center overflow-hidden rounded-full border-2 border-primary/30 bg-linear-to-br from-primary to-primary/60 font-bold text-[20px] text-primary-foreground">
                        <OperatorAvatar charId={user?.secretary_skin_id ?? user?.secretary} name={me} />
                    </span>
                </div>

                <div className="flex flex-col gap-1.5 text-center">
                    <h1 className="m-0 break-words font-(--font-heading) font-semibold text-[20px] text-foreground leading-tight tracking-[-0.01em]">{t("authorize.title", { app: app.name })}</h1>
                    {app.description ? <p className="m-0 break-words text-[14px] text-foreground/85 leading-[1.5]">{app.description}</p> : null}
                    <p className="m-0 text-[13px] text-muted-foreground leading-[1.5]">{t("authorize.by", { owner })}</p>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 rounded-lg border border-border bg-muted/40 px-3 py-2 text-[13px]">
                    <span className="min-w-0 truncate text-foreground">{t("authorize.account", { name: me })}</span>
                    <button type="button" className="cursor-pointer rounded-sm text-muted-foreground underline underline-offset-4 outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring" disabled={busy} onClick={() => void logout()}>
                        {t("authorize.switch")}
                    </button>
                </div>

                <div className="flex flex-col gap-4">
                    <PermissionList label={t("authorize.willSee")} tone="allow" items={sees} />
                    <PermissionList label={t("authorize.wontSee")} tone="deny" items={cannot} />
                </div>

                {consent.already_authorized ? <p className="m-0 rounded-lg border border-border bg-muted/40 px-3 py-2 text-[13px] text-foreground">{t("authorize.again")}</p> : null}

                <div className="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
                    <Button variant="outline" size="lg" disabled={busy} onClick={() => onDecide(false)}>
                        {t("authorize.deny")}
                    </Button>
                    <Button size="lg" loading={busy} onClick={() => onDecide(true)}>
                        {t("authorize.approve")}
                    </Button>
                </div>

                <p className="m-0 text-[12.5px] text-muted-foreground leading-[1.55]">{rt("authorize.redirectNote", { host: <code className="font-mono text-[12px] text-foreground/85">{consent.redirect_host}</code> })}</p>
            </div>
        </Card>
    );
}

function PermissionList({ label, tone, items }: { label: string; tone: "allow" | "deny"; items: string[] }) {
    const Icon = tone === "allow" ? CheckIcon : XIcon;
    return (
        <div className="flex flex-col gap-1.5">
            <span className="font-medium text-[13px] text-foreground">{label}</span>
            <ul className="m-0 flex list-none flex-col gap-1 p-0 text-[13.5px] text-muted-foreground leading-[1.5]">
                {items.map((item) => (
                    <li key={item} className="flex items-start gap-2">
                        <Icon className={cn("mt-[3px] size-3.5 shrink-0", tone === "allow" ? "text-success-foreground" : "text-muted-foreground")} aria-hidden="true" />
                        <span className="min-w-0 break-words">{item}</span>
                    </li>
                ))}
            </ul>
        </div>
    );
}
