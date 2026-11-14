import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { CheckIcon, CopyIcon, KeyRoundIcon, PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { type ReactNode, useEffect, useId, useRef, useState } from "react";
import { usePlayerName } from "#/components/header/impl/use-player-name";
import { Alert } from "#/components/ui/alert";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "#/components/ui/card";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Skeleton } from "#/components/ui/skeleton";
import { Textarea } from "#/components/ui/textarea";
import { toastManager } from "#/components/ui/toast";
import { createOAuthAppFn, deleteOAuthAppFn, listAuthorizationsFn, listOAuthAppsFn, type OAuthAppFields, revokeAuthorizationFn, rotateOAuthSecretFn, updateOAuthAppFn } from "#/lib/api/oauth";
import { REPO_URL } from "#/lib/constants";
import { type TypedRichT, useFormatters, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { OAuthApp } from "#/types/generated/OAuthApp";
import type { OAuthAppOwner } from "#/types/generated/OAuthAppOwner";
import type { OAuthAppWithSecret } from "#/types/generated/OAuthAppWithSecret";
import type { OAuthAuthorization } from "#/types/generated/OAuthAuthorization";
import type { messages } from "./AppsPanel.messages";
import { Mono } from "./SettingsShell";

/** Mirrors `MAX_APPS_PER_USER` and `MAX_REDIRECT_URIS` in the backend's `core::auth::oauth`, which enforces both. */
const MAX_APPS = 10;
const MAX_REDIRECT_URIS = 10;

const DOCS_URL = `${REPO_URL}/blob/main/backend/OAUTH.md`;

// Under "user": signing out drops every query with that prefix, so one
// account's applications never show to the next one on the same browser.
const AUTHORIZATIONS_KEY = ["user", "oauth", "authorizations"] as const;
const APPS_KEY = ["user", "oauth", "apps"] as const;

type T = TypedT<typeof messages>;

/**
 * Settings, Applications: the sites the visitor signed in to with their
 * myrtle.moe account, and the applications they registered themselves.
 */
export function AppsPanel({ userId }: { userId: string }) {
    return (
        <div className="flex flex-col gap-4">
            <AuthorizedCard userId={userId} />
            <DeveloperCard userId={userId} />
        </div>
    );
}

function useToasts() {
    const describeError = useErrorMessage();
    return {
        success: (id: string, title: string, description: string) => toastManager.add({ id: `${id}-${Date.now()}`, title, description, type: "success" }),
        error: (id: string, title: string, err: unknown) => toastManager.add({ id: `${id}-err-${Date.now()}`, title, description: describeError(err), type: "error" }),
    };
}

function useOwnerName(): (owner: OAuthAppOwner) => string {
    const playerName = usePlayerName();
    return (owner) => `${playerName(owner.nickname)}${owner.nick_number ? `#${owner.nick_number}` : ""}`;
}

function ListState({ loading, error, onRetry, failedText, retryText }: { loading: boolean; error: boolean; onRetry: () => void; failedText: string; retryText: string }) {
    if (loading) return <Skeleton className="h-14 w-full rounded-lg" />;
    if (!error) return null;
    return (
        <div className="flex flex-wrap items-center justify-between gap-3">
            <span className="text-[13px] text-destructive-foreground">{failedText}</span>
            <Button variant="outline" size="sm" onClick={onRetry}>
                {retryText}
            </Button>
        </div>
    );
}

function AuthorizedCard({ userId }: { userId: string }) {
    const t: T = useT("settings");
    const toasts = useToasts();
    const ownerName = useOwnerName();
    const { date, relativeShort } = useFormatters();
    const queryClient = useQueryClient();
    const query = useQuery({ queryKey: [...AUTHORIZATIONS_KEY, userId], queryFn: () => listAuthorizationsFn() });

    const revoke = useMutation({
        mutationFn: (entry: OAuthAuthorization) => revokeAuthorizationFn({ data: { id: entry.app.id } }),
        onSuccess: (_void, entry) => {
            queryClient.setQueryData<OAuthAuthorization[]>([...AUTHORIZATIONS_KEY, userId], (list) => list?.filter((e) => e.app.id !== entry.app.id));
            toasts.success("oauth-revoked", t("apps.toast.revoked.title"), t("apps.toast.revoked.body", { app: entry.app.name }));
        },
        onError: (err: unknown) => toasts.error("oauth-revoke", t("apps.toast.revokeFailed.title"), err),
    });

    const entries = query.data ?? [];

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t("apps.authorized.title")}</CardTitle>
                <CardDescription>{t("apps.authorized.desc")}</CardDescription>
            </CardHeader>
            <CardContent className="pt-0">
                <ListState loading={query.isPending} error={query.isError} onRetry={() => query.refetch()} failedText={t("apps.authorized.loadFailed")} retryText={t("apps.retry")} />
                {query.isSuccess && entries.length === 0 ? <p className="m-0 text-[13.5px] text-muted-foreground leading-[1.55]">{t("apps.authorized.empty")}</p> : null}
                {entries.length > 0 ? (
                    <ul className="m-0 flex list-none flex-col p-0">
                        {entries.map((entry) => (
                            <li key={entry.app.id} className="flex flex-col gap-2.5 border-border border-t py-3.5 first:border-t-0 first:pt-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
                                <div className="flex min-w-0 flex-col gap-0.5">
                                    <span className="break-words font-medium text-[14px] text-foreground leading-[1.35]">{entry.app.name}</span>
                                    {entry.app.description ? <span className="break-words text-[13px] text-muted-foreground leading-normal">{entry.app.description}</span> : null}
                                    <span className="text-[12.5px] text-muted-foreground leading-normal">
                                        {t("apps.authorized.by", { owner: ownerName(entry.app.owner) })}
                                        {" · "}
                                        {t("apps.authorized.since", { date: date(entry.authorized_at) })}
                                        {entry.last_used_at ? (
                                            <>
                                                {", "}
                                                {t("apps.authorized.lastUsed", { when: relativeShort(entry.last_used_at) })}
                                            </>
                                        ) : null}
                                    </span>
                                </div>
                                <Button
                                    variant="destructive-outline"
                                    size="sm"
                                    className="shrink-0 self-start sm:self-center"
                                    aria-label={t("apps.authorized.revokeLabel", { app: entry.app.name })}
                                    loading={revoke.isPending && revoke.variables?.app.id === entry.app.id}
                                    disabled={revoke.isPending}
                                    onClick={() => revoke.mutate(entry)}
                                >
                                    {t("apps.authorized.revoke")}
                                </Button>
                            </li>
                        ))}
                    </ul>
                ) : null}
            </CardContent>
        </Card>
    );
}

/** Which dialog is open over the developer card, and for which application. */
type Open = { kind: "create" } | { kind: "edit"; app: OAuthApp } | { kind: "rotate"; app: OAuthApp } | { kind: "delete"; app: OAuthApp } | { kind: "secret"; app: OAuthAppWithSecret; rotated: boolean } | null;

function DeveloperCard({ userId }: { userId: string }) {
    const t: T = useT("settings");
    const rt: TypedRichT<typeof messages> = useRichT("settings");
    const toasts = useToasts();
    const queryClient = useQueryClient();
    const key = [...APPS_KEY, userId];
    const query = useQuery({ queryKey: key, queryFn: () => listOAuthAppsFn() });
    const [open, setOpen] = useState<Open>(null);
    const close = () => setOpen(null);

    /** Put `app` into the cached list (replacing its old row, or appended), minus any secret. */
    const store = ({ client_secret: _secret, ...app }: OAuthApp & { client_secret?: string }) => {
        queryClient.setQueryData<OAuthApp[]>(key, (list = []) => (list.some((a) => a.id === app.id) ? list.map((a) => (a.id === app.id ? app : a)) : [...list, app]));
    };

    const rotate = useMutation({
        mutationFn: (app: OAuthApp) => rotateOAuthSecretFn({ data: { id: app.id } }),
        onSuccess: (app) => {
            store(app);
            setOpen({ kind: "secret", app, rotated: true });
            // The dialog now holds the only copy; the mutation must not keep one.
            rotate.reset();
        },
        onError: (err: unknown) => {
            close();
            toasts.error("oauth-rotate", t("apps.toast.rotateFailed.title"), err);
        },
    });

    const remove = useMutation({
        mutationFn: (app: OAuthApp) => deleteOAuthAppFn({ data: { id: app.id } }),
        onSuccess: (_void, app) => {
            queryClient.setQueryData<OAuthApp[]>(key, (list) => list?.filter((a) => a.id !== app.id));
            // The visitor may have authorized their own application.
            void queryClient.invalidateQueries({ queryKey: [...AUTHORIZATIONS_KEY, userId] });
            close();
            toasts.success("oauth-deleted", t("apps.toast.deleted.title"), t("apps.toast.deleted.body", { app: app.name }));
        },
        onError: (err: unknown) => {
            close();
            toasts.error("oauth-delete", t("apps.toast.deleteFailed.title"), err);
        },
    });

    const apps = query.data ?? [];
    const atLimit = apps.length >= MAX_APPS;

    return (
        <Card>
            <CardHeader>
                <CardTitle>{t("apps.dev.title")}</CardTitle>
                <CardDescription>
                    {rt("apps.dev.desc", {
                        docs: (
                            <a href={DOCS_URL} target="_blank" rel="noreferrer" className="text-foreground underline underline-offset-4 hover:text-primary">
                                {t("apps.dev.docsLink")}
                            </a>
                        ),
                    })}
                </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-4 pt-0">
                <ListState loading={query.isPending} error={query.isError} onRetry={() => query.refetch()} failedText={t("apps.dev.loadFailed")} retryText={t("apps.retry")} />
                {query.isSuccess && apps.length === 0 ? <p className="m-0 text-[13.5px] text-muted-foreground leading-[1.55]">{t("apps.dev.empty")}</p> : null}
                {apps.length > 0 ? (
                    <ul className="m-0 flex list-none flex-col gap-3 p-0">
                        {apps.map((app) => (
                            <li key={app.id} className="flex flex-col gap-3 rounded-xl border border-border p-4">
                                <div className="flex flex-col gap-0.5">
                                    <span className="break-words font-medium text-[14px] text-foreground leading-[1.35]">{app.name}</span>
                                    {app.description ? <span className="break-words text-[13px] text-muted-foreground leading-normal">{app.description}</span> : null}
                                    <span className="text-[12.5px] text-muted-foreground leading-normal">{t("apps.dev.users", { count: app.authorized_users })}</span>
                                </div>
                                <CopyField label={t("apps.dev.clientId")} value={app.id} />
                                <div className="flex flex-col gap-1">
                                    <span className="font-medium text-[12.5px] text-foreground">{t("apps.dev.redirectUris")}</span>
                                    <ul className="m-0 flex list-none flex-col gap-0.5 p-0">
                                        {app.redirect_uris.map((uri) => (
                                            <li key={uri} className="break-all">
                                                <Mono>{uri}</Mono>
                                            </li>
                                        ))}
                                    </ul>
                                </div>
                                <div className="flex flex-wrap gap-2">
                                    <Button variant="outline" size="sm" onClick={() => setOpen({ kind: "edit", app })}>
                                        <PencilIcon className="size-3.5" />
                                        {t("apps.dev.edit")}
                                    </Button>
                                    <Button variant="outline" size="sm" onClick={() => setOpen({ kind: "rotate", app })}>
                                        <KeyRoundIcon className="size-3.5" />
                                        {t("apps.dev.rotate")}
                                    </Button>
                                    <Button variant="destructive-outline" size="sm" onClick={() => setOpen({ kind: "delete", app })}>
                                        <Trash2Icon className="size-3.5" />
                                        {t("apps.dev.delete")}
                                    </Button>
                                </div>
                            </li>
                        ))}
                    </ul>
                ) : null}
                {query.isSuccess ? (
                    atLimit ? (
                        <p className="m-0 text-[13px] text-muted-foreground">{t("apps.dev.limit", { max: MAX_APPS })}</p>
                    ) : (
                        <div>
                            <Button variant="outline" onClick={() => setOpen({ kind: "create" })}>
                                <PlusIcon className="size-4" />
                                {t("apps.dev.new")}
                            </Button>
                        </div>
                    )
                ) : null}
            </CardContent>

            <AppFormDialog
                // Remounted per target, so the fields start from that application's values.
                key={open?.kind === "edit" ? open.app.id : "create"}
                open={open?.kind === "create" || open?.kind === "edit"}
                editing={open?.kind === "edit" ? open.app : null}
                onClose={close}
                onCreated={(app) => {
                    store(app);
                    setOpen({ kind: "secret", app, rotated: false });
                }}
                onSaved={(app) => {
                    store(app);
                    close();
                    toasts.success("oauth-saved", t("apps.toast.saved.title"), t("apps.toast.saved.body", { app: app.name }));
                }}
            />

            <ConfirmDialog
                open={open?.kind === "rotate"}
                title={t("apps.rotate.title")}
                body={open?.kind === "rotate" ? t("apps.rotate.body", { app: open.app.name }) : ""}
                confirm={t("apps.rotate.confirm")}
                cancel={t("apps.form.cancel")}
                pending={rotate.isPending}
                onConfirm={() => open?.kind === "rotate" && rotate.mutate(open.app)}
                onClose={close}
            />
            <ConfirmDialog
                open={open?.kind === "delete"}
                title={open?.kind === "delete" ? t("apps.delete.title", { app: open.app.name }) : ""}
                body={t("apps.delete.body")}
                confirm={t("apps.delete.confirm")}
                cancel={t("apps.form.cancel")}
                pending={remove.isPending}
                onConfirm={() => open?.kind === "delete" && remove.mutate(open.app)}
                onClose={close}
            />

            <SecretDialog shown={open?.kind === "secret" ? open : null} onClose={close} />
        </Card>
    );
}

/** A labelled read-only value with a Copy button. */
function CopyField({ label, value }: { label: string; value: string }) {
    const t: T = useT("settings");
    const [copied, setCopied] = useState(false);
    const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
    useEffect(
        () => () => {
            if (timer.current) clearTimeout(timer.current);
        },
        [],
    );

    const copy = async () => {
        try {
            await navigator.clipboard.writeText(value);
        } catch {
            // No clipboard permission: the value is on screen and selectable.
            return;
        }
        setCopied(true);
        if (timer.current) clearTimeout(timer.current);
        timer.current = setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div className="flex flex-col gap-1">
            <span className="font-medium text-[12.5px] text-foreground">{label}</span>
            <div className="flex items-center gap-2">
                <code className="min-w-0 flex-1 select-all break-all rounded-lg border border-border bg-muted/50 px-2.5 py-1.5 font-mono text-[12px] text-foreground/85">{value}</code>
                <Button variant="outline" size="sm" className="shrink-0" aria-label={t("apps.copyLabel", { what: label })} onClick={() => void copy()}>
                    {copied ? <CheckIcon className="size-3.5" /> : <CopyIcon className="size-3.5" />}
                    {/* Announced when it changes; the button's name stays "Copy <label>". */}
                    <span aria-live="polite">{copied ? t("apps.copied") : t("apps.copy")}</span>
                </Button>
            </div>
        </div>
    );
}

function AppFormDialog({ open, editing, onClose, onCreated, onSaved }: { open: boolean; editing: OAuthApp | null; onClose: () => void; onCreated: (app: OAuthAppWithSecret) => void; onSaved: (app: OAuthApp) => void }) {
    const t: T = useT("settings");
    const describeError = useErrorMessage();
    const [name, setName] = useState(editing?.name ?? "");
    const [description, setDescription] = useState(editing?.description ?? "");
    const [uris, setUris] = useState(editing?.redirect_uris.join("\n") ?? "");
    const [problem, setProblem] = useState<string | null>(null);
    const nameId = useId();
    const descriptionId = useId();
    const urisId = useId();
    const hintId = useId();

    const save = useMutation({
        mutationFn: async (fields: OAuthAppFields): Promise<{ created: OAuthAppWithSecret } | { saved: OAuthApp }> => (editing ? { saved: await updateOAuthAppFn({ data: { ...fields, id: editing.id } }) } : { created: await createOAuthAppFn({ data: fields }) }),
        onSuccess: (result) => {
            if ("created" in result) onCreated(result.created);
            else onSaved(result.saved);
            // A created application carries its secret; the mutation must not keep it.
            save.reset();
        },
        // The backend's reasons name the offending redirect URI; one per line.
        onError: (err: unknown) => setProblem(describeError(err)),
    });

    const submit = () => {
        const fields: OAuthAppFields = {
            name: name.trim(),
            description: description.trim(),
            redirect_uris: uris
                .split("\n")
                .map((line) => line.trim())
                .filter(Boolean),
        };
        if (!fields.name) return setProblem(t("apps.form.nameRequired"));
        if (fields.redirect_uris.length === 0) return setProblem(t("apps.form.redirectRequired"));
        setProblem(null);
        save.mutate(fields);
    };

    return (
        <Dialog open={open} onOpenChange={(next) => !next && !save.isPending && onClose()}>
            <DialogPopup className="sm:max-w-lg">
                <form
                    className="flex min-h-0 flex-col"
                    onSubmit={(e) => {
                        e.preventDefault();
                        submit();
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>{editing ? t("apps.form.editTitle") : t("apps.form.createTitle")}</DialogTitle>
                        <DialogDescription>{t("apps.form.intro")}</DialogDescription>
                    </DialogHeader>
                    <DialogPanel className="flex flex-col gap-4">
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor={nameId}>{t("apps.form.name")}</Label>
                            <Input id={nameId} value={name} onChange={(e) => setName(e.target.value)} maxLength={64} placeholder={t("apps.form.namePlaceholder")} autoComplete="off" disabled={save.isPending} />
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor={descriptionId}>{t("apps.form.description")}</Label>
                            <Input id={descriptionId} value={description} onChange={(e) => setDescription(e.target.value)} maxLength={280} autoComplete="off" disabled={save.isPending} aria-describedby={`${descriptionId}-hint`} />
                            <span id={`${descriptionId}-hint`} className="text-[12.5px] text-muted-foreground leading-normal">
                                {t("apps.form.descriptionHint")}
                            </span>
                        </div>
                        <div className="flex flex-col gap-1.5">
                            <Label htmlFor={urisId}>{t("apps.form.redirectUris")}</Label>
                            <Textarea id={urisId} value={uris} onChange={(e) => setUris(e.target.value)} rows={3} placeholder="https://example.com/auth/myrtle/callback" spellCheck={false} autoCapitalize="off" autoCorrect="off" className="font-mono text-[12.5px]" disabled={save.isPending} aria-describedby={hintId} />
                            <span id={hintId} className="text-[12.5px] text-muted-foreground leading-normal">
                                {t("apps.form.redirectHint", { max: MAX_REDIRECT_URIS })}
                            </span>
                        </div>
                        {problem ? (
                            <Alert variant="error" className="whitespace-pre-line break-words text-[13px]">
                                {problem}
                            </Alert>
                        ) : null}
                    </DialogPanel>
                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" disabled={save.isPending} />}>{t("apps.form.cancel")}</DialogClose>
                        <Button type="submit" loading={save.isPending}>
                            {editing ? t("apps.form.save") : t("apps.form.create")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}

function ConfirmDialog({ open, title, body, confirm, cancel, pending, onConfirm, onClose }: { open: boolean; title: string; body: ReactNode; confirm: string; cancel: string; pending: boolean; onConfirm: () => void; onClose: () => void }) {
    return (
        <AlertDialog open={open} onOpenChange={(next) => !next && !pending && onClose()}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle className="break-words">{title}</AlertDialogTitle>
                    <AlertDialogDescription>{body}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button type="button" variant="outline" disabled={pending} />}>{cancel}</AlertDialogClose>
                    <Button type="button" variant="destructive" loading={pending} onClick={onConfirm}>
                        {confirm}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}

/**
 * Shows a just-minted client secret. Closes only by its button: a stray click
 * outside must not lose the one view of it. The secret is rendered only while
 * the dialog is open and is not kept anywhere once it closes.
 */
function SecretDialog({ shown, onClose }: { shown: { app: OAuthAppWithSecret; rotated: boolean } | null; onClose: () => void }) {
    const t: T = useT("settings");
    // Only the wording outlives the close, so the title does not flip while
    // the popup animates out.
    const [rotated, setRotated] = useState(false);
    useEffect(() => {
        if (shown) setRotated(shown.rotated);
    }, [shown]);
    const isRotation = shown?.rotated ?? rotated;

    return (
        <AlertDialog open={shown !== null}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>{isRotation ? t("apps.secret.rotatedTitle") : t("apps.secret.createdTitle")}</AlertDialogTitle>
                    <AlertDialogDescription>{t("apps.secret.body")}</AlertDialogDescription>
                    {isRotation ? <p className="m-0 text-muted-foreground text-sm">{t("apps.secret.rotatedNote")}</p> : null}
                </AlertDialogHeader>
                {shown ? (
                    <div className="flex flex-col gap-3 px-6 pb-5 text-left">
                        <CopyField label={t("apps.dev.clientId")} value={shown.app.id} />
                        <CopyField label={t("apps.secret.label")} value={shown.app.client_secret} />
                    </div>
                ) : null}
                <AlertDialogFooter>
                    <Button type="button" onClick={onClose}>
                        {t("apps.secret.done")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
